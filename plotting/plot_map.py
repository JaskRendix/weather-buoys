#!/usr/bin/env python3

import argparse
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import cartopy.crs as ccrs
import cartopy.feature as cfeature
import matplotlib.pyplot as plt
import tomllib

CONFIG_FILE = Path(__file__).resolve().parent / "config.toml"


def load_config():
    with open(CONFIG_FILE, "rb") as f:
        return tomllib.load(f)


def main():
    parser = argparse.ArgumentParser(description="Plot NOAA buoy stations map.")
    parser.add_argument(
        "-c", "--config", type=Path, default=CONFIG_FILE, help="Path to config file"
    )
    args = parser.parse_args()

    config = load_config()
    map_cfg = config["map"]
    stations = config["stations"]

    fig = plt.figure(figsize=(10, 7))
    ax = plt.axes(projection=ccrs.PlateCarree())

    ax.set_extent(
        [map_cfg["min_lon"], map_cfg["max_lon"], map_cfg["min_lat"], map_cfg["max_lat"]]
    )
    ax.add_feature(cfeature.LAND, color="lightgray")
    ax.add_feature(cfeature.OCEAN, color="lightblue")
    ax.add_feature(cfeature.COASTLINE, linewidth=0.6)
    ax.add_feature(cfeature.BORDERS, linewidth=0.5)

    for buoy_id, coords in stations.items():
        longitude, latitude = coords[0], coords[1]
        ax.plot(longitude, latitude, "r.", markersize=12, transform=ccrs.PlateCarree())
        ax.text(
            longitude + 0.2,
            latitude + 0.2,
            buoy_id,
            color="red",
            transform=ccrs.PlateCarree(),
        )

    ax.set_title(map_cfg["title"])
    fig.tight_layout()

    output_path = Path(__file__).resolve().parent / map_cfg["output"]
    fig.savefig(output_path, dpi=150)
    plt.close(fig)
    print(f"Saved map to {output_path}")


if __name__ == "__main__":
    main()
