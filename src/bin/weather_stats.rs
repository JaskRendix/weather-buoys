use clap::{Parser, ValueEnum};
use std::error::Error;

use weather_buoys::arrays::{denan, mean};
use weather_buoys::io::read_buoy;
use weather_buoys::{MetricStats, StationStats, print_station_stats};

const DEFAULT_STATIONS: [&str; 9] = [
    "42001", "42002", "42003", "42020", "42035", "42036", "42039", "42040", "42055",
];

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Parser, Debug)]
#[command(author, version, about = "Process all weather buoy metrics serially")]
struct Args {
    #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
    stations: Option<Vec<String>>,

    #[arg(short, long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,
}

fn compute_metric(raw: &[f32], id: &str, name: &str) -> Result<MetricStats, String> {
    let cleaned = denan(raw);
    if cleaned.is_empty() {
        return Err(format!("No valid values found for {name} at station {id}"));
    }
    let max = cleaned.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mean = mean(&cleaned)
        .ok_or_else(|| format!("Could not calculate mean for {name} at station {id}"))?;
    Ok(MetricStats { max, mean })
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let station_ids: Vec<String> = args
        .stations
        .unwrap_or_else(|| DEFAULT_STATIONS.iter().map(|s| s.to_string()).collect());

    let mut station_stats = Vec::with_capacity(station_ids.len());

    for id in station_ids {
        let filename = format!("data/buoy_{id}.csv");

        let data =
            read_buoy(&filename).map_err(|error| format!("Could not read {filename}: {error}"))?;

        station_stats.push(StationStats {
            id: id.clone(),
            wind_speed: compute_metric(&data.wind_speed, &id, "wind speed")?,
            pressure: compute_metric(&data.pressure, &id, "pressure")?,
            air_temp: compute_metric(&data.air_temp, &id, "air temperature")?,
            dew_point: compute_metric(&data.dew_point, &id, "dew point")?,
            water_temp: compute_metric(&data.water_temp, &id, "water temperature")?,
            wave_height: compute_metric(&data.wave_height, &id, "wave height")?,
            wave_period: compute_metric(&data.wave_period, &id, "wave period")?,
        });
    }

    match args.format {
        OutputFormat::Text => print_station_stats(&station_stats),
        OutputFormat::Json => {
            let json_output = serde_json::to_string_pretty(&station_stats)?;
            println!("{json_output}");
        }
    }

    Ok(())
}
