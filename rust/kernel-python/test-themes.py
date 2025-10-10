#!/usr/bin/env python3

"""
Test script for visual verification of Stencila theme variables

This script generates various plots with different themes to verify theming works correctly
"""

import json
import os
from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np

# Source the theme function
import sys
sys.path.insert(0, str(Path(__file__).parent / "src"))
from theme import theme


def set_theme(variables):
    """Set a theme from Stencila plotting variables"""
    # Convert dict to JSON
    json_str = json.dumps(variables)
    # Apply theme
    theme(json_str)


def create_plots(output_dir):
    """Create various plots and save them to a folder"""
    # Create output directory
    Path(output_dir).mkdir(parents=True, exist_ok=True)

    # Get background color from rcParams
    bg = plt.rcParams.get("figure.facecolor", "white")

    # Generate sample data
    np.random.seed(42)
    x = np.arange(1, 51)
    y1 = np.cumsum(np.random.normal(0.5, 2, 50))
    y2 = np.cumsum(np.random.normal(-0.3, 1.5, 50))
    categories = ["A", "B", "C", "D", "E"]
    cat_values = [23, 45, 12, 67, 34]

    # --- MATPLOTLIB PLOTS ---

    # 1. Scatter plot
    fig, ax = plt.subplots(figsize=(9, 6))
    ax.scatter(x, y1, s=plt.rcParams["lines.markersize"]**2)
    ax.set_title("Matplotlib: Scatter Plot")
    ax.set_xlabel("X Axis")
    ax.set_ylabel("Y Axis")
    fig.savefig(os.path.join(output_dir, "01_matplotlib_scatter.png"))
    plt.close(fig)

    # 2. Line plot
    fig, ax = plt.subplots(figsize=(9, 6))
    ax.plot(x, y1, label="Series 1")
    ax.plot(x, y2, label="Series 2")
    ax.set_title("Matplotlib: Line Plot")
    ax.set_xlabel("X Axis")
    ax.set_ylabel("Y Axis")
    ax.set_ylim(-5, 30)
    ax.legend()
    fig.savefig(os.path.join(output_dir, "02_matplotlib_line.png"))
    plt.close(fig)

    # 3. Bar plot
    fig, ax = plt.subplots(figsize=(9, 6))
    bars = ax.bar(categories, cat_values)
    # Color each bar with sequential colors from the color cycle
    colors = [f"C{i}" for i in range(len(categories))]
    for bar, color in zip(bars, colors):
        bar.set_color(color)
    # Add legend matching ggplot2 behavior
    from matplotlib.patches import Patch
    legend_elements = [Patch(facecolor=f"C{i}", label=cat) for i, cat in enumerate(categories)]
    ax.legend(handles=legend_elements, title="category")
    ax.set_title("Matplotlib: Bar Plot")
    ax.set_xlabel("Category")
    ax.set_ylabel("Value")
    fig.savefig(os.path.join(output_dir, "03_matplotlib_bar.png"))
    plt.close(fig)

    # 4. Histogram
    fig, ax = plt.subplots(figsize=(9, 6))
    ax.hist(np.random.normal(0, 1, 1000), bins=30, color="C2")
    ax.set_title("Matplotlib: Histogram")
    ax.set_xlabel("Value")
    ax.set_ylabel("Frequency")
    fig.savefig(os.path.join(output_dir, "04_matplotlib_histogram.png"))
    plt.close(fig)

    # 5. Box plot
    fig, ax = plt.subplots(figsize=(9, 6))
    data_list = [
        np.random.normal(5, 1, 100),
        np.random.normal(7, 1, 100),
        np.random.normal(6, 1, 100),
        np.random.normal(8, 1, 100),
    ]
    bp = ax.boxplot(data_list, tick_labels=["A", "B", "C", "D"], patch_artist=True)
    # Color each box with sequential colors
    for patch, i in zip(bp["boxes"], range(len(data_list))):
        patch.set_facecolor(f"C{i}")
    ax.set_title("Matplotlib: Box Plot")
    ax.set_xlabel("Group")
    ax.set_ylabel("Value")
    fig.savefig(os.path.join(output_dir, "05_matplotlib_boxplot.png"))
    plt.close(fig)

    # 6. Multiple panels
    fig, ((ax1, ax2), (ax3, ax4)) = plt.subplots(2, 2, figsize=(9, 6))
    ax1.scatter(x, y1, s=plt.rcParams["lines.markersize"]**2, color="C0")
    ax1.set_title("Points")
    ax2.plot(x, y1, color="C1")
    ax2.set_title("Line")
    ax3.plot(x, y1, marker="o", color="C2")
    ax3.set_title("Both")
    bars = ax4.bar(categories[:4], cat_values[:4])
    for bar, i in zip(bars, range(4)):
        bar.set_color(f"C{i}")
    ax4.set_title("Bars")
    fig.tight_layout()
    fig.savefig(os.path.join(output_dir, "06_matplotlib_panels.png"))
    plt.close(fig)

    # 7. Annotated plot with subtitle and caption
    fig, ax = plt.subplots(figsize=(9, 6))
    ax.plot(x, y1, marker="o")
    ax.set_title("Annotated Plot")
    ax.set_xlabel("X Axis")
    ax.set_ylabel("Y Axis")
    # Use suptitle for subtitle effect (positioned below title)
    fig.suptitle("Matplotlib: Annotated Plot", y=0.98, fontsize=plt.rcParams["axes.titlesize"])
    ax.set_title("This is a subtitle to test subtitle styling", fontsize=plt.rcParams.get("axes.titlesize", 10) * 0.8, pad=10)
    # Add caption as text annotation at bottom
    fig.text(0.5, 0.02, "This is a caption to test caption styling",
             ha="center", fontsize=plt.rcParams["font.size"] * 0.9,
             color=plt.rcParams["text.color"])
    fig.savefig(os.path.join(output_dir, "07_matplotlib_annotated.png"), bbox_inches="tight")
    plt.close(fig)


