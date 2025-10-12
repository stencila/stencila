#!/usr/bin/env Rscript

# Test script for visual verification of Stencila theme variables
# This script generates various plots with different themes to verify theming works correctly

library(jsonlite)

# Source the theme_ function
source("src/theme.r")

# Function to set a theme from Stencila plotting variables
set_theme <- function(variables) {
  # Convert R list to JSON
  json <- toJSON(variables, auto_unbox = TRUE)

  # Apply theme
  theme_(json)
}

# Function to create various plots and save them to a folder
create_plots <- function(output_dir) {
  # Create output directory
  dir.create(output_dir, recursive = TRUE, showWarnings = FALSE)

  # Get background color and font size from theme options
  bg <- getOption("stencila.plot.background", "white")
  ps <- getOption("stencila.plot.font.size", 12)

  # Generate sample data
  set.seed(42)
  x <- 1:50
  y1 <- cumsum(rnorm(50, mean = 0.5, sd = 2))
  y2 <- cumsum(rnorm(50, mean = -0.3, sd = 1.5))
  categories <- c("A", "B", "C", "D", "E")
  cat_values <- c(23, 45, 12, 67, 34)

  # --- BASE R PLOTS ---

  # 1. Scatter plot
  png_file <- file.path(output_dir, "01_base_scatter.png")
  png(png_file, width = 9, height = 6, units = "in", res = 100, bg = bg)
  plot(x, y1,
       main = "Base R: Scatter Plot",
       xlab = "X Axis",
       ylab = "Y Axis",
       pch = 19)
  dev.off()
  
  # 2. Line plot
  png_file <- file.path(output_dir, "02_base_line.png")
  png(png_file, width = 9, height = 6, units = "in", res = 100, bg = bg)
  plot(x, y1, type = "l",
       main = "Base R: Line Plot",
       ylim = c(-5, 30),
       xlab = "X Axis",
       ylab = "Y Axis",
       col = 1)
  lines(x, y2, col = 2)
  legend("topright", legend = c("Series 1", "Series 2"),
         col = 1:2, lty = 1)
  dev.off()

  # 3. Bar plot
  png_file <- file.path(output_dir, "03_base_bar.png")
  png(png_file, width = 9, height = 6, units = "in", res = 100, bg = bg)
  barplot(cat_values,
          names.arg = categories,
          main = "Base R: Bar Plot",
          xlab = "Category",
          ylab = "Value",
          col = 1:5)
  dev.off()

  # 4. Histogram
  png_file <- file.path(output_dir, "04_base_histogram.png")
  png(png_file, width = 9, height = 6, units = "in", res = 100, bg = bg)
  hist(rnorm(1000),
       main = "Base R: Histogram",
       xlab = "Value",
       ylab = "Frequency",
       col = 3,
       breaks = 30)
  dev.off()

  # 5. Box plot
  png_file <- file.path(output_dir, "05_base_boxplot.png")
  png(png_file, width = 9, height = 6, units = "in", res = 100, bg = bg)
  data_list <- list(
    A = rnorm(100, mean = 5),
    B = rnorm(100, mean = 7),
    C = rnorm(100, mean = 6),
    D = rnorm(100, mean = 8)
  )
  boxplot(data_list,
          main = "Base R: Box Plot",
          xlab = "Group",
          ylab = "Value",
          col = 1:4)
  dev.off()

  # 6. Multiple panels
  png_file <- file.path(output_dir, "06_base_panels.png")
  png(png_file, width = 9, height = 6, units = "in", res = 100, bg = bg, pointsize = ps)
  par(mfrow = c(2, 2))
  plot(x, y1, type = "p", main = "Points", col = 1, pch = 19)
  plot(x, y1, type = "l", main = "Line", col = 2)
  plot(x, y1, type = "b", main = "Both", col = 3, pch = 19)
  barplot(cat_values[1:4], main = "Bars", col = 1:4)
  par(mfrow = c(1, 1))
  dev.off()

  # --- GGPLOT2 PLOTS (if available) ---

  if (requireNamespace("ggplot2", quietly = TRUE)) {
    library(ggplot2)

    # Create data frames for ggplot2
    df1 <- data.frame(x = x, y1 = y1, y2 = y2)
    df2 <- data.frame(category = categories, value = cat_values)
    df3 <- data.frame(value = rnorm(1000))
    df4 <- data.frame(
      group = rep(c("A", "B", "C", "D"), each = 100),
      value = c(rnorm(100, 5), rnorm(100, 7), rnorm(100, 6), rnorm(100, 8))
    )

    # 7. ggplot2 Scatter
    png_file <- file.path(output_dir, "07_ggplot_scatter.png")
    p <- ggplot(df1, aes(x = x, y = y1)) +
      geom_point() +
      labs(title = "ggplot2: Scatter Plot",
           x = "X Axis",
           y = "Y Axis")
    ggsave(png_file, p, width = 9, height = 6, dpi = 100)

    # 8. ggplot2 Line
    png_file <- file.path(output_dir, "08_ggplot_line.png")
    df_long <- data.frame(
      x = rep(x, 2),
      y = c(y1, y2),
      series = rep(c("Series 1", "Series 2"), each = length(x))
    )
    p <- ggplot(df_long, aes(x = x, y = y, color = series)) +
      geom_line() +
      labs(title = "ggplot2: Line Plot",
           x = "X Axis",
           y = "Y Axis",
           color = "Series")
    ggsave(png_file, p, width = 9, height = 6, dpi = 100)

    # 9. ggplot2 Bar
    png_file <- file.path(output_dir, "09_ggplot_bar.png")
    p <- ggplot(df2, aes(x = category, y = value, fill = category)) +
      geom_bar(stat = "identity") +
      labs(title = "ggplot2: Bar Plot",
           x = "Category",
           y = "Value")
    ggsave(png_file, p, width = 9, height = 6, dpi = 100)

    # 10. ggplot2 Histogram
    png_file <- file.path(output_dir, "10_ggplot_histogram.png")
    p <- ggplot(df3, aes(x = value)) +
      geom_histogram(bins = 30) +
      labs(title = "ggplot2: Histogram",
           x = "Value",
           y = "Count")
    ggsave(png_file, p, width = 9, height = 6, dpi = 100)

    # 11. ggplot2 Boxplot
    png_file <- file.path(output_dir, "11_ggplot_boxplot.png")
    p <- ggplot(df4, aes(x = group, y = value, fill = group)) +
      geom_boxplot() +
      labs(title = "ggplot2: Box Plot",
           x = "Group",
           y = "Value")
    ggsave(png_file, p, width = 9, height = 6, dpi = 100)

    # 12. ggplot2 Faceted plot
    png_file <- file.path(output_dir, "12_ggplot_facet.png")
    p <- ggplot(df4, aes(x = value)) +
      geom_histogram(bins = 20) +
      facet_wrap(~ group, ncol = 2) +
      labs(title = "ggplot2: Faceted Histogram",
           x = "Value",
           y = "Count")
    ggsave(png_file, p, width = 9, height = 6, dpi = 100)

    # 13. ggplot2 with subtitle and caption
    png_file <- file.path(output_dir, "13_ggplot_annotated.png")
    p <- ggplot(df1, aes(x = x, y = y1)) +
      geom_line() +
      geom_point() +
      labs(title = "ggplot2: Annotated Plot",
           subtitle = "This is a subtitle to test subtitle styling",
           caption = "This is a caption to test caption styling",
           x = "X Axis",
           y = "Y Axis")
    ggsave(png_file, p, width = 9, height = 6, dpi = 100)
  } else {
    cat("ggplot2 not available, skipping ggplot2 plots\n")
  }
}

