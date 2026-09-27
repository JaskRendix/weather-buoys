use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use weather_buoys::io::read_buoy;

static TEST_ID: AtomicUsize = AtomicUsize::new(0);

fn temporary_csv_path(test_name: &str) -> PathBuf {
    let id = TEST_ID.fetch_add(1, Ordering::Relaxed);

    std::env::temp_dir().join(format!(
        "weather_buoys_{test_name}_{}_{}.csv",
        std::process::id(),
        id
    ))
}

fn write_csv(test_name: &str, contents: &str) -> PathBuf {
    let path = temporary_csv_path(test_name);

    fs::write(&path, contents).expect("failed to create temporary CSV file");

    path
}

fn remove_file(path: &PathBuf) {
    fs::remove_file(path).expect("failed to remove temporary CSV file");
}

#[test]
fn read_buoy_reads_all_eight_fields() {
    let path = write_csv(
        "reads_valid_data",
        "2024-01-01T00:00:00Z,5.5,1013.2,25.0,20.0,22.0,1.5,5.0\n\
         2024-01-01T01:00:00Z,8.25,1012.0,24.5,19.5,21.5,1.6,5.2\n",
    );

    let result = read_buoy(&path).expect("valid CSV should be read successfully");

    assert_eq!(
        result.time,
        vec![
            "2024-01-01T00:00:00Z".to_string(),
            "2024-01-01T01:00:00Z".to_string(),
        ]
    );
    assert_eq!(result.wind_speed, vec![5.5, 8.25]);
    assert_eq!(result.pressure, vec![1013.2, 1012.0]);
    assert_eq!(result.air_temp, vec![25.0, 24.5]);
    assert_eq!(result.dew_point, vec![20.0, 19.5]);
    assert_eq!(result.water_temp, vec![22.0, 21.5]);
    assert_eq!(result.wave_height, vec![1.5, 1.6]);
    assert_eq!(result.wave_period, vec![5.0, 5.2]);

    remove_file(&path);
}

#[test]
fn read_buoy_trims_whitespace_around_fields() {
    let path = write_csv(
        "trims_fields",
        " 2024-01-01T00:00:00Z , 5.5 , 1013.2 , 25.0 , 20.0 , 22.0 , 1.5 , 5.0 \n",
    );

    let result = read_buoy(&path).expect("CSV with surrounding whitespace should be valid");

    assert_eq!(result.time, vec!["2024-01-01T00:00:00Z".to_string()]);
    assert_eq!(result.wind_speed, vec![5.5]);
    assert_eq!(result.pressure, vec![1013.2]);

    remove_file(&path);
}

#[test]
fn read_buoy_accepts_quoted_csv_fields() {
    let path = write_csv(
        "quoted_fields",
        "\"2024-01-01 00:00:00\",5.5,1013.2,25.0,20.0,22.0,1.5,5.0\n",
    );

    let result = read_buoy(&path).expect("quoted CSV fields should be read successfully");

    assert_eq!(result.time, vec!["2024-01-01 00:00:00".to_string()]);
    assert_eq!(result.wind_speed, vec![5.5]);

    remove_file(&path);
}

#[test]
fn read_buoy_ignores_records_with_fewer_than_eight_fields() {
    let path = write_csv(
        "short_records",
        "2024-01-01T00:00:00Z,5.5,1013.2,25.0,20.0,22.0,1.5,5.0\n\
         incomplete,record\n\
         \n\
         2024-01-01T01:00:00Z,8.0,1012.0,24.5,19.5,21.5,1.6,5.2\n",
    );

    let result = read_buoy(&path).expect("short records should be skipped");

    assert_eq!(
        result.time,
        vec![
            "2024-01-01T00:00:00Z".to_string(),
            "2024-01-01T01:00:00Z".to_string(),
        ]
    );
    assert_eq!(result.wind_speed, vec![5.5, 8.0]);

    remove_file(&path);
}

#[test]
fn read_buoy_ignores_extra_fields_after_wave_period() {
    let path = write_csv(
        "extra_fields",
        "2024-01-01T00:00:00Z,5.5,1013.2,25.0,20.0,22.0,1.5,5.0,extra,ignored\n",
    );

    let result = read_buoy(&path).expect("records with extra fields should be valid");

    assert_eq!(result.time, vec!["2024-01-01T00:00:00Z".to_string()]);
    assert_eq!(result.wind_speed, vec![5.5]);
    assert_eq!(result.wave_period, vec![5.0]);

    remove_file(&path);
}

#[test]
fn read_buoy_returns_empty_vectors_for_empty_file() {
    let path = write_csv("empty_file", "");

    let result = read_buoy(&path).expect("an empty CSV should be valid");

    assert!(result.time.is_empty());
    assert!(result.wind_speed.is_empty());
    assert!(result.pressure.is_empty());
    assert!(result.air_temp.is_empty());

    remove_file(&path);
}

#[test]
fn read_buoy_returns_error_for_invalid_field_type() {
    let path = write_csv(
        "invalid_field",
        "2024-01-01T00:00:00Z,not-a-number,1013.2,25.0,20.0,22.0,1.5,5.0\n",
    );

    let result = read_buoy(&path);

    assert!(result.is_err());

    remove_file(&path);
}

#[test]
fn read_buoy_returns_error_for_empty_field() {
    let path = write_csv(
        "empty_field",
        "2024-01-01T00:00:00Z,,1013.2,25.0,20.0,22.0,1.5,5.0\n",
    );

    let result = read_buoy(&path);

    assert!(result.is_err());

    remove_file(&path);
}

#[test]
fn read_buoy_accepts_special_f32_values() {
    let path = write_csv("special_float_values", "a,inf,-inf,NaN,inf,-inf,NaN,inf\n");

    let result = read_buoy(&path).expect("special f32 values should parse");

    assert_eq!(result.time, vec!["a".to_string()]);
    assert_eq!(result.wind_speed[0], f32::INFINITY);
    assert_eq!(result.pressure[0], f32::NEG_INFINITY);
    assert!(result.air_temp[0].is_nan());

    remove_file(&path);
}

#[test]
fn read_buoy_returns_error_for_missing_file() {
    let path = temporary_csv_path("missing_file");

    let result = read_buoy(&path);

    assert!(result.is_err());
}

#[test]
fn read_buoy_accepts_string_paths() {
    let path = write_csv(
        "string_path",
        "2024-01-01T00:00:00Z,4.25,1013.2,25.0,20.0,22.0,1.5,5.0\n",
    );

    let path_as_string = path.to_string_lossy().into_owned();
    let result = read_buoy(path_as_string).expect("String paths should be accepted");

    assert_eq!(result.time, vec!["2024-01-01T00:00:00Z".to_string()]);
    assert_eq!(result.wind_speed, vec![4.25]);

    remove_file(&path);
}

fn write_bytes(test_name: &str, contents: &[u8]) -> PathBuf {
    let path = temporary_csv_path(test_name);

    fs::write(&path, contents).expect("failed to create temporary CSV file");

    path
}

#[test]
fn read_buoy_returns_error_for_invalid_utf8() {
    let path = write_bytes(
        "invalid_utf8",
        b"2024-01-01T00:00:00Z,\xff,1013.2,25.0,20.0,22.0,1.5,5.0\n",
    );

    let result = read_buoy(&path);

    assert!(result.is_err());

    remove_file(&path);
}
