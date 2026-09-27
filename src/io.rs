use std::error::Error;
use std::fs::File;
use std::path::Path;

pub struct BuoyData {
    pub time: Vec<String>,
    pub wind_speed: Vec<f32>,
    pub pressure: Vec<f32>,
    pub air_temp: Vec<f32>,
    pub dew_point: Vec<f32>,
    pub water_temp: Vec<f32>,
    pub wave_height: Vec<f32>,
    pub wave_period: Vec<f32>,
}

pub fn read_buoy<P: AsRef<Path>>(filename: P) -> Result<BuoyData, Box<dyn Error>> {
    let file = File::open(filename)?;

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(file);

    let mut time = Vec::new();
    let mut wind_speed = Vec::new();
    let mut pressure = Vec::new();
    let mut air_temp = Vec::new();
    let mut dew_point = Vec::new();
    let mut water_temp = Vec::new();
    let mut wave_height = Vec::new();
    let mut wave_period = Vec::new();

    for result in reader.records() {
        let record = result?;

        if record.len() < 8 {
            continue;
        }

        time.push(record[0].trim().to_string());
        wind_speed.push(record[1].trim().parse()?);
        pressure.push(record[2].trim().parse()?);
        air_temp.push(record[3].trim().parse()?);
        dew_point.push(record[4].trim().parse()?);
        water_temp.push(record[5].trim().parse()?);
        wave_height.push(record[6].trim().parse()?);
        wave_period.push(record[7].trim().parse()?);
    }

    Ok(BuoyData {
        time,
        wind_speed,
        pressure,
        air_temp,
        dew_point,
        water_temp,
        wave_height,
        wave_period,
    })
}
