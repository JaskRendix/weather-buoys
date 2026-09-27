pub mod arrays;
pub mod io;
pub mod parallel;

#[derive(Debug, serde::Serialize)]
pub struct MetricStats {
    pub max: f32,
    pub mean: f32,
}

#[derive(Debug, serde::Serialize)]
pub struct StationStats {
    pub id: String,
    pub wind_speed: MetricStats,
    pub pressure: MetricStats,
    pub air_temp: MetricStats,
    pub dew_point: MetricStats,
    pub water_temp: MetricStats,
    pub wave_height: MetricStats,
    pub wave_period: MetricStats,
}

pub fn print_station_stats(results: &[StationStats]) {
    macro_rules! print_extrema {
        ($name:expr, $unit:expr, $accessor:ident) => {
            let max_st = results
                .iter()
                .max_by(|a, b| a.$accessor.max.total_cmp(&b.$accessor.max))
                .unwrap();
            let min_mean_st = results
                .iter()
                .min_by(|a, b| a.$accessor.mean.total_cmp(&b.$accessor.mean))
                .unwrap();
            let max_mean_st = results
                .iter()
                .max_by(|a, b| a.$accessor.mean.total_cmp(&b.$accessor.mean))
                .unwrap();

            println!("--- {} ---", $name);
            println!(
                "  Max: {} {} at station {}",
                max_st.$accessor.max, $unit, max_st.id
            );
            println!(
                "  Highest Mean: {} {} at station {}",
                max_mean_st.$accessor.mean, $unit, max_mean_st.id
            );
            println!(
                "  Lowest Mean: {} {} at station {}",
                min_mean_st.$accessor.mean, $unit, min_mean_st.id
            );
        };
    }

    print_extrema!("Wind Speed", "m/s", wind_speed);
    print_extrema!("Sea Level Pressure", "hPa", pressure);
    print_extrema!("Air Temperature", "°C", air_temp);
    print_extrema!("Dew Point", "°C", dew_point);
    print_extrema!("Water Temperature", "°C", water_temp);
    print_extrema!("Wave Height", "m", wave_height);
    print_extrema!("Wave Period", "s", wave_period);
}
