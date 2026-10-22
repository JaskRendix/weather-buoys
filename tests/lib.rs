use weather_buoys::{MetricStats, StationStats, print_station_stats};

fn dummy_metric(max: f32, mean: f32) -> MetricStats {
    MetricStats { max, mean }
}

fn station(id: &str, max_val: f32, mean_val: f32) -> StationStats {
    StationStats {
        id: id.to_owned(),
        wind_speed: dummy_metric(max_val, mean_val),
        pressure: dummy_metric(max_val, mean_val),
        air_temp: dummy_metric(max_val, mean_val),
        dew_point: dummy_metric(max_val, mean_val),
        water_temp: dummy_metric(max_val, mean_val),
        wave_height: dummy_metric(max_val, mean_val),
        wave_period: dummy_metric(max_val, mean_val),
    }
}

#[test]
fn prints_stats_for_multiple_stations() {
    let results = vec![
        station("42001", 12.5, 6.0),
        station("42002", 35.2, 14.5),
        station("42003", 22.1, 9.2),
    ];

    print_station_stats(&results);
}

#[test]
fn handles_infinity() {
    let results = vec![
        station("finite", 10.0, 2.0),
        station("positive-infinity", f32::INFINITY, f32::INFINITY),
        station("negative-infinity", f32::NEG_INFINITY, f32::NEG_INFINITY),
    ];

    print_station_stats(&results);
}

#[test]
fn handles_nan_without_panicking() {
    let results = vec![
        station("normal", 10.0, 5.0),
        station("nan", f32::NAN, f32::NAN),
    ];

    // total_cmp provides a total ordering, so NaN should not cause a panic.
    print_station_stats(&results);
}

#[test]
fn does_not_require_station_ids_to_be_unique_or_non_empty() {
    let results = vec![
        station("", 1.0, 1.0),
        station(" ", 2.0, 2.0),
        station("station\nwith\nnewlines", 3.0, 3.0),
    ];

    print_station_stats(&results);
}

#[test]
fn handles_very_large_and_small_finite_values() {
    let results = vec![
        station("small", f32::MIN, f32::MIN),
        station("large", f32::MAX, f32::MAX),
        station("zero", 0.0, 0.0),
    ];

    print_station_stats(&results);
}
