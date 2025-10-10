import json

def theme(variables_json: str) -> None:
    """
    Apply the document theme to the kernel instance

    Applies Stencila `plot-*` theme variables to matplotlib `rcParams`.
    See https://matplotlib.org/stable/api/matplotlib_configuration_api.html#matplotlib.rcParams
    and https://matplotlib.org/stable/users/explain/customizing.html#runtime-rc-settings.

    Color variables in the theme use hex colors and size values use numbers
    in pt units.
    """

    try:
        import matplotlib.pyplot as plt
        from cycler import cycler
    except ImportError:
        return

    variables: dict[str, str] = json.loads(variables_json)

    # Helper to get variable value, preferring dark mode if background is dark
    def get_var(name: str, dark_suffix: str = "-dark") -> str | None:
        # TODO: Detect dark mode properly, for now just use light mode
        return variables.get(name)

    # Helper to parse numeric values from CSS units
    def parse_number(value: str | None) -> float | None:
        if value is None:
            return None
        try:
            return float(value)
        except ValueError:
            return None

    # Helper to parse CSS font stacks into a list of font names
    def parse_fonts(value: str | None) -> list[str] | None:
        if value is None:
            return None
        # Split on commas and clean up each font name
        fonts = []
        for font in value.split(","):
            # Strip whitespace and quotes
            font = font.strip().strip("'\"")
            if font:
                fonts.append(font)
        return fonts if fonts else None
    
    # <NA> = No corresponding Stencila theme variable available yet

    # Figure

    # Figure background
    if bg := get_var("plot-background"):
        plt.rcParams["figure.facecolor"] = bg
        plt.rcParams["savefig.facecolor"] = bg
    # plt.rcParams["figure.edgecolor"] = <NA>
    # plt.rcParams["savefig.edgecolor"] = <NA>
    
    # plt.rcParams["figure.figsize"] = <NA>
    
    # plt.rcParams["figure.dpi"] = <NA>
    # plt.rcParams["savefig.dpi"] = <NA>
    
    # plt.rcParams["figure.autolayout"] = <NA>

    # Constrained layout with padding
    # matplotlib's constrained layout automatically prevents labels from overlapping
    # and allows setting padding in absolute units (inches)
    top = parse_number(get_var("plot-padding-top"))
    bottom = parse_number(get_var("plot-padding-bottom"))
    left = parse_number(get_var("plot-padding-left"))
    right = parse_number(get_var("plot-padding-right"))

    if top is not None or bottom is not None or left is not None or right is not None:
        # Enable constrained layout
        plt.rcParams["figure.constrained_layout.use"] = True

        # Convert pt to inches (1 pt = 1/72 inch)
        # h_pad controls top and bottom padding
        # w_pad controls left and right padding
        # Use the maximum of each pair for symmetric padding, defaulting to 0
        h_pad_inches = max(top or 0, bottom or 0) / 72
        w_pad_inches = max(left or 0, right or 0) / 72

        plt.rcParams["figure.constrained_layout.h_pad"] = h_pad_inches
        plt.rcParams["figure.constrained_layout.w_pad"] = w_pad_inches

    # plt.rcParams["figure.constrained_layout.hspace"] = <NA>
    # plt.rcParams["figure.constrained_layout.wspace"] = <NA>

    # Note: figure.subplot.* parameters are NOT used because they define axes position
    # as fractions (0-1), which doesn't translate well to absolute padding values and
    # can interfere with matplotlib's automatic layout for titles and labels.
    # plt.rcParams["figure.subplot.left"] = <NA>
    # plt.rcParams["figure.subplot.right"] = <NA>
    # plt.rcParams["figure.subplot.bottom"] = <NA>
    # plt.rcParams["figure.subplot.top"] = <NA>
    # plt.rcParams["figure.subplot.hspace"] = <NA>
    # plt.rcParams["figure.subplot.wspace"] = <NA>

    # plt.rcParams["figure.titlesize"] = <NA>
    # plt.rcParams["figure.titleweight"] = <NA>
    # plt.rcParams["figure.labelsize"] = <NA>
    # plt.rcParams["figure.labelweight"] = <NA>
    # plt.rcParams["figure.frameon"] = <NA>
    # plt.rcParams["figure.max_open_warning"] = <NA>
    # plt.rcParams["figure.raise_window"] = <NA>

    # plt.rcParams["savefig.format"] = <NA>
    # plt.rcParams["savefig.bbox"] = <NA>
    # plt.rcParams["savefig.pad_inches"] = <NA>
    # plt.rcParams["savefig.transparent"] = <NA>
    # plt.rcParams["savefig.orientation"] = <NA>

    # Axes

    # Axes background
    if bg := get_var("plot-panel"):
        plt.rcParams["axes.facecolor"] = bg

    # Axes edge (spines/panel border)
    if color := get_var("plot-axis-line-color"):
        plt.rcParams["axes.edgecolor"] = color
    if width := parse_number(get_var("plot-panel-border-width")):
        plt.rcParams["axes.linewidth"] = width

    # Axes labels (axis titles)
    if color := get_var("plot-axis-title-color"):
        plt.rcParams["axes.labelcolor"] = color
    if size := parse_number(get_var("plot-axis-title-size")):
        plt.rcParams["axes.labelsize"] = size
    # plt.rcParams["axes.labelpad"] = <NA>
    # plt.rcParams["axes.labelweight"] = <NA>

    # Axes title
    if color := get_var("plot-text-color"):
        plt.rcParams["axes.titlecolor"] = color
    if size := parse_number(get_var("plot-title-size")):
        plt.rcParams["axes.titlesize"] = size
    # plt.rcParams["axes.titlepad"] = <NA>
    # plt.rcParams["axes.titleweight"] = <NA>

    # Color cycle
    colors = []
    for i in range(1, 13):
        if color := get_var(f"plot-color-{i}"):
            colors.append(color)
    if colors:
        plt.rcParams["axes.prop_cycle"] = cycler(color=colors)

    # Axes grid
    # Grid color
    if color := get_var("plot-grid-color"):
        plt.rcParams["grid.color"] = color

    # Check for axis-specific grid widths first, then fall back to general grid width
    grid_x_width = parse_number(get_var("plot-grid-x-width"))
    grid_y_width = parse_number(get_var("plot-grid-y-width"))
    grid_width = parse_number(get_var("plot-grid-width"))

    # Determine which axes should show grid
    if grid_x_width is not None or grid_y_width is not None:
        # Use axis-specific widths
        if grid_x_width is not None and grid_x_width > 0:
            if grid_y_width is not None and grid_y_width > 0:
                plt.rcParams["axes.grid"] = True
                plt.rcParams["axes.grid.axis"] = "both"
                # Note: matplotlib can't set different widths per axis via rcParams
                # Use average or max of the two
                plt.rcParams["grid.linewidth"] = max(grid_x_width, grid_y_width)
            else:
                plt.rcParams["axes.grid"] = True
                plt.rcParams["axes.grid.axis"] = "x"
                plt.rcParams["grid.linewidth"] = grid_x_width
        elif grid_y_width is not None and grid_y_width > 0:
            plt.rcParams["axes.grid"] = True
            plt.rcParams["axes.grid.axis"] = "y"
            plt.rcParams["grid.linewidth"] = grid_y_width
    elif grid_width is not None and grid_width > 0:
        # Use general grid width
        plt.rcParams["axes.grid"] = True
        plt.rcParams["grid.linewidth"] = grid_width
    # plt.rcParams["axes.grid.which"] = <NA>
    # plt.rcParams["grid.linestyle"] = ...  # plot-grid-dash is "0" or "4 2", needs conversion
    # plt.rcParams["grid.alpha"] = <NA>

    # Axes spines visibility
    # plt.rcParams["axes.spines.left"] = <NA>
    # plt.rcParams["axes.spines.bottom"] = <NA>
    # plt.rcParams["axes.spines.top"] = <NA>
    # plt.rcParams["axes.spines.right"] = <NA>

    # Axes margins and limits
    # plt.rcParams["axes.xmargin"] = <NA>
    # plt.rcParams["axes.ymargin"] = <NA>
    # plt.rcParams["axes.zmargin"] = <NA>
    # plt.rcParams["axes.autolimit_mode"] = <NA>

    # Axes formatters
    # plt.rcParams["axes.formatter.limits"] = <NA>
    # plt.rcParams["axes.formatter.use_locale"] = <NA>
    # plt.rcParams["axes.formatter.use_mathtext"] = <NA>
    # plt.rcParams["axes.formatter.min_exponent"] = <NA>
    # plt.rcParams["axes.formatter.useoffset"] = <NA>
    # plt.rcParams["axes.formatter.offset_threshold"] = <NA>

    # Axes other
    # plt.rcParams["axes.unicode_minus"] = <NA>
    # plt.rcParams["axes.axisbelow"] = <NA>
    # plt.rcParams["axes.titlelocation"] = <NA>
    # plt.rcParams["axes.titlepad"] = <NA> (already listed above)
    # plt.rcParams["axes.titleweight"] = <NA> (already listed above)
    # plt.rcParams["axes.titley"] = <NA>
    # plt.rcParams["axes3d.grid"] = <NA>

    # Lines

    if width := parse_number(get_var("plot-line-width")):
        plt.rcParams["lines.linewidth"] = width
    # plt.rcParams["lines.linestyle"] = ...  # plot-line-dash needs conversion
    if cap := get_var("plot-line-cap"):
        if cap in ["butt", "round", "projecting"]:
            plt.rcParams["lines.solid_capstyle"] = cap
            plt.rcParams["lines.dash_capstyle"] = cap
    if join := get_var("plot-line-join"):
        if join in ["miter", "round", "bevel"]:
            plt.rcParams["lines.solid_joinstyle"] = join
            plt.rcParams["lines.dash_joinstyle"] = join

    if marker_size := parse_number(get_var("plot-point-size")):
        plt.rcParams["lines.markersize"] = marker_size
    # plt.rcParams["lines.marker"] = <NA>
    if marker_width := parse_number(get_var("plot-point-border-width")):
        plt.rcParams["lines.markeredgewidth"] = marker_width
    # plt.rcParams["lines.antialiased"] = <NA>

    # Markers

    # plt.rcParams["markers.fillstyle"] = <NA>

    # Patches (for bar charts, etc.)

    if width := parse_number(get_var("plot-bar-border-width")):
        plt.rcParams["patch.linewidth"] = width
    # plt.rcParams["patch.facecolor"] = ...  # Uses color cycle
    # plt.rcParams["patch.edgecolor"] = <NA>
    # plt.rcParams["patch.force_edgecolor"] = <NA>
    # plt.rcParams["patch.antialiased"] = <NA>

    # Fonts and text

    if fonts := parse_fonts(get_var("plot-font-family")):
        # matplotlib expects a list of font names or generic families
        plt.rcParams["font.family"] = fonts

    if mono_fonts := parse_fonts(get_var("plot-font-mono")):
        # Set the list of monospace fonts
        plt.rcParams["font.monospace"] = mono_fonts

    if size := parse_number(get_var("plot-font-size")):
        plt.rcParams["font.size"] = size
    # plt.rcParams["font.weight"] = <NA>
    # plt.rcParams["font.style"] = <NA>

    if color := get_var("plot-text-color"):
        plt.rcParams["text.color"] = color
    # plt.rcParams["text.antialiased"] = <NA>

    # X-axis ticks

    if color := get_var("plot-tick-color"):
        plt.rcParams["xtick.color"] = color
    if size := parse_number(get_var("plot-font-size")):
        plt.rcParams["xtick.labelsize"] = size
    if tick_size := parse_number(get_var("plot-tick-size")):
        plt.rcParams["xtick.major.size"] = tick_size
    if width := parse_number(get_var("plot-tick-width")):
        plt.rcParams["xtick.major.width"] = width
    # plt.rcParams["xtick.major.pad"] = <NA>
    # plt.rcParams["xtick.minor.size"] = <NA>
    # plt.rcParams["xtick.minor.width"] = <NA>
    # plt.rcParams["xtick.minor.pad"] = <NA>
    # plt.rcParams["xtick.minor.visible"] = <NA>
    # plt.rcParams["xtick.direction"] = <NA>
    # plt.rcParams["xtick.alignment"] = <NA>
    # plt.rcParams["xtick.top"] = <NA>
    # plt.rcParams["xtick.bottom"] = <NA>
    # plt.rcParams["xtick.labeltop"] = <NA>
    # plt.rcParams["xtick.labelbottom"] = <NA>

    # Y-axis ticks

    if color := get_var("plot-tick-color"):
        plt.rcParams["ytick.color"] = color
    if size := parse_number(get_var("plot-font-size")):
        plt.rcParams["ytick.labelsize"] = size
    if tick_size := parse_number(get_var("plot-tick-size")):
        plt.rcParams["ytick.major.size"] = tick_size
    if width := parse_number(get_var("plot-tick-width")):
        plt.rcParams["ytick.major.width"] = width
    # plt.rcParams["ytick.major.pad"] = <NA>
    # plt.rcParams["ytick.minor.size"] = <NA>
    # plt.rcParams["ytick.minor.width"] = <NA>
    # plt.rcParams["ytick.minor.pad"] = <NA>
    # plt.rcParams["ytick.minor.visible"] = <NA>
    # plt.rcParams["ytick.direction"] = <NA>
    # plt.rcParams["ytick.alignment"] = <NA>
    # plt.rcParams["ytick.left"] = <NA>
    # plt.rcParams["ytick.right"] = <NA>
    # plt.rcParams["ytick.labelleft"] = <NA>
    # plt.rcParams["ytick.labelright"] = <NA>

    # Legend

    if bg := get_var("plot-legend-background"):
        plt.rcParams["legend.facecolor"] = bg
    if color := get_var("plot-legend-border-color"):
        plt.rcParams["legend.edgecolor"] = color
    if width := parse_number(get_var("plot-legend-border-width")):
        if width > 0:
            plt.rcParams["legend.frameon"] = True
        # Note: legend.linewidth is not a valid rcParam in matplotlib
        # Border width must be set per-legend via legend.get_frame().set_linewidth()
    if size := parse_number(get_var("plot-legend-size")):
        plt.rcParams["legend.fontsize"] = size
    if color := get_var("plot-legend-text-color"):
        plt.rcParams["legend.labelcolor"] = color
    # plt.rcParams["legend.loc"] = ...  # plot-legend-position is "auto", could map
    # plt.rcParams["legend.framealpha"] = <NA>
    # plt.rcParams["legend.shadow"] = <NA>
    # plt.rcParams["legend.numpoints"] = <NA>
    # plt.rcParams["legend.scatterpoints"] = <NA>
    # plt.rcParams["legend.markerscale"] = ...  # plot-legend-marker-size exists but needs conversion
    # plt.rcParams["legend.columnspacing"] = <NA>
    # plt.rcParams["legend.labelspacing"] = ...  # plot-legend-gap exists but may need conversion
    # plt.rcParams["legend.handlelength"] = <NA>
    # plt.rcParams["legend.handleheight"] = <NA>
    # plt.rcParams["legend.handletextpad"] = <NA>
    # plt.rcParams["legend.borderpad"] = <NA>
    # plt.rcParams["legend.borderaxespad"] = <NA>
    # plt.rcParams["legend.title_fontsize"] = <NA>
    # plt.rcParams["legend.fancybox"] = <NA>

    # Box plot

    # plt.rcParams["boxplot.notch"] = <NA>
    # plt.rcParams["boxplot.vertical"] = <NA>
    # plt.rcParams["boxplot.whiskers"] = <NA>
    # plt.rcParams["boxplot.bootstrap"] = <NA>
    # plt.rcParams["boxplot.patchartist"] = <NA>
    # plt.rcParams["boxplot.showmeans"] = <NA>
    # plt.rcParams["boxplot.showcaps"] = <NA>
    # plt.rcParams["boxplot.showbox"] = <NA>
    # plt.rcParams["boxplot.showfliers"] = <NA>
    # plt.rcParams["boxplot.meanline"] = <NA>

    # Scatter

    # plt.rcParams["scatter.marker"] = <NA>
    # plt.rcParams["scatter.edgecolors"] = <NA>

    # Error bar

    # plt.rcParams["errorbar.capsize"] = <NA>

    # Histogram

    # plt.rcParams["hist.bins"] = <NA>

    # Date

    # plt.rcParams["date.autoformatter.year"] = <NA>
    # plt.rcParams["date.autoformatter.month"] = <NA>
    # plt.rcParams["date.autoformatter.day"] = <NA>
    # plt.rcParams["date.autoformatter.hour"] = <NA>
    # plt.rcParams["date.autoformatter.minute"] = <NA>
    # plt.rcParams["date.autoformatter.second"] = <NA>
    # plt.rcParams["date.autoformatter.microsecond"] = <NA>
    # plt.rcParams["date.epoch"] = <NA>
    # plt.rcParams["date.converter"] = <NA>

    # Image

    # plt.rcParams["image.aspect"] = <NA>
    # plt.rcParams["image.cmap"] = <NA>
    # plt.rcParams["image.interpolation"] = <NA>
    # plt.rcParams["image.interpolation_stage"] = <NA>
    # plt.rcParams["image.origin"] = <NA>
    # plt.rcParams["image.resample"] = <NA>
    # plt.rcParams["image.lut"] = <NA>
    # plt.rcParams["image.composite_image"] = <NA>

    # Contour

    # plt.rcParams["contour.negative_linestyle"] = <NA>
    # plt.rcParams["contour.corner_mask"] = <NA>
    # plt.rcParams["contour.linewidth"] = <NA>
    # plt.rcParams["contour.algorithm"] = <NA>

    # Pcolor/Pcolormesh

    # plt.rcParams["pcolor.shading"] = <NA>
    # plt.rcParams["pcolormesh.snap"] = <NA>

    # Polar axes

    # plt.rcParams["polaraxes.grid"] = <NA>

    # Animation

    # plt.rcParams["animation.html"] = <NA>
    # plt.rcParams["animation.writer"] = <NA>
    # plt.rcParams["animation.codec"] = <NA>
    # plt.rcParams["animation.bitrate"] = <NA>
    # plt.rcParams["animation.frame_format"] = <NA>
    # plt.rcParams["animation.ffmpeg_path"] = <NA>
    # plt.rcParams["animation.ffmpeg_args"] = <NA>
    # plt.rcParams["animation.convert_path"] = <NA>
    # plt.rcParams["animation.convert_args"] = <NA>
    # plt.rcParams["animation.embed_limit"] = <NA>

    # Path

    # plt.rcParams["path.simplify"] = <NA>
    # plt.rcParams["path.simplify_threshold"] = <NA>
    # plt.rcParams["path.snap"] = <NA>
    # plt.rcParams["path.sketch"] = <NA>
    # plt.rcParams["path.effects"] = <NA>

    # AGG rendering

    # plt.rcParams["agg.path.chunksize"] = <NA>

    # Mathtext

    # plt.rcParams["mathtext.fontset"] = <NA>
    # plt.rcParams["mathtext.default"] = <NA>
    # plt.rcParams["mathtext.fallback"] = <NA>
    # plt.rcParams["mathtext.rm"] = <NA>
    # plt.rcParams["mathtext.it"] = <NA>
    # plt.rcParams["mathtext.bf"] = <NA>
    # plt.rcParams["mathtext.sf"] = <NA>
    # plt.rcParams["mathtext.tt"] = <NA>
    # plt.rcParams["mathtext.cal"] = <NA>

    # Backend

    # plt.rcParams["backend"] = <NA>
    # plt.rcParams["backend_fallback"] = <NA>
    # plt.rcParams["interactive"] = <NA>
    # plt.rcParams["toolbar"] = <NA>
    # plt.rcParams["webagg.port"] = <NA>
    # plt.rcParams["webagg.address"] = <NA>
    # plt.rcParams["webagg.open_in_browser"] = <NA>
    # plt.rcParams["webagg.port_retries"] = <NA>

    # Miscellaneous

    # plt.rcParams["timezone"] = <NA>
    # plt.rcParams["datapath"] = <NA>
    # plt.rcParams["pdf.compression"] = <NA>
    # plt.rcParams["pdf.fonttype"] = <NA>
    # plt.rcParams["pdf.use14corefonts"] = <NA>
    # plt.rcParams["pdf.inheritcolor"] = <NA>
    # plt.rcParams["ps.papersize"] = <NA>
    # plt.rcParams["ps.useafm"] = <NA>
    # plt.rcParams["ps.usedistiller"] = <NA>
    # plt.rcParams["ps.distiller.res"] = <NA>
    # plt.rcParams["ps.fonttype"] = <NA>
    # plt.rcParams["svg.image_inline"] = <NA>
    # plt.rcParams["svg.fonttype"] = <NA>
    # plt.rcParams["svg.hashsalt"] = <NA>