# Three contrasting themes that we can compare to check that theme variables
# are being applied properly. No variable should be the same in all three themes.
themes <- list(
  # Monochrome: light background, grays, sans fonts
  # Uses axis-specific grid widths to demonstrate the feature
  "monochrome" = list(
    "plot-background" = "#ffffff",
    "plot-panel" = "#f5f5f5",
    "plot-padding-top" = 2,
    "plot-padding-right" = 2,
    "plot-padding-bottom" = 2,
    "plot-padding-left" = 2,
    "plot-axis-line-color" = "#404040",
    "plot-axis-line-width" = 0.5,
    "plot-axis-title-color" = "#404040",
    "plot-axis-title-size" = 11,
    "plot-color-1" = "#2c2c2c",
    "plot-color-2" = "#474747",
    "plot-color-3" = "#5c5c5c",
    "plot-color-4" = "#717171",
    "plot-color-5" = "#858585",
    "plot-color-6" = "#9a9a9a",
    "plot-color-7" = "#adadad",
    "plot-color-8" = "#c2c2c2",
    "plot-color-9" = "#d6d6d6",
    "plot-color-10" = "#e0e0e0",
    "plot-color-11" = "#ebebeb",
    "plot-color-12" = "#f0f0f0",
    "plot-font-family" = "Liberation Sans",
    "plot-font-size" = 10,
    "plot-grid-color" = "#aaaaaa",
    "plot-grid-x-width" = 0,  # No vertical bars
    "plot-grid-y-width" = 1,  # Only horizontal
    "plot-legend-background" = "#ffffff",
    "plot-legend-border-color" = "#808080",
    "plot-legend-border-width" = 1,
    "plot-legend-text-color" = "#404040",
    "plot-legend-position" = "right",
    "plot-line-width" = 1.5,
    "plot-point-size" = 6,
    "plot-text-color" = "#303030",
    "plot-title-size" = 14,
    "plot-subtitle-size" = 11
  ),

  # Vintage: parchment background, muted colors, serif fonts
  # Uses axis-specific grid widths with opposite emphasis from monochrome
  "vintage" = list(
    "plot-background" = "#f1d8b8ff",
    "plot-panel" = "#f8efe0",
    "plot-padding-top" = 10,
    "plot-padding-right" = 20,
    "plot-padding-bottom" = 30,
    "plot-padding-left" = 40,
    "plot-axis-line-color" = "#5a4a3a",
    "plot-axis-line-width" = 1.5,
    "plot-axis-title-color" = "#5a4a3a",
    "plot-axis-title-size" = 12,
    "plot-color-1" = "#8b4513",
    "plot-color-2" = "#a0522d",
    "plot-color-3" = "#cd853f",
    "plot-color-4" = "#d2691e",
    "plot-color-5" = "#bc8f8f",
    "plot-color-6" = "#8b7355",
    "plot-color-7" = "#6b8e23",
    "plot-color-8" = "#808000",
    "plot-color-9" = "#9acd32",
    "plot-color-10" = "#b8860b",
    "plot-color-11" = "#daa520",
    "plot-color-12" = "#cd5c5c",
    "plot-font-family" = "Liberation Serif",
    "plot-font-size" = 11,
    "plot-grid-color" = "#a75809ff",
    "plot-grid-x-width" = 1.2,  # Thicker vertical grid lines
    "plot-grid-y-width" = 0.4,  # Thinner horizontal grid lines
    "plot-legend-background" = "#f8efe0",
    "plot-legend-border-color" = "#8b7355",
    "plot-legend-border-width" = 1.5,
    "plot-legend-text-color" = "#5a4a3a",
    "plot-legend-position" = "bottom",
    "plot-line-width" = 2,
    "plot-point-size" = 8,
    "plot-text-color" = "#4a3a2a",
    "plot-title-size" = 16,
    "plot-subtitle-size" = 12
  ),

  # Cyberpunk: dark background, bright colors, monospace fonts
  "cyberpunk" = list(
    "plot-background" = "#0a0e27",
    "plot-panel" = "#151933",
    "plot-padding-top" = 20,
    "plot-padding-right" = 20,
    "plot-padding-bottom" = 40,
    "plot-padding-left" = 40,
    "plot-axis-line-color" = "#00ffff",
    "plot-axis-line-width" = 2,
    "plot-axis-title-color" = "#00ffff",
    "plot-axis-title-size" = 10,
    "plot-color-1" = "#00ffff",
    "plot-color-2" = "#ff00ff",
    "plot-color-3" = "#ff0080",
    "plot-color-4" = "#00ff00",
    "plot-color-5" = "#0080ff",
    "plot-color-6" = "#ffff00",
    "plot-color-7" = "#ff8000",
    "plot-color-8" = "#8000ff",
    "plot-color-9" = "#ff0040",
    "plot-color-10" = "#00ff80",
    "plot-color-11" = "#80ff00",
    "plot-color-12" = "#ff00c0",
    "plot-font-family" = "Liberation Mono",
    "plot-font-size" = 12,
    "plot-grid-color" = "#178a1bff",
    "plot-grid-width" = 1,
    "plot-legend-background" = "#151933",
    "plot-legend-border-color" = "#ff00ff",
    "plot-legend-border-width" = 2,
    "plot-legend-text-color" = "#00ffff",
    "plot-legend-position" = "top",
    "plot-line-width" = 2.5,
    "plot-point-size" = 10,
    "plot-text-color" = "#00ffff",
    "plot-title-size" = 18,
    "plot-subtitle-size" = 16
  )
)

# Run tests for each theme
run_tests <- function() {
  for (theme in names(themes)) {
    cat("Generating plots for theme: ", theme, "\n")
    set_theme(themes[[theme]])
    create_plots(file.path("test-themes", theme))
  }
}

# Run the tests
if (!interactive()) {
  run_tests()
}
