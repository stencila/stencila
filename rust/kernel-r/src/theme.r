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

  # Background and foreground

  # Background color
  if (!is.null(bg <- get_var("plot-background"))) {
    params$bg <- bg
  }

  # Foreground color (used for axes, boxes, etc.)
  if (!is.null(color <- get_var("plot-axis-line-color"))) {
    params$fg <- color
  }

  # Plot colors

  # Default plotting color (use first color from palette)
  if (!is.null(color <- get_var("plot-color-1"))) {
    params$col <- color
  }

  # Color for axis annotation
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

  # Character expansion (point size)
  if (!is.null(point_size <- parse_number(get_var("plot-point-size")))) {
    params$cex <- point_size / 6  # Convert pt to relative size (6pt ≈ 1.0 for points)
  }

  # Text properties

  # Font family
  if (!is.null(family <- map_font_family(parse_fonts(get_var("plot-font-family"))))) {
    # R supports: "serif", "sans", "mono", "symbol"
    params$family <- family
  }

  # Font size (points)
  if (!is.null(ps <- parse_number(get_var("plot-font-size")))) {
    params$ps <- ps
  }

  # Character expansion for axis annotation
  if (!is.null(size <- parse_number(get_var("plot-font-size")))) {
    params$cex.axis <- size / 12  # Convert pt to relative size (12pt = 1.0)
  }

  # Character expansion for x and y labels
  if (!is.null(size <- parse_number(get_var("plot-axis-title-size")))) {
    params$cex.lab <- size / 12  # Convert pt to relative size (12pt = 1.0)
  }

  # Character expansion for main title
  if (!is.null(size <- parse_number(get_var("plot-title-size")))) {
    params$cex.main <- size / 12  # Convert pt to relative size (12pt = 1.0)
  }

  # Character expansion for sub-title
  if (!is.null(size <- parse_number(get_var("plot-subtitle-size")))) {
    params$cex.sub <- size / 12  # Convert pt to relative size (12pt = 1.0)
  }

  # Font face for axis annotation
  # params$font.axis <- NA  # No corresponding variable

  # Font face for x and y labels
  # params$font.lab <- NA  # No corresponding variable

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

  # Tick mark length (negative values are inside, positive outside)
  if (!is.null(tick_size <- parse_number(get_var("plot-tick-size")))) {
    params$tcl <- -tick_size / 12  # Convert pt to relative size and invert for inside ticks
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

  # Margins and layout

  # Margins (lines of text)
  # params$mar <- NA  # plot-padding-* exist but are in rem, need conversion

  # Margins in inches
  # params$mai <- NA  # plot-padding-* exist but are in rem, need conversion

  # Outer margins (lines of text)
  # params$oma <- NA  # No corresponding variable

  # Outer margins in inches
  # params$omi <- NA  # No corresponding variable

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

  # Box type around plot: "o"=box, "l"=L, "7"=top+right, "c"=C, "u"=U, "n"=none
  # params$bty <- NA  # No corresponding variable

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

  if (!requireNamespace("ggplot2", quietly = TRUE)) {
    return(invisible(NULL))
  }

  # Build ggplot2 theme elements
  theme_elements <- list()

  # Plot-level

  # Plot background
  if (!is.null(bg <- get_var("plot-background"))) {
    theme_elements$plot.background <- ggplot2::element_rect(fill = bg, color = NA)
  }

  # Plot title
  if (!is.null(color <- get_var("plot-text-color"))) {
    size <- parse_number(get_var("plot-title-size"))
    if (!is.null(size)) {
      theme_elements$plot.title <- ggplot2::element_text(color = color, size = size)
    } else {
      theme_elements$plot.title <- ggplot2::element_text(color = color)
    }
  }

  # Plot subtitle
  if (!is.null(color <- get_var("plot-text-color"))) {
    size <- parse_number(get_var("plot-subtitle-size"))
    if (!is.null(size)) {
      theme_elements$plot.subtitle <- ggplot2::element_text(color = color, size = size)
    } else {
      theme_elements$plot.subtitle <- ggplot2::element_text(color = color)
    }
  }

  # Plot caption
  if (!is.null(color <- get_var("plot-text-color"))) {
    theme_elements$plot.caption <- ggplot2::element_text(color = color)
  }

  # Plot margin
  # theme_elements$plot.margin <- NA  # plot-padding-* exist but are in rem, need conversion

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
  if (!is.null(bg <- get_var("plot-background"))) {
    theme_elements$panel.background <- ggplot2::element_rect(fill = bg, color = NA)
  }

  # Panel border
  if (!is.null(color <- get_var("plot-panel-border-color"))) {
    width <- parse_number(get_var("plot-panel-border-width"))
    if (!is.null(width) && width > 0) {
      theme_elements$panel.border <- ggplot2::element_rect(color = color, linewidth = width, fill = NA)
    } else {
      theme_elements$panel.border <- ggplot2::element_blank()
    }
  }

  # Panel grid
  if (!is.null(color <- get_var("plot-grid-color"))) {
    width <- parse_number(get_var("plot-grid-width"))
    if (!is.null(width)) {
      theme_elements$panel.grid.major <- ggplot2::element_line(color = color, linewidth = width)
      # theme_elements$panel.grid.major <- ggplot2::element_line(color = color, linewidth = width, linetype = ...)  # plot-grid-dash needs conversion
    }
  }

  # Panel grid minor (use same color as major)
  if (!is.null(color <- get_var("plot-grid-color"))) {
    theme_elements$panel.grid.minor <- ggplot2::element_line(color = color, linewidth = 0.5)
  }

  # Panel grid axis-specific
  # theme_elements$panel.grid.major.x <- NA  # No corresponding variable
  # theme_elements$panel.grid.major.y <- NA  # No corresponding variable
  # theme_elements$panel.grid.minor.x <- NA  # No corresponding variable
  # theme_elements$panel.grid.minor.y <- NA  # No corresponding variable

  # Panel spacing (between facets)
  # theme_elements$panel.spacing <- NA  # plot-gap-x and plot-gap-y exist but need implementation
  # theme_elements$panel.spacing.x <- NA  # plot-gap-x exists but needs implementation
  # theme_elements$panel.spacing.y <- NA  # plot-gap-y exists but needs implementation

  # Panel ontop (panel on top of data)
  # theme_elements$panel.ontop <- NA  # No corresponding variable

  # Axes

  # Axis line
  if (!is.null(color <- get_var("plot-axis-line-color"))) {
    width <- parse_number(get_var("plot-axis-line-width"))
    if (!is.null(width)) {
      theme_elements$axis.line <- ggplot2::element_line(color = color, linewidth = width)
    }
  }

  # Axis text (tick labels)
  if (!is.null(color <- get_var("plot-tick-color"))) {
    size <- parse_number(get_var("plot-font-size"))
    if (!is.null(size)) {
      theme_elements$axis.text <- ggplot2::element_text(color = color, size = size)
      theme_elements$axis.text.x <- ggplot2::element_text(color = color, size = size)
      theme_elements$axis.text.y <- ggplot2::element_text(color = color, size = size)
    } else {
      theme_elements$axis.text <- ggplot2::element_text(color = color)
      theme_elements$axis.text.x <- ggplot2::element_text(color = color)
      theme_elements$axis.text.y <- ggplot2::element_text(color = color)
    }
  }

  # Axis title
  if (!is.null(color <- get_var("plot-axis-title-color"))) {
    size <- parse_number(get_var("plot-axis-title-size"))
    if (!is.null(size)) {
      theme_elements$axis.title <- ggplot2::element_text(color = color, size = size)
      theme_elements$axis.title.x <- ggplot2::element_text(color = color, size = size)
      theme_elements$axis.title.y <- ggplot2::element_text(color = color, size = size)
    } else {
      theme_elements$axis.title <- ggplot2::element_text(color = color)
      theme_elements$axis.title.x <- ggplot2::element_text(color = color)
      theme_elements$axis.title.y <- ggplot2::element_text(color = color)
    }
  }

  # Axis ticks
  if (!is.null(color <- get_var("plot-tick-color"))) {
    width <- parse_number(get_var("plot-tick-width"))
    tick_size <- parse_number(get_var("plot-tick-size"))
    if (!is.null(width)) {
      theme_elements$axis.ticks <- ggplot2::element_line(color = color, linewidth = width)
    }
    if (!is.null(tick_size)) {
      theme_elements$axis.ticks.length <- ggplot2::unit(tick_size, "pt")
    }
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

  # Legend background
  if (!is.null(bg <- get_var("plot-legend-background"))) {
    theme_elements$legend.background <- ggplot2::element_rect(fill = bg, color = NA)
  }

  # Legend key (background underneath legend keys)
  if (!is.null(bg <- get_var("plot-legend-background"))) {
    theme_elements$legend.key <- ggplot2::element_rect(fill = bg, color = NA)
  }

  # Legend border
  if (!is.null(color <- get_var("plot-legend-border-color"))) {
    width <- parse_number(get_var("plot-legend-border-width"))
    if (!is.null(width) && width > 0) {
      theme_elements$legend.background <- ggplot2::element_rect(
        fill = get_var("plot-legend-background"),
        color = color,
        linewidth = width
      )
    }
  }

  # Legend text
  if (!is.null(color <- get_var("plot-legend-text-color"))) {
    size <- parse_number(get_var("plot-legend-size"))
    if (!is.null(size)) {
      theme_elements$legend.text <- ggplot2::element_text(color = color, size = size)
    } else {
      theme_elements$legend.text <- ggplot2::element_text(color = color)
    }
  }

  # Legend title
  if (!is.null(color <- get_var("plot-legend-text-color"))) {
    size <- parse_number(get_var("plot-legend-size"))
    if (!is.null(size)) {
      theme_elements$legend.title <- ggplot2::element_text(color = color, size = size)
    } else {
      theme_elements$legend.title <- ggplot2::element_text(color = color)
    }
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

  invisible(NULL)
}
