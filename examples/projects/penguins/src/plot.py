#!/usr/bin/env python3
"""Create static publication figures from committed analysis tables."""

import argparse
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import pandas as pd
import seaborn as sns

LABELS = {
    "culmen_length_mm": "Culmen length (mm)",
    "culmen_depth_mm": "Culmen depth (mm)",
    "flipper_length_mm": "Flipper length (mm)",
    "body_mass_g": "Body mass (g)",
    "size_score": "Structural size score",
    "delta_13c": "δ¹³C",
    "delta_15n": "δ¹⁵N",
}


def dimorphism(frame: pd.DataFrame) -> plt.Figure:
    data = frame.loc[frame["metric"].eq("size_score")].copy()
    figure, axis = plt.subplots(figsize=(7.2, 4.4))
    sns.pointplot(
        data=data, x="species", y="mean", hue="sex", dodge=0.28, markers=["o", "s"], ax=axis
    )
    for offset, sex in zip([-0.14, 0.14], ["Female", "Male"], strict=True):
        subset = data.loc[data["sex"].eq(sex)].sort_values("species")
        x = range(len(subset))
        axis.errorbar(
            [value + offset for value in x],
            subset["mean"],
            yerr=subset["std_error"],
            fmt="none",
            color="black",
            capsize=3,
        )
    axis.set(xlabel=None, ylabel="Mean structural-size PC1 (± SE)")
    axis.legend(title="Sex", frameon=False)
    sns.despine()
    return figure


def isotopes(frame: pd.DataFrame) -> plt.Figure:
    data = frame.loc[frame["record_type"].eq("candidate")].copy()
    order = ["null", "sex", "size", "year", "sex_year", "size_year", "sex_x_year", "size_x_year"]
    figure, axes = plt.subplots(
        2, 3, figsize=(12, 7), sharex=True, sharey=True, constrained_layout=True
    )
    for axis, ((species, isotope), group) in zip(
        axes.flat, data.groupby(["species", "isotope"], sort=True), strict=True
    ):
        values = group.set_index("model")["weight"].reindex(order)
        sns.barplot(x=values.index, y=values.values, color="#247ba0", ax=axis)
        axis.set_title(f"{species}: {LABELS[isotope]}")
        axis.set(xlabel=None, ylabel="Akaike weight", ylim=(0, 1))
        axis.tick_params(axis="x", rotation=55)
    return figure


def sensitivity(frame: pd.DataFrame) -> plt.Figure:
    data = frame.melt(
        id_vars=["universe", "species", "isotope"],
        value_vars=["sex_evidence_weight", "size_evidence_weight"],
        var_name="explanatory_term",
        value_name="cumulative_weight",
    )
    data["explanatory_term"] = data["explanatory_term"].map(
        {"sex_evidence_weight": "Sex", "size_evidence_weight": "Structural size"}
    )
    data["endpoint"] = data["species"] + " " + data["isotope"].map(LABELS)
    figure, axis = plt.subplots(figsize=(10, 6.2))
    sns.stripplot(
        data=data,
        x="cumulative_weight",
        y="endpoint",
        hue="explanatory_term",
        dodge=True,
        jitter=0.14,
        alpha=0.72,
        ax=axis,
    )
    axis.axvline(0.5, color="0.65", linestyle="--", linewidth=1)
    axis.set(xlabel="Cumulative model weight", ylabel=None, xlim=(-0.02, 1.02))
    axis.legend(title=None, frameon=False, loc="lower right")
    sns.despine()
    return figure


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=["dimorphism", "isotopes", "sensitivity"])
    parser.add_argument("--input", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    frame = pd.read_csv(args.input)
    figure = globals()[args.mode](frame)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    figure.savefig(
        args.output,
        dpi=220,
        bbox_inches="tight",
        metadata={"Creator": "Stencila Palmer Penguins ASTRA analysis"},
    )
    plt.close(figure)
    print(f"Wrote {args.output}")


if __name__ == "__main__":
    main()
