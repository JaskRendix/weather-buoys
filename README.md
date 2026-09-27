# weather-buoys

Processing weather buoy data in parallel using Rust. This codebase was ported from Fortran to Rust (originally based on the [modern-fortran/weather-buoys](https://github.com/modern-fortran/weather-buoys) repository).

## Getting Started

### Downloading Data

Download the historical dataset for the default buoys (2005–2017):

```bash
cargo run --bin get_buoy
```

You can customize the year range, specify particular stations, or change the target output directory via command-line arguments:

```bash
cargo run --bin get_buoy -- --start-year 2010 --end-year 2015 --buoys 42001 42002 --output-dir custom_data
```

### Running the Serial Program

Process all default stations serially and display extreme and mean statistics across all 7 meteorological metrics (wind speed, sea level pressure, air temperature, dew point, water temperature, wave height, and wave period):

```bash
cargo run --bin weather_stats
```

You can pass custom station IDs directly, or output the results as pretty-printed JSON for scripting and integrations:

```bash
cargo run --bin weather_stats -- --stations 42001 42002 --format json
```

Expected text output format:

```text
--- Wind Speed ---
  Max: 40.9 m/s at station 42001
  Highest Mean: 6.4071636 m/s at station 42002
  Lowest Mean: 5.739427 m/s at station 42001
--- Sea Level Pressure ---
  Max: 1040.7 hPa at station 42002
  ...
```

### Running the Parallel Program

Process the stations in parallel utilizing Rayon for maximum performance:

```bash
cargo run --bin weather_stats_parallel
```

Just like the serial version, custom stations and JSON formatting can be queried via arguments:

```bash
cargo run --bin weather_stats_parallel -- --stations 42001 42002 42003 --format json
```

## Plotting and Visualization

To generate visual maps and time-series plots using the included Python scripts, set up a virtual environment and install the required dependencies:

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install -r plotting/requirements.txt
```

### Plotting Buoy Maps

Generate a map displaying the geographic distribution of the weather buoy stations across the Gulf of Mexico:

```bash
python plotting/plot_map.py
```

### Plotting Meteorological Time-Series Metrics

You can plot any of the supported parameters (such as `wind_speed`, `pressure`, `air_temp`, `dew_point`, `water_temp`, `wave_height`, or `wave_period`) for any downloaded station:

```bash
# Plot pressure for buoy 42001
python plotting/plot_metric.py --buoy 42001 --metric pressure

# Plot air temperature 
python plotting/plot_metric.py --metric air_temp

# Plot wave height
python plotting/plot_metric.py --metric wave_height
```
