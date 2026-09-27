use clap::Parser;
use flate2::read::GzDecoder;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};

const DEFAULT_BUOYS: [&str; 9] = [
    "42001", "42002", "42003", "42020", "42035", "42036", "42039", "42040", "42055",
];

const DEFAULT_START_YEAR: i32 = 2005;
const DEFAULT_END_YEAR: i32 = 2017;
const HISTORICAL_BASE_URL: &str = "https://www.ndbc.noaa.gov/view_text_file.php";
const HISTORICAL_DIR: &str = "data/historical/stdmet/";
const USER_AGENT: &str = "weather-buoys-rust/0.1";

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Download historical weather buoy datasets from NDBC"
)]
struct Args {
    #[arg(short, long, default_value_t = DEFAULT_START_YEAR)]
    start_year: i32,

    #[arg(short, long, default_value_t = DEFAULT_END_YEAR)]
    end_year: i32,

    #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
    buoys: Option<Vec<String>>,

    #[arg(short, long, default_value = "data")]
    output_dir: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let buoy_list = args
        .buoys
        .unwrap_or_else(|| DEFAULT_BUOYS.iter().map(|s| s.to_string()).collect());

    fs::create_dir_all(&args.output_dir)?;

    let client = Client::builder().user_agent(USER_AGENT).build()?;

    let total_tasks = (buoy_list.len() as u64) * ((args.end_year - args.start_year + 1) as u64);
    let pb = ProgressBar::new(total_tasks);
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} [{elapsed_precise}] [{pos}/{len}] {msg}")
            .unwrap(),
    );

    for buoy_id in &buoy_list {
        pb.set_message(format!("processing buoy: {buoy_id}"));
        download_buoy(
            &client,
            buoy_id,
            args.start_year,
            args.end_year,
            &args.output_dir,
            &pb,
        )?;
    }

    pb.finish_with_message("all buoys downloaded successfully");

    Ok(())
}

fn download_buoy(
    client: &Client,
    buoy_id: &str,
    start_year: i32,
    end_year: i32,
    output_dir: &str,
    pb: &ProgressBar,
) -> Result<(), Box<dyn Error>> {
    let output_path = format!("{output_dir}/buoy_{buoy_id}.csv");
    let mut output = File::create(&output_path)?;

    for year in start_year..=end_year {
        pb.set_message(format!("buoy {buoy_id} - downloading {year}"));

        let url = format!(
            "{}?filename={}h{}.txt.gz&dir={}",
            HISTORICAL_BASE_URL, buoy_id, year, HISTORICAL_DIR
        );

        let response = client.get(&url).send()?.error_for_status()?;
        let bytes = response.bytes()?;

        // Check if the response is actually gzipped, or served as plain text
        let reader: Box<dyn BufRead> = if bytes.len() >= 2 && bytes[0] == 0x1f && bytes[1] == 0x8b {
            let decoder = GzDecoder::new(&bytes[..]);
            Box::new(BufReader::new(decoder))
        } else {
            Box::new(BufReader::new(&bytes[..]))
        };

        let mut column_indices: Option<HashMap<String, usize>> = None;

        for line in reader.lines() {
            let line = line?;

            // Detect and parse the header row dynamically
            if line.starts_with('#') {
                if let Some(parsed_map) = parse_header(&line) {
                    // Ensure we are grabbing the name row, not the units row
                    if parsed_map.contains_key("WSPD") || parsed_map.contains_key("SPD") {
                        column_indices = Some(parsed_map);
                    }
                }
                continue;
            }

            if let Some(ref indices) = column_indices
                && let Some(record) = parse_record(&line, indices)
            {
                writeln!(output, "{record}")?;
            }
        }

        pb.inc(1);
    }

    Ok(())
}

fn parse_header(line: &str) -> Option<HashMap<String, usize>> {
    let cleaned = line.trim_start_matches('#');
    let fields: Vec<&str> = cleaned.split_whitespace().collect();
    if fields.is_empty() {
        return None;
    }

    let mut map = HashMap::new();
    for (index, field) in fields.into_iter().enumerate() {
        map.insert(field.to_uppercase(), index);
    }
    Some(map)
}

fn parse_record(line: &str, indices: &HashMap<String, usize>) -> Option<String> {
    let fields: Vec<&str> = line.split_whitespace().collect();

    // We need at least the basic time columns to form a timestamp
    if fields.len() < 5 {
        return None;
    }

    let year: i32 = fields[0].parse().ok()?;
    let month: u32 = fields[1].parse().ok()?;
    let day: u32 = fields[2].parse().ok()?;
    let hour: u32 = fields[3].parse().ok()?;

    // Sometimes 'mm' (minute) is omitted or non-numeric in very old formats
    let minute: u32 = if fields[4].chars().all(char::is_numeric) {
        fields[4].parse().unwrap_or(0)
    } else {
        0
    };

    if !(1..=12).contains(&month) || !(1..=31).contains(&day) || hour > 23 || minute > 59 {
        return None;
    }

    let timestamp = format!("{year:04}-{month:02}-{day:02}_{hour:02}:{minute:02}:00");

    // Dynamically look up positions based on NOAA's official headers
    let wspd_idx = indices.get("WSPD").or_else(|| indices.get("SPD")).copied();
    let pres_idx = indices.get("PRES").or_else(|| indices.get("BAR")).copied();
    let atmp_idx = indices.get("ATMP").copied();
    let wtmp_idx = indices.get("WTMP").copied();
    let dewp_idx = indices.get("DEWP").copied();
    let wvht_idx = indices.get("WVHT").copied();
    let apd_idx = indices.get("APD").copied();

    // Safely extract values. If a column doesn't exist, we insert f32::NAN
    let wspd = wspd_idx.and_then(|i| fields.get(i)).map_or(f32::NAN, |&v| parse_value(v, 99.0));
    let pres = pres_idx.and_then(|i| fields.get(i)).map_or(f32::NAN, |&v| parse_value(v, 9999.0));
    let atmp = atmp_idx.and_then(|i| fields.get(i)).map_or(f32::NAN, |&v| parse_value(v, 999.0));
    let dewp = dewp_idx.and_then(|i| fields.get(i)).map_or(f32::NAN, |&v| parse_value(v, 999.0));
    let wtmp = wtmp_idx.and_then(|i| fields.get(i)).map_or(f32::NAN, |&v| parse_value(v, 999.0));
    let wvht = wvht_idx.and_then(|i| fields.get(i)).map_or(f32::NAN, |&v| parse_value(v, 90.0));
    let apd = apd_idx.and_then(|i| fields.get(i)).map_or(f32::NAN, |&v| parse_value(v, 90.0));

    Some(format!(
        "{timestamp},{wspd:.1},{pres:.1},{atmp:.1},{dewp:.1},{wtmp:.1},{wvht:.2},{apd:.2}"
    ))
}

fn parse_value(value: &str, invalid_threshold: f32) -> f32 {
    let parsed = match value.parse::<f32>() {
        Ok(value) => value,
        Err(_) => return f32::NAN,
    };

    if parsed >= invalid_threshold {
        f32::NAN
    } else {
        parsed
    }
}
