#!/usr/bin/env python3
"""Normalize the original Palmer Penguins data without dropping observations."""

import argparse
import re
from pathlib import Path

import numpy as np
import pandas as pd


def snake_case(value: str) -> str:
    """Convert a source column label to a compact snake-case name."""
    value = value.replace("(o/oo)", "permil").replace("(mm)", "mm").replace("(g)", "g")
    return re.sub(r"_+", "_", re.sub(r"[^a-z0-9]+", "_", value.lower())).strip("_")


def prepare(input_path: Path) -> pd.DataFrame:
    """Read and normalize the source data while retaining all source rows."""
    frame = pd.read_csv(input_path, na_values=[""], keep_default_na=True)
    frame.columns = [snake_case(column) for column in frame.columns]
    frame = frame.rename(
        columns={
            "delta_15_n_permil": "delta_15n",
            "delta_13_c_permil": "delta_13c",
            "date_egg": "date_egg",
        }
    )
    frame["species"] = frame["species"].str.extract(r"^(Adelie|Chinstrap|Gentoo)")
    frame["sex"] = frame["sex"].replace({".": pd.NA}).str.title()
    frame["date_egg"] = pd.to_datetime(frame["date_egg"], errors="coerce")
    # Patsy/statsmodels does not accept pandas' nullable Int64 dtype. A float
    # retains missing years while remaining formula-compatible.
    frame["year"] = frame["date_egg"].dt.year.astype(float)
    frame["row_id"] = (
        frame["species"].str.lower().str[:3]
        + "-"
        + frame["sample_number"].astype("Int64").astype(str).str.zfill(3)
    )

    numeric = [
        "culmen_length_mm",
        "culmen_depth_mm",
        "flipper_length_mm",
        "body_mass_g",
        "delta_15n",
        "delta_13c",
    ]
    frame[numeric] = frame[numeric].apply(pd.to_numeric, errors="coerce")
    frame["clutch_complete"] = frame["clutch_completion"].eq("Yes")
    frame["morphology_complete"] = (
        frame[["culmen_length_mm", "culmen_depth_mm", "flipper_length_mm"]].notna().all(axis=1)
    )
    frame["isotope_complete"] = frame[["delta_13c", "delta_15n"]].notna().all(axis=1)
    frame["quality_issue"] = np.select(
        [
            frame["sex"].isna(),
            ~frame["morphology_complete"],
            ~frame["isotope_complete"],
        ],
        ["missing_sex", "missing_morphology", "missing_isotope"],
        default="none",
    )
    columns = ["row_id", *[column for column in frame.columns if column != "row_id"]]
    return frame[columns]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    result = prepare(args.input)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    result.to_csv(args.output, index=False)
    print(f"Wrote {len(result)} normalized observations to {args.output}")


if __name__ == "__main__":
    main()
