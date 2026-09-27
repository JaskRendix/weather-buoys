#!/usr/bin/env python3

import argparse
import csv
from datetime import datetime
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import tomllib

CONFIG_FILE = Path(__file__).resolve().parent / "config.toml"

# Mapping metrics to their CSV column index and display units/labels
METRICS = {
    "wind_speed": {"col": 1, "label": "Wind speed [m/s]", "ylim": (0, 20)},
    "pressure": {"col": 2, "label": "Sea Level Pressure [hPa]", "ylim": (950, 1050)},
    "air_temp": {"col": 3, "label": "Air Temperature [°C]", "ylim": (-10, 40)},
    "dew_point": {"col": 4, "label": "Dew Point [°C]", "ylim": (-10, 35)},
    "water_temp": {"col": 5, "label": "Water Temperature [°C]", "ylim": (0, 35)},
    "wave_height": {"col": 6, "label": "Wave Height [m]", "ylim": (0, 12)},
    "wave_period": {"col": 7, "label": "Wave Period [s]", "ylim": (0, 25)},
}


def load_config():
    with open(CONFIG_FILE, "rb") as f:
        return tomllib.load(f)


def read_metric_data(path, start_time, end_time, col_index):
    times = []
    values = []

    if not path.exists():
        raise FileNotFoundError(
            f"Data file not found: {path}. Run 'cargo run --bin get_buoy' first."
        )

    with path.open(newline="", encoding="utf-8") as file:
        reader = csv.reader(file)

        for row in reader:
            if len(row) <= col_index:
                continue

            try:
                timestamp = datetime.strptime(row[0].strip(), "%Y-%m-%d_%H:%M:%S")
                val = float(row[col_index].strip())
            except ValueError:
                continue

            if start_time <= timestamp <= end_time:
                times.append(timestamp)
                values.append(val)

    return times, values


def main():
    config = load_config()
    defaults = config["defaults"]

    parser = argparse.ArgumentParser(
        description="Plot a specific meteorological metric for a weather buoy."
    )
    parser.add_argument(
        "-b", "--buoy", default=defaults["buoy"], help="Buoy ID to plot"
    )
    parser.add_argument(
        "-m",
        "--metric",
        default="wind_speed",
        choices=list(METRICS.keys()),
        help="Metric to plot",
    )
    parser.add_argument(
        "-s",
        "--start",
        default=defaults["start_time"],
        help="Start time (YYYY-MM-DDTHH:MM:SS)",
    )
    parser.add_argument(
        "-e",
        "--end",
        default=defaults["end_time"],
        help="End time (YYYY-MM-DDTHH:MM:SS)",
    )
    args = parser.parse_args()

    buoy_id = args.buoy
    metric_key = args.metric
    metric_info = METRICS[metric_key]

    start_time = datetime.fromisoformat(args.start)
    end_time = datetime.fromisoformat(args.end)

    data_file = Path(__file__).resolve().parent.parent / "data" / f"buoy_{buoy_id}.csv"
    output_file = Path(__file__).resolve().parent / f"{metric_key}_{buoy_id}.png"

    times, values = read_metric_data(
        data_file, start_time, end_time, metric_info["col"]
    )

    fig, ax = plt.subplots(figsize=(12, 6))
    ax.plot(times, values, "k-", linewidth=0.8)

    ax.set_xlim(start_time, end_time)
    ax.set_ylim(metric_info["ylim"])
    ax.tick_params(axis="both", labelsize=14)

    ax.set_title(
        f"Measured {metric_key.replace('_', ' ')} at buoy {buoy_id}", fontsize=16
    )
    ax.set_ylabel(metric_info["label"], fontsize=16)
    ax.set_xlabel("Time", fontsize=16)
    ax.grid(True)

    fig.autofmt_xdate()
    fig.tight_layout()
    fig.savefig(output_file, dpi=150)
    plt.close(fig)

    print(f"Saved {metric_key} plot to {output_file}")


if __name__ == "__main__":
    main()
