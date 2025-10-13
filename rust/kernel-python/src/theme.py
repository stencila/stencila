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

    # Matplotlib configuration
    try:
        import matplotlib.pyplot as plt
        from cycler import cycler
        has_matplotlib = True
    except ImportError:
        has_matplotlib = False
    # Only apply if matplotlib is available
    if has_matplotlib:
        # Helper to map Stencila shape names to matplotlib marker symbols
        # Maps the 8 cross-library compatible shapes to matplotlib's marker codes
        def map_shape_to_matplotlib(shape: str) -> str:
            mapping = {
                'circle': 'o',
                'square': 's',
                'triangle': '^',
                'diamond': 'D',
                'cross': 'x',
                'star': '*',
                'pentagon': 'p',
                'hexagon': 'h',
            }
            return mapping.get(shape, 'o')

        # Helper to map Stencila line type names to matplotlib linestyle codes
        # Maps the 6 cross-library compatible line types to matplotlib's linestyle formats
        def map_line_type_to_matplotlib(line_type: str) -> str | tuple:
            mapping = {
                'solid': '-',
                'dashed': '--',
                'dotted': ':',
                'dashdot': '-.',
                'longdash': (0, (8, 2)),
                'twodash': (0, (4, 2, 1, 2, 1, 2)),
            }
            return mapping.get(line_type, '-')
        
        # <NA> = No corresponding Stencila theme variable available yet

        # Figure

        # Figure background
        if bg := get_var("plot-background"):
            plt.rcParams["figure.facecolor"] = bg
            plt.rcParams["savefig.facecolor"] = bg
        # plt.rcParams["figure.edgecolor"] = <NA>
        # plt.rcParams["savefig.edgecolor"] = <NA>
    
        # Figure dimensions and DPI
        # Theme provides values in points, convert to inches (1 pt = 1/72 inch)
        plot_width_pt = parse_number(get_var("plot-width"))
        plot_height_pt = parse_number(get_var("plot-height"))
        plot_dpi = parse_number(get_var("plot-dpi"))
    
        if plot_width_pt is not None and plot_height_pt is not None:
            width_in = plot_width_pt / 72
            height_in = plot_height_pt / 72
            plt.rcParams["figure.figsize"] = [width_in, height_in]
    
        if plot_dpi is not None:
            plt.rcParams["figure.dpi"] = plot_dpi
            plt.rcParams["savefig.dpi"] = plot_dpi
    
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
        if width := parse_number(get_var("plot-axis-line-width")):
            plt.rcParams["axes.linewidth"] = width
    
        # Control spine visibility based on panel border setting
        # When false, show only left and bottom spines (axes lines)
        # to match R and ggplot2 behavior. When true, show all four spines (full border).
        panel_border_raw = get_var("plot-panel-border")
        panel_border = str(panel_border_raw).lower().strip()
        if panel_border_raw is None or panel_border in ("false", "0", "no", "off"):
            # Show only left and bottom spines (L shape, matching R's bty="l")
            plt.rcParams["axes.spines.left"] = True
            plt.rcParams["axes.spines.bottom"] = True
            plt.rcParams["axes.spines.top"] = False
            plt.rcParams["axes.spines.right"] = False
        else:
            # Show all four spines (full border, matching R's bty="o")
            plt.rcParams["axes.spines.left"] = True
            plt.rcParams["axes.spines.bottom"] = True
            plt.rcParams["axes.spines.top"] = True
            plt.rcParams["axes.spines.right"] = True
    
        # Axes labels (axis titles)
        if color := get_var("plot-axis-title-color"):
            plt.rcParams["axes.labelcolor"] = color
        if size := parse_number(get_var("plot-axis-title-size")):
            plt.rcParams["axes.labelsize"] = size
        if weight := get_var("plot-axis-title-weight"):
            plt.rcParams["axes.labelweight"] = weight
        # plt.rcParams["axes.labelpad"] = <NA>
    
        # Axes title
        if color := get_var("plot-text-color"):
            plt.rcParams["axes.titlecolor"] = color
        if size := parse_number(get_var("plot-title-size")):
            plt.rcParams["axes.titlesize"] = size
        # plt.rcParams["axes.titlepad"] = <NA>
        # plt.rcParams["axes.titleweight"] = <NA>
    
        # Color and shape cycle
        colors = []
        for i in range(1, 13):
            if color := get_var(f"plot-color-{i}"):
                colors.append(color)
    
        shapes = []
        for i in range(1, 9):
            if shape := get_var(f"plot-shape-{i}"):
                shapes.append(map_shape_to_matplotlib(shape))
    
        # Collect line_types using the 6 cross-library compatible theme line_types
        line_types = []
        for i in range(1, 7):
            if line_type := get_var(f"plot-line-type-{i}"):
                line_types.append(map_line_type_to_matplotlib(line_type))
    
        # Generate gradient colormap from ramp endpoints for continuous/sequential color scales
        # These are used for heatmaps and other plots with continuous data (imshow, contourf, etc.)
        ramp_start = get_var("plot-ramp-start")
        ramp_end = get_var("plot-ramp-end")
    
        if ramp_start and ramp_end:
            try:
                from matplotlib.colors import LinearSegmentedColormap
    
                # Create a smooth continuous colormap with 256 colors (standard for matplotlib)
                # This interpolates smoothly between start and end colors
                # Note: ramp_steps is for discrete color scales, but matplotlib heatmaps need continuous
                stencila_cmap = LinearSegmentedColormap.from_list(
                    "stencila_gradient", [ramp_start, ramp_end], N=256
                )
    
                # Register the colormap so it can be used by name
                # Try newer API first (matplotlib >= 3.5), fall back to older API
                try:
                    import matplotlib
                    matplotlib.colormaps.register(stencila_cmap, name="stencila_gradient")
                except (AttributeError, ValueError):
                    # Fall back to older API for matplotlib < 3.5
                    import matplotlib.cm as cm
                    cm.register_cmap(name="stencila_gradient", cmap=stencila_cmap)
    
                # Set as default colormap for image/heatmap functions
                plt.rcParams["image.cmap"] = "stencila_gradient"
            except (ImportError, ValueError, RuntimeError):
                pass
    
        # Get point opacity for inclusion in prop_cycle
        point_opacity = parse_number(get_var("plot-point-opacity"))
    
        # Combine colors, shapes, line_types, and alpha into prop_cycle
        # Note: alpha needs to be in the cycle because matplotlib doesn't have a global rcParam for it
        # When adding cyclers, all must have equal length - use the max and repeat shorter ones
        cycle_length = max(
            len(colors) if colors else 0,
            len(shapes) if shapes else 0,
            len(line_types) if line_types else 0
        )
    
        cycle_parts = []
        if colors and cycle_length > 0:
            # Repeat colors to match cycle_length (e.g., 12 colors repeated to 12)
            repeated_colors = (colors * ((cycle_length + len(colors) - 1) // len(colors)))[:cycle_length]
            cycle_parts.append(cycler(color=repeated_colors))
        if shapes and cycle_length > 0:
            # Repeat shapes to match cycle_length (e.g., 8 shapes repeated to 12)
            repeated_shapes = (shapes * ((cycle_length + len(shapes) - 1) // len(shapes)))[:cycle_length]
            cycle_parts.append(cycler(marker=repeated_shapes))
        if line_types and cycle_length > 0:
            # Repeat line_types to match cycle_length (e.g., 6 line types repeated to 12)
            repeated_line_types = (line_types * ((cycle_length + len(line_types) - 1) // len(line_types)))[:cycle_length]
            cycle_parts.append(cycler(linestyle=repeated_line_types))
        # Only add alpha to cycle if opacity > 0 (when using filled markers)
        if point_opacity is not None and point_opacity > 0 and cycle_length > 0:
            # Repeat alpha value for each series (all series get same alpha)
            alpha_values = [point_opacity] * cycle_length
            cycle_parts.append(cycler(alpha=alpha_values))
    
        # Combine all cycle parts
        if cycle_parts:
            combined_cycle = cycle_parts[0]
            for part in cycle_parts[1:]:
                combined_cycle = combined_cycle + part
            plt.rcParams["axes.prop_cycle"] = combined_cycle
    
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
        # plt.rcParams["grid.alpha"] = <NA>
    
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
    
        if marker_size := parse_number(get_var("plot-point-size")):
            plt.rcParams["lines.markersize"] = marker_size
        # plt.rcParams["lines.marker"] = <NA>
        # plt.rcParams["lines.markeredgewidth"] = <NA>
        # plt.rcParams["lines.antialiased"] = <NA>
    
        # Markers
    
        # Control marker fill based on plot-point-opacity:
        # - When opacity = 0: use open/unfilled markers for better overlap visibility
        # - When opacity > 0: use filled markers with the specified alpha (set in prop_cycle above)
        if point_opacity is None or point_opacity == 0:
            plt.rcParams["markers.fillstyle"] = "none"
            # Seaborn sets markeredgewidth to 0, making unfilled markers invisible
            # Set a visible edge width for unfilled markers
            line_width = parse_number(get_var("plot-line-width"))
            plt.rcParams["lines.markeredgewidth"] = line_width if line_width else 1.5
        else:
            plt.rcParams["markers.fillstyle"] = "full"
    
        # Patches (for bar charts, etc.)
    
        # plt.rcParams["patch.facecolor"] = ...  # Uses color cycle
        # plt.rcParams["patch.edgecolor"] = <NA>
        # plt.rcParams["patch.force_edgecolor"] = <NA>
        # plt.rcParams["patch.antialiased"] = <NA>
        # plt.rcParams["patch.linewidth"] = <NA>
    
        # Fonts and text
    
        if fonts := parse_fonts(get_var("plot-font-family")):
            # matplotlib expects a list of font names or generic families
            plt.rcParams["font.family"] = fonts
    
        if size := parse_number(get_var("plot-font-size")):
            plt.rcParams["font.size"] = size
        # plt.rcParams["font.weight"] = <NA>
        # plt.rcParams["font.style"] = <NA>
    
        if color := get_var("plot-text-color"):
            plt.rcParams["text.color"] = color
        # plt.rcParams["text.antialiased"] = <NA>
    
        # X-axis ticks
    
        if size := parse_number(get_var("plot-font-size")):
            plt.rcParams["xtick.labelsize"] = size
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
    
        if size := parse_number(get_var("plot-font-size")):
            plt.rcParams["ytick.labelsize"] = size
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
    
        # Store legend border width in matplotlib's rcParams for programmatic access
        if width := parse_number(get_var("plot-legend-border-width")):
            if width > 0:
                plt.rcParams["legend.frameon"] = True
        if size := parse_number(get_var("plot-legend-text-size")):
            plt.rcParams["legend.fontsize"] = size
        if color := get_var("plot-legend-text-color"):
            plt.rcParams["legend.labelcolor"] = color
    
        # Store legend position in matplotlib's rcParams for programmatic access
        # Note: 'legend.loc' is not a standard matplotlib/seaborn rcParam - location must be set per-legend
        # when calling plt.legend(loc=...) or sns.move_legend(). We store it in a custom rcParam so
        # users can access it via plt.rcParams["legend.loc"] and apply it manually:
        # - matplotlib: plt.legend(loc=plt.rcParams["legend.loc"])
        # - seaborn: sns.move_legend(obj, loc=plt.rcParams["legend.loc"])
        if position := get_var("plot-legend-position"):
            position_lower = position.lower().strip()
            # Map Stencila position values to matplotlib's loc parameter
            position_map = {
                'auto': 'best',           # matplotlib's automatic positioning
                'right': 'center right',
                'left': 'center left',
                'top': 'upper center',
                'bottom': 'lower center',
                # Note: 'none' cannot be handled via rcParams - legends must be hidden per-plot
                # using plt.legend().set_visible(False) or by not calling plt.legend()
            }
            if position_lower in position_map:
                # Store in a custom rcParam key for user access
                plt.rcParams["legend.loc"] = position_map[position_lower]
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
        # plt.rcParams["image.cmap"] = ...  # Handled via plot-ramp-* variables above
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

    # Plotly template configuration
    #
    # When Plotly is installed, configure a template that applies Stencila theme colors,
    # shapes, and line types to Plotly figures. This template uses the same palette
    # as defined in web/src/themes/base/plots.css and matches the browser-side theming
    # in web/src/nodes/image-object-plotly.ts
    try:
        import plotly.io as pio
        import plotly.graph_objects as go
        has_plotly = True
    except ImportError:
        has_plotly = False

    if has_plotly:
        # Helper to map Stencila shape names to Plotly marker symbols
        # Matches the mapping in web/src/nodes/image-object-plotly.ts
        def map_shape_to_plotly(shape: str, opacity: float = 0) -> str:
            # When opacity is 0, use open variants for better overlap visibility
            if opacity == 0:
                mapping = {
                    'circle': 'circle-open',
                    'square': 'square-open',
                    'triangle': 'triangle-up-open',
                    'diamond': 'diamond-open',
                    'cross': 'x',
                    'star': 'star-open',
                    'pentagon': 'pentagon-open',
                    'hexagon': 'hexagon-open',
                }
            else:
                # When opacity > 0, use filled variants
                mapping = {
                    'circle': 'circle',
                    'square': 'square',
                    'triangle': 'triangle-up',
                    'diamond': 'diamond',
                    'cross': 'x',
                    'star': 'star',
                    'pentagon': 'pentagon',
                    'hexagon': 'hexagon',
                }
            return mapping.get(shape, 'circle-open' if opacity == 0 else 'circle')

        # Helper to map Stencila line type names to Plotly dash patterns
        # Matches the mapping in web/src/nodes/image-object-plotly.ts
        def map_line_type_to_plotly(line_type: str) -> str:
            mapping = {
                'solid': 'solid',
                'dashed': 'dash',
                'dotted': 'dot',
                'dashdot': 'dashdot',
                'longdash': 'longdash',
                'twodash': 'dashdot',  # Plotly doesn't have twodash, use dashdot as closest match
            }
            return mapping.get(line_type, 'solid')

        # Extract theme color palette
        theme_colors = []
        for i in range(1, 13):
            if color := get_var(f"plot-color-{i}"):
                theme_colors.append(color)

        # Extract theme shapes
        theme_shapes = []
        for i in range(1, 9):
            if shape := get_var(f"plot-shape-{i}"):
                theme_shapes.append(shape)

        # Extract theme line types
        theme_line_types = []
        for i in range(1, 7):
            if line_type := get_var(f"plot-line-type-{i}"):
                theme_line_types.append(line_type)

        # Get point opacity for marker configuration
        theme_point_opacity = parse_number(get_var("plot-point-opacity")) or 0
        theme_point_size = parse_number(get_var("plot-point-size"))
        theme_line_width = parse_number(get_var("plot-line-width"))

        # Build cycling scatter trace templates
        # When Plotly adds traces, it will cycle through these configurations
        # This matches the behavior of the browser-side theming in image-object-plotly.ts
        scatter_templates = []

        # We need to create templates that cycle through all combinations
        # The maximum number of templates should match the LCM of shapes and line types,
        # but for simplicity, we'll create one template per shape (8 templates)
        # and cycle through line types within each
        max_templates = max(len(theme_shapes), len(theme_line_types)) if theme_shapes or theme_line_types else 1

        for i in range(max_templates):
            template_config = {}

            # Set marker configuration if shapes are available
            if theme_shapes:
                shape = theme_shapes[i % len(theme_shapes)]
                plotly_symbol = map_shape_to_plotly(shape, theme_point_opacity)
                marker_config = {'symbol': plotly_symbol}

                if theme_point_size is not None:
                    marker_config['size'] = theme_point_size

                # Apply opacity if > 0
                if theme_point_opacity > 0:
                    marker_config['opacity'] = theme_point_opacity

                template_config['marker'] = marker_config

            # Set line configuration if line types are available
            if theme_line_types:
                line_type = theme_line_types[i % len(theme_line_types)]
                plotly_dash = map_line_type_to_plotly(line_type)
                line_config = {'dash': plotly_dash}

                if theme_line_width is not None:
                    line_config['width'] = theme_line_width

                template_config['line'] = line_config

            scatter_templates.append(template_config)

        # Create the Plotly template
        # The colorway sets the default colors for traces
        # The data.scatter list provides cycling defaults for marker symbols and line dash patterns
        layout_config = {}
        if theme_colors:
            layout_config['colorway'] = theme_colors

        # Set margin to 0 to let browser-side CSS padding handle spacing
        # This matches the behavior in web/src/nodes/image-object-plotly.ts
        layout_config['margin'] = dict(l=0, r=0, t=0, b=0)

        data_config = {}
        if scatter_templates:
            data_config['scatter'] = [go.Scatter(**config) for config in scatter_templates]

        stencila_template = go.layout.Template(
            layout=layout_config,
            data=data_config
        )

        # Register and set as default
        pio.templates["stencila"] = stencila_template
        pio.templates.default = "stencila"
