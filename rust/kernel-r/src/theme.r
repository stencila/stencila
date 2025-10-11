# Create a persistent environment to store theme parameters
# This allows theme settings to persist across graphics device creation
.stencila_theme_env <- new.env(parent = emptyenv())
.stencila_theme_env$par_params <- list()
.stencila_theme_env$color_palette <- NULL

# Hook function to apply stored theme parameters to new plots
# This is called automatically via setHook("plot.new", ...) whenever a new plot is created
.apply_theme_hook <- function() {
  if (length(.stencila_theme_env$par_params) > 0) {
    do.call(par, .stencila_theme_env$par_params)
  }
  if (!is.null(.stencila_theme_env$color_palette)) {
    palette(.stencila_theme_env$color_palette)
  }
}

# Apply the document theme to the kernel instance
#
# Applies Stencila `plot-*` theme variables to base plots (see
# https://stat.ethz.ch/R-manual/R-devel/library/graphics/html/par.html) and
# ggplots (see https://ggplot2.tidyverse.org/reference/theme.html)
#
# Color variables in the theme use hex colors and size values use numbers
# in pt units.
theme_ <- function(json) {
  variables <- jsonlite::fromJSON(json, simplifyVector = FALSE)

  # Helper to get variable value
  get_var <- function(name) {
    # TODO: Detect dark mode properly, for now just use light mode
    variables[[name]]
  }

  # Helper to parse numeric values from CSS units
  parse_number <- function(value) {
    if (is.null(value)) return(NULL)
    tryCatch(
      as.numeric(value),
      warning = function(w) NULL,
      error = function(e) NULL
    )
  }

  # Helper to convert points to millimeters for ggplot2 SIZE parameters
  # ggplot2's size (for text) uses TRUE millimeters
  # Conversion: 1 pt = 1/72 inch = 25.4/72 mm ≈ 0.35mm
  # ggplot2::.pt constant ≈ 2.845276 (72/25.4)
  pt_to_mm <- function(pt_value) {
    if (is.null(pt_value)) return(NULL)
    pt_value / ggplot2::.pt
  }

  # Helper to convert points to ggplot2 LINEWIDTH units
  # IMPORTANT: ggplot2's linewidth uses a DIFFERENT unit than size!
  # Due to historical reasons, 1 ggplot2 linewidth unit ≈ 0.75mm (not 1mm)
  # Conversion: 1 pt = 25.4/72 mm ≈ 0.353mm, then 0.353mm / 0.75mm/unit
  # Factor: 54/25.4 ≈ 2.126 (not 72/25.4 like for size!)
  pt_to_ggplot_linewidth <- function(pt_value) {
    if (is.null(pt_value)) return(NULL)
    pt_value / (54/25.4)
  }

  # Helper to parse CSS font stacks into a vector of font names
  parse_fonts <- function(value) {
    if (is.null(value)) return(NULL)
    # Split on commas and clean up each font name
    fonts <- strsplit(value, ",")[[1]]
    fonts <- trimws(fonts)
    fonts <- gsub("^['\"]|['\"]$", "", fonts)  # Remove quotes
    fonts[fonts != ""]
  }

  # Helper to map CSS font names to R's supported font families
  # R supports: "serif", "sans", "mono", "symbol"
  map_font_family <- function(fonts) {
    if (is.null(fonts) || length(fonts) == 0) return(NULL)

    # Check each font in order and return first match
    # IMPORTANT: Check "sans" before "serif" because "sans-serif" contains both
    for (font in fonts) {
      font_lower <- tolower(font)
      if (grepl("mono|courier|consolas|menlo|ubuntu mono|source code", font_lower)) {
        return("mono")
      } else if (grepl("sans|arial|helvetica|verdana|tahoma", font_lower)) {
        return("sans")
      } else if (grepl("serif|times|georgia|garamond", font_lower)) {
        return("serif")
      }
    }
    # Default to sans-serif
    return("sans")
  }

  # <NA> = No corresponding Stencila theme variable available yet

  # Collect parameters to set via par()
  params <- list()

  # IMPORTANT: Get base font size early for use in character expansion calculations.
  # In R's par() system, `ps` sets the base font size and `cex.*` parameters are
  # multipliers relative to `ps`. For correct sizing, we must divide desired sizes
  # by the base font size, not by arbitrary constants like 12.
  base_font_size <- parse_number(get_var("plot-font-size"))
  if (is.null(base_font_size)) {
    base_font_size <- 12  # Default fallback for calculations
  }

  # Background and foreground

  # Background color
  if (!is.null(bg <- get_var("plot-background"))) {
    params$bg <- bg
  }

  # Foreground color (used for axes, boxes, etc.)
  if (!is.null(color <- get_var("plot-axis-line-color"))) {
    params$fg <- color
  }

  # Color for axis tick labels
  if (!is.null(color <- get_var("plot-tick-color"))) {
    params$col.axis <- color
  }

  # Color for x and y labels
  if (!is.null(color <- get_var("plot-axis-title-color"))) {
    params$col.lab <- color
  }

  # Color for main title
  if (!is.null(color <- get_var("plot-text-color"))) {
    params$col.main <- color
  }

  # Color for sub-title
  if (!is.null(color <- get_var("plot-text-color"))) {
    params$col.sub <- color
  }

  # Line properties

  # Line width
  if (!is.null(width <- parse_number(get_var("plot-line-width")))) {
    params$lwd <- width
  }

  # Line type
  # params$lty <- NA  # plot-line-dash is "0" or "4 2", needs conversion to R line type

  # Line mitre limit
  # params$lmitre <- NA  # No corresponding variable

  # Line end style
  if (!is.null(cap <- get_var("plot-line-cap"))) {
    if (cap %in% c("round", "butt", "square")) {
      params$lend <- switch(cap,
        "round" = 1,
        "butt" = 0,
        "square" = 2
      )
    }
  }

  # Line join style
  if (!is.null(join <- get_var("plot-line-join"))) {
    if (join %in% c("round", "mitre", "bevel")) {
      params$ljoin <- switch(join,
        "round" = 1,
        "mitre" = 0,
        "bevel" = 2
      )
    }
  }

  # Point properties

  # Point character
  # params$pch <- NA  # No corresponding variable

  # Point size
  # Were were doing the following but cex scales text as well as points
  # so was creating tiny text.
  # if (!is.null(point_size <- parse_number(get_var("plot-point-size")))) {
  #   params$cex <- point_size / base_font_size
  # }

  # Text properties

  # Font family
  if (!is.null(family <- map_font_family(parse_fonts(get_var("plot-font-family"))))) {
    # R supports: "serif", "sans", "mono", "symbol"
    params$family <- family
  }

  # Font size (points)
  params$ps <- base_font_size

  # Character expansion for axis annotation
  # Previously divided by hardcoded 12, which caused incorrect sizing
  if (!is.null(size <- parse_number(get_var("plot-font-size")))) {
    params$cex.axis <- size / base_font_size
  }

  # Character expansion for x and y labels
  # If plot-axis-title-size=11pt and plot-font-size=10pt, then cex.lab should be 11/10=1.1
  if (!is.null(size <- parse_number(get_var("plot-axis-title-size")))) {
    params$cex.lab <- size / base_font_size
  }

  # Character expansion for main title
  # If plot-title-size=14pt and plot-font-size=10pt, then cex.main should be 14/10=1.4
  if (!is.null(size <- parse_number(get_var("plot-title-size")))) {
    params$cex.main <- size / base_font_size
  }

  # Character expansion for sub-title
  # If plot-subtitle-size=11pt and plot-font-size=10pt, then cex.sub should be 11/10=1.1
  if (!is.null(size <- parse_number(get_var("plot-subtitle-size")))) {
    params$cex.sub <- size / base_font_size
  }

  # Font face for axis annotation
  # params$font.axis <- NA  # No corresponding variable

  # Font face for x and y labels (axis titles)
  if (!is.null(weight <- get_var("plot-axis-title-weight"))) {
    # R font face codes: 1=plain, 2=bold, 3=italic, 4=bold italic
    params$font.lab <- if (weight == "bold" || weight >= 700) 2 else 1
  }

  # Font face for main title
  # params$font.main <- NA  # No corresponding variable (defaults to bold)

  # Font face for sub-title
  # params$font.sub <- NA  # No corresponding variable

  # Default font face
  # params$font <- NA  # No corresponding variable

  # Text justification
  # params$adj <- NA  # No corresponding variable

  # String rotation (degrees)
  # params$srt <- NA  # No corresponding variable

  # Character rotation (degrees)
  # params$crt <- NA  # No corresponding variable

  # Line height multiplier
  # params$lheight <- NA  # No corresponding variable

  # Axes and ticks

  # Tick mark length (negative values are outside, positive inside)
  # tcl is measured in "lines of text", which depends on the font size
  if (!is.null(tick_size <- parse_number(get_var("plot-tick-size")))) {
    params$tcl <- -(tick_size / (base_font_size * 1.2))
  }

  # Axis line width
  # Note: lwd.axis is not a standard par() parameter
  # if (!is.null(width <- parse_number(get_var("plot-axis-line-width")))) {
  #   params$lwd.axis <- width  # Not a standard par() parameter
  # }

  # Tick line width (not a standard par parameter, but used by axis())
  # params$lwd.ticks <- NA  # plot-tick-width exists but not a par() parameter

  # Tick mark length (alternative to tcl, in fraction of plot region)
  # params$tck <- NA  # No corresponding variable

  # Axis label orientation (0=parallel, 1=horizontal, 2=perpendicular, 3=vertical)
  # params$las <- NA  # No corresponding variable

  # Axis labeling style (c(x,y,len) where len is number of intervals)
  # params$lab <- NA  # No corresponding variable

  # Margin line for axis title, labels and line
  # params$mgp <- NA  # No corresponding variable

  # X-axis interpolation points (c(x1, x2, n))
  # params$xaxp <- NA  # No corresponding variable

  # Y-axis interpolation points (c(y1, y2, n))
  # params$yaxp <- NA  # No corresponding variable

  # X-axis calculation style ("r"=regular, "i"=internal, "d"=direct)
  # params$xaxs <- NA  # No corresponding variable

  # Y-axis calculation style ("r"=regular, "i"=internal, "d"=direct)
  # params$yaxs <- NA  # No corresponding variable

  # X-axis type ("s"=standard, "t"=time, "n"=none)
  # params$xaxt <- NA  # No corresponding variable

  # Y-axis type ("s"=standard, "t"=time, "n"=none)
  # params$yaxt <- NA  # No corresponding variable

  # Logarithmic x-axis
  # params$xlog <- NA  # No corresponding variable

  # Logarithmic y-axis
  # params$ylog <- NA  # No corresponding variable

  # Y-label bias in character-string labels
  # params$ylbias <- NA  # No corresponding variable

  # Box type around plot: "o"=box, "l"=L, "7"=top+right, "c"=C, "u"=U, "n"=none
  # Control box type based on plot-panel-border-width
  # When border width is 0 or null, use "l" (left and bottom only) to match ggplot2 behavior
  # where panel.border is disabled and axis.line is used
  panel_border_width <- parse_number(get_var("plot-panel-border-width"))
  if (is.null(panel_border_width) || panel_border_width == 0) {
    params$bty <- "l"
  } else {
    params$bty <- "o"
  }

  # Margins and layout

  # Margins (lines of text)
  # Set mar instead of omi to be consistent with ggplot and matplotlib where
  # padding represents total space (not additional space on top of defaults).
  # Convert plot-padding-* from pt to lines and use sensible minimums for axes/labels.
  top <- parse_number(get_var("plot-padding-top"))
  right <- parse_number(get_var("plot-padding-right"))
  bottom <- parse_number(get_var("plot-padding-bottom"))
  left <- parse_number(get_var("plot-padding-left"))

  if (!is.null(top) || !is.null(right) || !is.null(bottom) || !is.null(left)) {
    # Convert from pt to lines (1 line ≈ base_font_size * 1.2)
    # Line height multiplier of 1.2 is R's default
    pt_to_lines <- function(pt) {
      if (is.null(pt)) return(NULL)
      pt / (base_font_size * 1.2)
    }

    # Calculate themed padding in lines
    top_lines <- pt_to_lines(top)
    right_lines <- pt_to_lines(right)
    bottom_lines <- pt_to_lines(bottom)
    left_lines <- pt_to_lines(left)

    # Set minimum margins to ensure axes and labels fit
    # These are typical defaults that provide space for axis labels and titles
    min_bottom <- 4  # Space for x-axis labels and title
    min_left <- 4    # Space for y-axis labels and title
    min_top <- 2     # Space for main title
    min_right <- 1   # Minimal right margin

    # Add themed padding to minimums (consistent with ggplot/matplotlib where
    # padding is around the box containing axes/labels/titles)
    # mar order is c(bottom, left, top, right)
    params$mar <- c(
      min_bottom + (if (is.null(bottom_lines)) 0 else bottom_lines),
      min_left + (if (is.null(left_lines)) 0 else left_lines),
      min_top + (if (is.null(top_lines)) 0 else top_lines),
      min_right + (if (is.null(right_lines)) 0 else right_lines)
    )
  }

  # Margin expansion factor
  # params$mex <- NA  # No corresponding variable

  # Outer margin in normalized device coordinates
  # params$omd <- NA  # No corresponding variable

  # Multi-figure layout

  # Multi-figure rows and columns (filled by columns)
  # params$mfcol <- NA  # No corresponding variable

  # Multi-figure rows and columns (filled by rows)
  # params$mfrow <- NA  # No corresponding variable

  # Current figure number in multi-figure layout
  # params$mfg <- NA  # No corresponding variable

  # Plot region and dimensions

  # Figure region in normalized device coordinates (c(x1, x2, y1, y2))
  # params$fig <- NA  # No corresponding variable

  # Figure dimensions in inches (c(width, height))
  # params$fin <- NA  # No corresponding variable

  # Plot dimensions in inches (c(width, height))
  # params$pin <- NA  # No corresponding variable

  # Plot region coordinates as fraction of figure region (c(x1, x2, y1, y2))
  # params$plt <- NA  # No corresponding variable

  # Plot type: "s" for square, "m" for maximal
  # params$pty <- NA  # No corresponding variable

  # User coordinate system extremes (c(x1, x2, y1, y2))
  # params$usr <- NA  # No corresponding variable

  # Other graphical parameters

  # Clipping region: FALSE=plot, TRUE=figure, NA=device
  # params$xpd <- NA  # No corresponding variable

  # Annotations flag (title and axis labels)
  # params$ann <- NA  # No corresponding variable

  # Ask before starting new plot page
  # params$ask <- NA  # No corresponding variable

  # Add to current plot (TRUE) or start new plot (FALSE)
  # params$new <- NA  # No corresponding variable

  # Obsolete/deprecated parameters

  # Error bar type (obsolete)
  # params$err <- NA  # Obsolete parameter

  # Marker height (obsolete)
  # params$mkh <- NA  # Obsolete parameter

  # Smoothing parameter (obsolete)
  # params$smo <- NA  # Obsolete parameter

  # Legend (handled by legend() function, not par())
  # - plot-legend-background
  # - plot-legend-border-color, plot-legend-border-width
  # - plot-legend-text-color
  # - plot-legend-size

  # Grid (not a par() parameter, set via grid() function)
  # - plot-grid-color
  # - plot-grid-width
  # - plot-grid-dash

  # Additional theme variables without direct par() equivalents

  # Animation: plot-anim-duration, plot-anim-ease
  # Area opacity: plot-area-opacity
  # Bar properties: plot-bar-*, plot-bar-category-gap, plot-bar-gap, plot-bar-radius
  # Candle properties: plot-candle-*
  # Color palette (12 colors): plot-color-1 through plot-color-12
  # Contrast grid: plot-contrast-grid
  # Crosshair: plot-crosshair-*
  # Focus outline: plot-focus-outline-*
  # Gaps: plot-gap-x, plot-gap-y
  # Heatmap: plot-heatmap-*
  # Hover opacity: plot-hover-opacity
  # Mark opacity: plot-mark-opacity
  # Muted colors: plot-muted
  # Negative/positive colors: plot-negative, plot-positive
  # Panel: plot-panel, plot-panel-border-*
  # Ramp: plot-ramp-*
  # Selection: plot-selection-*
  # Stroke: plot-stroke-width
  # Tooltip: plot-tooltip-*
  # Warning colors: plot-warning
  # Zero line: plot-zero-line-*

  # Store parameters for use by the plot.new hook
  # Instead of applying them directly, we store them so they persist across device creation
  .stencila_theme_env$par_params <- params

  # Store background color in a global option so kernel.r can access it when creating PNG devices
  # This is necessary because PNG device background must be set at device creation time
  if (!is.null(bg <- get_var("plot-background"))) {
    options(stencila.plot.background = bg)
  } else {
    options(stencila.plot.background = "white")
  }

  # Store font size in a global option so it can be accessed when creating PNG devices
  # This is necessary because PNG device pointsize must be set at device creation time
  options(stencila.plot.font.size = base_font_size)

  # Store plot dimensions and DPI in global options
  # These values come from the theme as points and need to be converted to inches
  # for use with PNG devices (width/height with units="in")
  plot_width_pt <- parse_number(get_var("plot-width"))
  plot_height_pt <- parse_number(get_var("plot-height"))
  plot_dpi <- parse_number(get_var("plot-dpi"))

  if (!is.null(plot_width_pt)) {
    options(stencila.plot.width = plot_width_pt / 72)
  } else {
    options(stencila.plot.width = 8)  # Default 8 inches
  }

  if (!is.null(plot_height_pt)) {
    options(stencila.plot.height = plot_height_pt / 72)
  } else {
    options(stencila.plot.height = 6)  # Default 6 inches
  }

  if (!is.null(plot_dpi)) {
    options(stencila.plot.dpi = plot_dpi)
  } else {
    options(stencila.plot.dpi = 100)  # Default 100 DPI
  }

  # Set color palette using the 12 theme colors
  colors <- c()
  for (i in 1:12) {
    color <- get_var(paste0("plot-color-", i))
    if (!is.null(color)) {
      colors <- c(colors, color)
    }
  }
  # Store the color palette for use by the plot.new hook
  if (length(colors) > 0) {
    .stencila_theme_env$color_palette <- colors
  } else {
    .stencila_theme_env$color_palette <- NULL
  }

  # Register the hook to apply theme settings on each new plot
  # This ensures the settings persist across graphics device creation
  setHook("plot.new", .apply_theme_hook, action = "replace")

  ###########################################################################################

  if (!requireNamespace("ggplot2", quietly = TRUE)) {
    return(invisible(NULL))
  }

  # Build ggplot2 theme elements
  # IMPORTANT: ggplot2 uses different units than our theme system:
  # - Text sizes (element_text): Uses pt (points) - matches our theme system ✓
  # - Line widths (linewidth parameter): Uses mm (millimeters) - needs conversion!
  # - Unit lengths (unit()): Can specify units explicitly (e.g., unit(x, "pt"))
  #
  # To convert line widths from pt to mm, divide by ggplot2::.pt (≈ 2.845276)
  # Example: 1pt linewidth → 1 / ggplot2::.pt ≈ 0.35mm in ggplot2 units
  theme_elements <- list()

  # Plot-level

  # Plot background
  if (!is.null(bg <- get_var("plot-background"))) {
    theme_elements$plot.background <- ggplot2::element_rect(fill = bg, color = NA)
  }

  # Plot title
  plot_title_params <- list()
  if (!is.null(color <- get_var("plot-text-color"))) {
    plot_title_params$color <- color
  }
  if (!is.null(size <- parse_number(get_var("plot-title-size")))) {
    plot_title_params$size <- size
  }
  if (length(plot_title_params) > 0) {
    theme_elements$plot.title <- do.call(ggplot2::element_text, plot_title_params)
  }

  # Plot subtitle
  plot_subtitle_params <- list()
  if (!is.null(color <- get_var("plot-text-color"))) {
    plot_subtitle_params$color <- color
  }
  if (!is.null(size <- parse_number(get_var("plot-subtitle-size")))) {
    plot_subtitle_params$size <- size
  }
  if (length(plot_subtitle_params) > 0) {
    theme_elements$plot.subtitle <- do.call(ggplot2::element_text, plot_subtitle_params)
  }

  # Plot caption
  plot_caption_params <- list()
  if (!is.null(color <- get_var("plot-text-color"))) {
    plot_caption_params$color <- color
  }
  if (length(plot_caption_params) > 0) {
    theme_elements$plot.caption <- do.call(ggplot2::element_text, plot_caption_params)
  }

  # Plot margin
  top <- parse_number(get_var("plot-padding-top"))
  right <- parse_number(get_var("plot-padding-right"))
  bottom <- parse_number(get_var("plot-padding-bottom"))
  left <- parse_number(get_var("plot-padding-left"))

  if (!is.null(top) || !is.null(right) || !is.null(bottom) || !is.null(left)) {
    # Default to 0 for NULL values
    theme_elements$plot.margin <- ggplot2::margin(
      if (is.null(top)) 0 else top,
      if (is.null(right)) 0 else right,
      if (is.null(bottom)) 0 else bottom,
      if (is.null(left)) 0 else left,
      unit = "pt"
    )
  }

  # Plot tag (upper-left label)
  # theme_elements$plot.tag <- NA  # No corresponding variable

  # Plot title position
  # theme_elements$plot.title.position <- NA  # No corresponding variable

  # Plot caption position
  # theme_elements$plot.caption.position <- NA  # No corresponding variable

  # Plot tag position
  # theme_elements$plot.tag.position <- NA  # No corresponding variable

  # Panel

  # Panel background
  if (!is.null(bg <- get_var("plot-panel"))) {
    theme_elements$panel.background <- ggplot2::element_rect(fill = bg, color = NA)
  }

  # Panel border
  # Check for width = 0 case first to set element_blank()
  border_width <- parse_number(get_var("plot-panel-border-width"))
  if (!is.null(border_width) && border_width == 0) {
    theme_elements$panel.border <- ggplot2::element_blank()
  } else {
    # Build parameters for element_rect with available properties
    panel_border_params <- list(fill = NA)
    if (!is.null(color <- get_var("plot-axis-line-color"))) {
      panel_border_params$color <- color
    }
    if (!is.null(border_width) && border_width > 0) {
      panel_border_params$linewidth <- pt_to_ggplot_linewidth(border_width)
    }
    if (length(panel_border_params) > 1) {  # More than just fill = NA
      theme_elements$panel.border <- do.call(ggplot2::element_rect, panel_border_params)
    }
  }

  # Panel grid
  if (!is.null(color <- get_var("plot-grid-color"))) {
    # Get axis-specific widths (in pt), falling back to general plot-grid-width
    width_x <- parse_number(get_var("plot-grid-x-width"))
    width_y <- parse_number(get_var("plot-grid-y-width"))
    width_general <- parse_number(get_var("plot-grid-width"))

    # Use axis-specific widths if available, otherwise use general width
    if (is.null(width_x)) width_x <- width_general
    if (is.null(width_y)) width_y <- width_general

    # Set axis-specific major grids
    if (!is.null(width_x) && width_x > 0) {
      theme_elements$panel.grid.major.x <- ggplot2::element_line(color = color, linewidth = pt_to_ggplot_linewidth(width_x))
      theme_elements$panel.grid.minor.x <- ggplot2::element_line(color = color, linewidth = pt_to_ggplot_linewidth(width_x * 0.5))
    } else if (!is.null(width_x) && width_x == 0) {
      # Width of 0 means no grid lines
      theme_elements$panel.grid.major.x <- ggplot2::element_blank()
      theme_elements$panel.grid.minor.x <- ggplot2::element_blank()
    }

    if (!is.null(width_y) && width_y > 0) {
      theme_elements$panel.grid.major.y <- ggplot2::element_line(color = color, linewidth = pt_to_ggplot_linewidth(width_y))
      theme_elements$panel.grid.minor.y <- ggplot2::element_line(color = color, linewidth = pt_to_ggplot_linewidth(width_y * 0.5))
    } else if (!is.null(width_y) && width_y == 0) {
      # Width of 0 means no grid lines
      theme_elements$panel.grid.major.y <- ggplot2::element_blank()
      theme_elements$panel.grid.minor.y <- ggplot2::element_blank()
    }

    # Set general grid as fallback (used when axis-specific is not available)
    # This ensures backwards compatibility with themes that only use plot-grid-width
    if (!is.null(width_general) && width_general > 0) {
      if (is.null(width_x) && is.null(width_y)) {
        # If no axis-specific widths, set general grid
        theme_elements$panel.grid.major <- ggplot2::element_line(color = color, linewidth = pt_to_ggplot_linewidth(width_general))
        theme_elements$panel.grid.minor <- ggplot2::element_line(color = color, linewidth = pt_to_ggplot_linewidth(width_general * 0.5))
      }
    }
  }

  # Panel spacing (between facets)
  # theme_elements$panel.spacing <- NA  # plot-gap-x and plot-gap-y exist but need implementation
  # theme_elements$panel.spacing.x <- NA  # plot-gap-x exists but needs implementation
  # theme_elements$panel.spacing.y <- NA  # plot-gap-y exists but needs implementation

  # Panel ontop (panel on top of data)
  # theme_elements$panel.ontop <- NA  # No corresponding variable

  # Axes

  # Axis line - only set if panel borders are disabled
  # When panel.border is enabled, it draws all 4 sides uniformly, so axis.line would
  # cause visual inconsistency (double-drawing on left/bottom making them appear thicker)
  panel_border_width <- parse_number(get_var("plot-panel-border-width"))
  if (is.null(panel_border_width) || panel_border_width == 0) {
    axis_line_params <- list()
    if (!is.null(color <- get_var("plot-axis-line-color"))) {
      axis_line_params$color <- color
    }
    if (!is.null(width <- parse_number(get_var("plot-axis-line-width")))) {
      axis_line_params$linewidth <- pt_to_ggplot_linewidth(width)
    }
    if (length(axis_line_params) > 0) {
      theme_elements$axis.line <- do.call(ggplot2::element_line, axis_line_params)
    }
  }

  # Axis text (tick labels)
  axis_text_params <- list()
  if (!is.null(color <- get_var("plot-tick-color"))) {
    axis_text_params$color <- color
  }
  if (!is.null(size <- parse_number(get_var("plot-font-size")))) {
    axis_text_params$size <- size
  }
  if (length(axis_text_params) > 0) {
    theme_elements$axis.text <- do.call(ggplot2::element_text, axis_text_params)
    theme_elements$axis.text.x <- do.call(ggplot2::element_text, axis_text_params)
    theme_elements$axis.text.y <- do.call(ggplot2::element_text, axis_text_params)
  }

  # Axis title
  axis_title_params <- list()
  if (!is.null(color <- get_var("plot-axis-title-color"))) {
    axis_title_params$color <- color
  }
  if (!is.null(size <- parse_number(get_var("plot-axis-title-size")))) {
    axis_title_params$size <- size
  }
  if (!is.null(weight <- get_var("plot-axis-title-weight"))) {
    # Map CSS font-weight to ggplot2 face: normal→plain, bold/700→bold
    axis_title_params$face <- if (weight == "bold" || weight >= 700) "bold" else "plain"
  }
  if (length(axis_title_params) > 0) {
    theme_elements$axis.title <- do.call(ggplot2::element_text, axis_title_params)
    theme_elements$axis.title.x <- do.call(ggplot2::element_text, axis_title_params)
    theme_elements$axis.title.y <- do.call(ggplot2::element_text, axis_title_params)
  }

  # Axis ticks
  axis_ticks_params <- list()
  if (!is.null(color <- get_var("plot-tick-color"))) {
    axis_ticks_params$color <- color
  }
  if (!is.null(width <- parse_number(get_var("plot-tick-width")))) {
    axis_ticks_params$linewidth <- pt_to_ggplot_linewidth(width)
  }
  if (length(axis_ticks_params) > 0) {
    theme_elements$axis.ticks <- do.call(ggplot2::element_line, axis_ticks_params)
  }
  # Tick length is independent of tick line properties
  if (!is.null(tick_size <- parse_number(get_var("plot-tick-size")))) {
    # tick_size is already in pt, unit() handles the conversion
    theme_elements$axis.ticks.length <- ggplot2::unit(tick_size, "pt")
  }

  # Axis position-specific elements
  # theme_elements$axis.title.x.top <- NA  # No corresponding variable
  # theme_elements$axis.title.x.bottom <- NA  # No corresponding variable
  # theme_elements$axis.title.y.left <- NA  # No corresponding variable
  # theme_elements$axis.title.y.right <- NA  # No corresponding variable

  # theme_elements$axis.text.x.top <- NA  # No corresponding variable
  # theme_elements$axis.text.x.bottom <- NA  # No corresponding variable
  # theme_elements$axis.text.y.left <- NA  # No corresponding variable
  # theme_elements$axis.text.y.right <- NA  # No corresponding variable

  # Axis polar coordinate elements
  # theme_elements$axis.text.theta <- NA  # No corresponding variable
  # theme_elements$axis.text.r <- NA  # No corresponding variable
  # theme_elements$axis.ticks.theta <- NA  # No corresponding variable
  # theme_elements$axis.ticks.r <- NA  # No corresponding variable

  # Axis ticks position-specific
  # theme_elements$axis.ticks.x <- NA  # No corresponding variable
  # theme_elements$axis.ticks.y <- NA  # No corresponding variable
  # theme_elements$axis.ticks.x.top <- NA  # No corresponding variable
  # theme_elements$axis.ticks.x.bottom <- NA  # No corresponding variable
  # theme_elements$axis.ticks.y.left <- NA  # No corresponding variable
  # theme_elements$axis.ticks.y.right <- NA  # No corresponding variable

  # Axis minor ticks
  # theme_elements$axis.minor.ticks <- NA  # No corresponding variable
  # theme_elements$axis.minor.ticks.x <- NA  # No corresponding variable
  # theme_elements$axis.minor.ticks.y <- NA  # No corresponding variable
  # theme_elements$axis.minor.ticks.x.top <- NA  # No corresponding variable
  # theme_elements$axis.minor.ticks.x.bottom <- NA  # No corresponding variable
  # theme_elements$axis.minor.ticks.y.left <- NA  # No corresponding variable
  # theme_elements$axis.minor.ticks.y.right <- NA  # No corresponding variable
  # theme_elements$axis.minor.ticks.length <- NA  # No corresponding variable
  # theme_elements$axis.minor.ticks.length.x <- NA  # No corresponding variable
  # theme_elements$axis.minor.ticks.length.y <- NA  # No corresponding variable
  # theme_elements$axis.minor.ticks.length.x.top <- NA  # No corresponding variable
  # theme_elements$axis.minor.ticks.length.x.bottom <- NA  # No corresponding variable
  # theme_elements$axis.minor.ticks.length.y.left <- NA  # No corresponding variable
  # theme_elements$axis.minor.ticks.length.y.right <- NA  # No corresponding variable

  # Axis lines position-specific
  # theme_elements$axis.line.x <- NA  # No corresponding variable
  # theme_elements$axis.line.y <- NA  # No corresponding variable
  # theme_elements$axis.line.x.top <- NA  # No corresponding variable
  # theme_elements$axis.line.x.bottom <- NA  # No corresponding variable
  # theme_elements$axis.line.y.left <- NA  # No corresponding variable
  # theme_elements$axis.line.y.right <- NA  # No corresponding variable

  # Axis ticks length position-specific
  # theme_elements$axis.ticks.length.x <- NA  # No corresponding variable
  # theme_elements$axis.ticks.length.y <- NA  # No corresponding variable
  # theme_elements$axis.ticks.length.x.top <- NA  # No corresponding variable
  # theme_elements$axis.ticks.length.x.bottom <- NA  # No corresponding variable
  # theme_elements$axis.ticks.length.y.left <- NA  # No corresponding variable
  # theme_elements$axis.ticks.length.y.right <- NA  # No corresponding variable

  # Legend

  # Legend background and border
  legend_background_params <- list()
  if (!is.null(bg <- get_var("plot-legend-background"))) {
    legend_background_params$fill <- bg
  }
  if (!is.null(border_color <- get_var("plot-legend-border-color"))) {
    legend_background_params$color <- border_color
  } else {
    # If no border color specified, set to NA (no border)
    legend_background_params$color <- NA
  }
  if (!is.null(border_width <- parse_number(get_var("plot-legend-border-width")))) {
    if (border_width > 0) {
      legend_background_params$linewidth <- pt_to_ggplot_linewidth(border_width)
    }
  }
  if (length(legend_background_params) > 0) {
    theme_elements$legend.background <- do.call(ggplot2::element_rect, legend_background_params)
  }

  # Legend key (background underneath legend keys)
  if (!is.null(bg <- get_var("plot-legend-background"))) {
    theme_elements$legend.key <- ggplot2::element_rect(fill = bg, color = NA)
  }

  # Legend text
  # Use accumulator pattern to build element_text() parameters independently
  legend_text_params <- list()
  if (!is.null(color <- get_var("plot-legend-text-color"))) {
    legend_text_params$color <- color
  }
  if (!is.null(size <- parse_number(get_var("plot-legend-size")))) {
    legend_text_params$size <- size
  }
  if (length(legend_text_params) > 0) {
    theme_elements$legend.text <- do.call(ggplot2::element_text, legend_text_params)
  }

  # Legend title
  # Use accumulator pattern to build element_text() parameters independently
  legend_title_params <- list()
  if (!is.null(color <- get_var("plot-legend-text-color"))) {
    legend_title_params$color <- color
  }
  if (!is.null(size <- parse_number(get_var("plot-legend-size")))) {
    legend_title_params$size <- size
  }
  if (length(legend_title_params) > 0) {
    theme_elements$legend.title <- do.call(ggplot2::element_text, legend_title_params)
  }

  # Legend position
  if (!is.null(position <- get_var("plot-legend-position"))) {
    if (position == "auto") {
      theme_elements$legend.position <- "right"
    } else if (position %in% c("top", "bottom", "left", "right", "none")) {
      theme_elements$legend.position <- position
    }
  }

  # Legend spacing
  # theme_elements$legend.spacing <- NA  # plot-legend-gap exists but needs implementation
  # theme_elements$legend.spacing.x <- NA  # No corresponding variable
  # theme_elements$legend.spacing.y <- NA  # No corresponding variable

  # Legend key dimensions
  # theme_elements$legend.key.size <- NA  # plot-legend-marker-size exists but needs implementation
  # theme_elements$legend.key.width <- NA  # No corresponding variable
  # theme_elements$legend.key.height <- NA  # No corresponding variable
  # theme_elements$legend.key.spacing <- NA  # No corresponding variable
  # theme_elements$legend.key.spacing.x <- NA  # No corresponding variable
  # theme_elements$legend.key.spacing.y <- NA  # No corresponding variable

  # Legend layout and positioning
  # theme_elements$legend.direction <- NA  # No corresponding variable
  # theme_elements$legend.justification <- NA  # No corresponding variable
  # theme_elements$legend.margin <- NA  # No corresponding variable

  # Legend box (when multiple legends)
  # theme_elements$legend.box <- NA  # No corresponding variable
  # theme_elements$legend.box.just <- NA  # No corresponding variable
  # theme_elements$legend.box.margin <- NA  # No corresponding variable
  # theme_elements$legend.box.background <- NA  # No corresponding variable
  # theme_elements$legend.box.spacing <- NA  # No corresponding variable

  # Text (global text settings)

  # Global text color and size
  color <- get_var("plot-text-color")
  size <- parse_number(get_var("plot-font-size"))

  # For ggplot2, use the first specific font name from the CSS stack
  # ggplot2 can handle actual font names, not just generic families
  fonts <- parse_fonts(get_var("plot-font-family"))
  family <- NULL
  if (!is.null(fonts) && length(fonts) > 0) {
    # Find first specific font (not generic like "sans-serif")
    for (font in fonts) {
      font_lower <- tolower(font)
      if (!font_lower %in% c("sans-serif", "serif", "monospace", "cursive", "fantasy")) {
        family <- font
        break
      }
    }
    # If no specific font found, map generic families to R equivalents
    if (is.null(family)) {
      family <- map_font_family(fonts)
    }
  }

  # Build element_text with available parameters
  text_params <- list()
  if (!is.null(color)) text_params$color <- color
  if (!is.null(size)) text_params$size <- size
  if (!is.null(family)) text_params$family <- family

  if (length(text_params) > 0) {
    theme_elements$text <- do.call(ggplot2::element_text, text_params)
  }

  # Line (global line settings)

  if (!is.null(color <- get_var("plot-axis-line-color"))) {
    theme_elements$line <- ggplot2::element_line(color = color)
  }

  # Strip (facet labels)

  # Strip background
  if (!is.null(bg <- get_var("plot-background"))) {
    theme_elements$strip.background <- ggplot2::element_rect(fill = bg, color = NA)
  }

  # Strip text
  if (!is.null(color <- get_var("plot-text-color"))) {
    theme_elements$strip.text <- ggplot2::element_text(color = color)
  }

  # Strip axis-specific
  # theme_elements$strip.text.x <- NA  # No corresponding variable
  # theme_elements$strip.text.y <- NA  # No corresponding variable
  # theme_elements$strip.text.x.top <- NA  # No corresponding variable
  # theme_elements$strip.text.x.bottom <- NA  # No corresponding variable
  # theme_elements$strip.text.y.left <- NA  # No corresponding variable
  # theme_elements$strip.text.y.right <- NA  # No corresponding variable

  # Strip background axis-specific
  # theme_elements$strip.background.x <- NA  # No corresponding variable
  # theme_elements$strip.background.y <- NA  # No corresponding variable

  # Strip placement and clipping
  # theme_elements$strip.placement <- NA  # No corresponding variable
  # theme_elements$strip.switch.pad.grid <- NA  # No corresponding variable
  # theme_elements$strip.switch.pad.wrap <- NA  # No corresponding variable
  # theme_elements$strip.clip <- NA  # No corresponding variable

  # Global element types
  # theme_elements$rect <- NA  # No corresponding variable (affects all rectangles)
  # theme_elements$title <- NA  # No corresponding variable (affects all titles)

  # Additional theme variables without direct ggplot2 equivalents

  # Animation: plot-anim-duration, plot-anim-ease
  # Area opacity: plot-area-opacity
  # Bar properties: plot-bar-*, plot-bar-category-gap, plot-bar-gap, plot-bar-radius
  # Candle properties: plot-candle-*
  # Contrast grid: plot-contrast-grid
  # Crosshair: plot-crosshair-*
  # Focus outline: plot-focus-outline-*
  # Gaps: plot-gap-x, plot-gap-y (could map to facet spacing)
  # Heatmap: plot-heatmap-*
  # Hover opacity: plot-hover-opacity
  # Mark opacity: plot-mark-opacity
  # Muted colors: plot-muted
  # Negative/positive colors: plot-negative, plot-positive
  # Panel: plot-panel (handled above)
  # Ramp: plot-ramp-*
  # Selection: plot-selection-*
  # Stroke: plot-stroke-width
  # Tooltip: plot-tooltip-*
  # Warning colors: plot-warning
  # Zero line: plot-zero-line-*

  # Apply ggplot2 theme if any elements were set
  if (length(theme_elements) > 0) {
    custom_theme <- do.call(ggplot2::theme, theme_elements)
    ggplot2::theme_set(custom_theme)
  }

  # Set ggplot2 discrete color scale using the 12 theme colors
  # Note: Modifying ggplot2's default scales requires updating options
  if (length(colors) > 0) {
    options(
      ggplot2.discrete.colour = colors,
      ggplot2.discrete.fill = colors
    )
  }

  # Set default geom aesthetics to use plot-color-1 when no aesthetic is mapped
  # This ensures plots without explicit color mappings use the theme's primary color
  # instead of ggplot2's defaults (black for points/lines, grey for bars/histograms)
  # Note: update_geom_defaults uses lowercase geom names (e.g., "point" for geom_point)
  # Note: geom_histogram inherits from "bar" since it's just geom_bar with stat_bin
  if (!is.null(color_1 <- get_var("plot-color-1"))) {
    # For points, also set the size from plot-point-size if available
    # ggplot2's point size parameter uses mm, so convert from pt
    point_defaults <- list(colour = color_1)
    if (!is.null(point_size <- parse_number(get_var("plot-point-size")))) {
      point_defaults$size <- pt_to_mm(point_size)
    }
    ggplot2::update_geom_defaults("point", point_defaults)

    ggplot2::update_geom_defaults("line", list(colour = color_1))
    ggplot2::update_geom_defaults("path", list(colour = color_1))
    ggplot2::update_geom_defaults("bar", list(fill = color_1))
    ggplot2::update_geom_defaults("col", list(fill = color_1))
    ggplot2::update_geom_defaults("boxplot", list(
      fill = color_1,
      colour = get_var("plot-axis-line-color")  # whiskers/outline use axis color for contrast
    ))
    ggplot2::update_geom_defaults("density", list(fill = color_1))
    ggplot2::update_geom_defaults("area", list(fill = color_1))
    ggplot2::update_geom_defaults("ribbon", list(fill = color_1))
  }

  invisible(NULL)
}