# Three contrasting themes that we can compare to check that theme variables
# are being applied properly. No variable should be the same in all three themes.
themes = {
    # Monochrome: light background, grays, sans fonts
    # Uses axis-specific grid widths to demonstrate the feature
    "monochrome": {
        "plot-background": "#ffffff",
        "plot-padding-top": 4,
        "plot-padding-right": 4,
        "plot-padding-bottom": 8,
        "plot-padding-left": 8,
        "plot-axis-line-color": "#404040",
        "plot-axis-line-width": 0.5,
        "plot-axis-title-color": "#404040",
        "plot-axis-title-size": 11,
        "plot-color-1": "#2c2c2c",
        "plot-color-2": "#474747",
        "plot-color-3": "#5c5c5c",
        "plot-color-4": "#717171",
        "plot-color-5": "#858585",
        "plot-color-6": "#9a9a9a",
        "plot-color-7": "#adadad",
        "plot-color-8": "#c2c2c2",
        "plot-color-9": "#d6d6d6",
        "plot-color-10": "#e0e0e0",
        "plot-color-11": "#ebebeb",
        "plot-color-12": "#f0f0f0",
        "plot-font-family": "Liberation Sans",
        "plot-font-size": 10,
        "plot-grid-color": "#aaaaaa",
        "plot-grid-x-width": 0,  # No vertical bars
        "plot-grid-y-width": 1,  # Only horizontal
        "plot-legend-background": "#ffffff",
        "plot-legend-border-color": "#808080",
        "plot-legend-border-width": 1,
        "plot-legend-text-color": "#404040",
        "plot-legend-size": 9,
        "plot-legend-position": "right",
        "plot-line-cap": "round",
        "plot-line-join": "round",
        "plot-line-width": 1.5,
        "plot-panel-border-color": "#808080",
        "plot-panel-border-width": 1,
        "plot-point-size": 6,
        "plot-text-color": "#303030",
        "plot-tick-color": "#404040",
        "plot-tick-size": 4,
        "plot-tick-width": 1,
        "plot-title-size": 14,
        "plot-subtitle-size": 11,
    },
    # Vintage: parchment background, muted colors, serif fonts
    # Uses axis-specific grid widths with opposite emphasis from monochrome
    "vintage": {
        "plot-background": "#f1d8b8ff",
        "plot-padding-top": 12,
        "plot-padding-right": 12,
        "plot-padding-bottom": 24,
        "plot-padding-left": 24,
        "plot-axis-line-color": "#5a4a3a",
        "plot-axis-line-width": 1.5,
        "plot-axis-title-color": "#5a4a3a",
        "plot-axis-title-size": 12,
        "plot-color-1": "#8b4513",
        "plot-color-2": "#a0522d",
        "plot-color-3": "#cd853f",
        "plot-color-4": "#d2691e",
        "plot-color-5": "#bc8f8f",
        "plot-color-6": "#8b7355",
        "plot-color-7": "#6b8e23",
        "plot-color-8": "#808000",
        "plot-color-9": "#9acd32",
        "plot-color-10": "#b8860b",
        "plot-color-11": "#daa520",
        "plot-color-12": "#cd5c5c",
        "plot-font-family": "Liberation Serif",
        "plot-font-size": 11,
        "plot-grid-color": "#a75809ff",
        "plot-grid-x-width": 1.2,  # Thicker vertical grid lines
        "plot-grid-y-width": 0.4,  # Thinner horizontal grid lines
        "plot-legend-background": "#f8efe0",
        "plot-legend-border-color": "#8b7355",
        "plot-legend-border-width": 1.5,
        "plot-legend-text-color": "#5a4a3a",
        "plot-legend-size": 10,
        "plot-legend-position": "bottom",
        "plot-line-cap": "round",
        "plot-line-join": "round",
        "plot-line-width": 2,
        "plot-panel-border-color": "#8b7355",
        "plot-panel-border-width": 1.5,
        "plot-point-size": 8,
        "plot-text-color": "#4a3a2a",
        "plot-tick-color": "#5a4a3a",
        "plot-tick-size": 8,
        "plot-tick-width": 1.5,
        "plot-title-size": 16,
        "plot-subtitle-size": 12,
    },
    # Cyberpunk: dark background, bright colors, monospace fonts
    "cyberpunk": {
        "plot-background": "#0a0e27",
        "plot-padding-top": 20,
        "plot-padding-right": 20,
        "plot-padding-bottom": 40,
        "plot-padding-left": 40,
        "plot-axis-line-color": "#00ffff",
        "plot-axis-line-width": 2,
        "plot-axis-title-color": "#00ffff",
        "plot-axis-title-size": 10,
        "plot-color-1": "#00ffff",
        "plot-color-2": "#ff00ff",
        "plot-color-3": "#ff0080",
        "plot-color-4": "#00ff00",
        "plot-color-5": "#0080ff",
        "plot-color-6": "#ffff00",
        "plot-color-7": "#ff8000",
        "plot-color-8": "#8000ff",
        "plot-color-9": "#ff0040",
        "plot-color-10": "#00ff80",
        "plot-color-11": "#80ff00",
        "plot-color-12": "#ff00c0",
        "plot-font-family": "Liberation Mono",
        "plot-font-size": 12,
        "plot-grid-color": "#178a1bff",
        "plot-grid-width": 1,
        "plot-legend-background": "#151933",
        "plot-legend-border-color": "#ff00ff",
        "plot-legend-border-width": 2,
        "plot-legend-text-color": "#00ffff",
        "plot-legend-size": 8,
        "plot-legend-position": "top",
        "plot-line-cap": "butt",
        "plot-line-join": "bevel",
        "plot-line-width": 2.5,
        "plot-panel-border-color": "#ff00ff",
        "plot-panel-border-width": 2,
        "plot-point-size": 10,
        "plot-text-color": "#00ffff",
        "plot-tick-color": "#00ffff",
        "plot-tick-size": 3,
        "plot-tick-width": 2,
        "plot-title-size": 18,
        "plot-subtitle-size": 16,
    },
}


def run_tests():
    """Run tests for each theme"""
    for theme_name in themes:
        print(f"Generating plots for theme: {theme_name}")
        set_theme(themes[theme_name])
        create_plots(os.path.join("test-themes", theme_name))


# Run the tests
if __name__ == "__main__":
    run_tests()
