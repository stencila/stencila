import { css } from '@twind/core'
import { html } from 'lit'

import { deepMerge } from '../utilities/deepMerge'
import { buildPlotTheme, PlotTokens } from '../utilities/plotTheme'

/**
 * Map Stencila shape names to Plotly symbol names
 *
 * When opacity = 0, uses open/unfilled variants for better overlap visibility.
 * When opacity > 0, uses filled variants with the specified opacity.
 */
function mapShapeToPlotly(shape: string, opacity: number): string {
  // When opacity is 0, use open variants
  if (opacity === 0) {
    const openMapping: Record<string, string> = {
      'circle': 'circle-open',
      'square': 'square-open',
      'triangle': 'triangle-up-open',
      'diamond': 'diamond-open',
      'cross': 'x',
      'star': 'star-open',
      'pentagon': 'pentagon-open',
      'hexagon': 'hexagon-open',
    }
    return openMapping[shape] || 'circle-open'
  }

  // When opacity > 0, use filled variants
  const filledMapping: Record<string, string> = {
    'circle': 'circle',
    'square': 'square',
    'triangle': 'triangle-up',
    'diamond': 'diamond',
    'cross': 'x',
    'star': 'star',
    'pentagon': 'pentagon',
    'hexagon': 'hexagon',
  }
  return filledMapping[shape] || 'circle'
}

/**
 * Map Stencila lineType names to Plotly dash format
 *
 * Plotly supports 'solid', 'dot', 'dash', 'longdash', 'dashdot', 'longdashdot'
 */
function mapLineTypeToPlotly(lineType: string): string {
  const mapping: Record<string, string> = {
    'solid': 'solid',
    'dashed': 'dash',
    'dotted': 'dot',
    'dashdot': 'dashdot',
    'longdash': 'longdash',
    'twodash': 'dashdot',  // Plotly doesn't have twodash, use dashdot as closest match
  }
  return mapping[lineType] || 'solid'
}

/**
 * Convert plot tokens to Plotly template
 */
function toPlotlyTemplate(t: PlotTokens): Partial<any> {
  // Convert CSS font-weight to numeric value for Plotly (normal→400, bold→700, or pass through)
  const convertFontWeight = (weight: string): number => {
    if (weight === 'normal') return 400
    if (weight === 'bold') return 700
    const numeric = parseInt(weight, 10)
    return isNaN(numeric) ? 700 : numeric
  }

  const axisBase = {
    color: t.textColor,
    title: {
      font: {
        family: t.fontFamily,
        size: t.axis.titleSize,
        color: t.axis.titleColor,
        weight: convertFontWeight(t.axis.titleWeight),
      },
    },
    // Note: In Plotly, linecolor/linewidth control both the axis line and the mirrored
    // panel border (via mirror: true). There's no separate styling for panel borders,
    // so we use axis tokens instead of panelBorderColor/panelBorderWidth.
    linecolor: t.axis.lineColor,
    linewidth: t.axis.lineWidth,
    showline: true,
    // Mirror axes to simulate panel borders if specified (will be same width and color as axes)
    mirror: t.panelBorderWidth > 0 ? true : false,
    tickcolor: t.axis.tickColor,
    ticklen: t.axis.tickSize,
    tickwidth: t.axis.tickWidth,
    zerolinecolor: t.zero,
    // Enable automargin to let Plotly automatically calculate space for axis labels and titles.
    // This ensures labels/titles fit without being cut off, regardless of their size.
    automargin: true,
  }

  return {
    layout: {
      colorway: t.palette,
      paper_bgcolor: t.background,
      plot_bgcolor: t.panel,
      font: { family: t.fontFamily, size: t.fontSize, color: t.textColor },
      title: {
        font: { family: t.fontFamily, size: t.titleSize, color: t.textColor },
      },
      // IMPORTANT: Plotly's margin is the space allocated FOR axis labels/titles, not padding beyond them.
      // To achieve consistent padding behavior with other plotting libraries (ECharts, Vega-Lite, matplotlib, ggplot2),
      // we apply padding via CSS on the container element instead of using Plotly's margin.
      // Set margins to 0 and let automargin expand them automatically as needed to fit labels/titles.
      margin: {
        t: 0,
        r: 0,
        b: 0,
        l: 0,
      },
      xaxis: {
        ...axisBase,
        showgrid: t.axis.gridXWidth > 0,
        gridcolor: t.axis.gridColor,
        griddash: t.axis.gridDash > 0 ? 'dash' : 'solid',
        gridwidth: t.axis.gridXWidth,
      },
      yaxis: {
        ...axisBase,
        showgrid: t.axis.gridYWidth > 0,
        gridcolor: t.axis.gridColor,
        griddash: t.axis.gridDash > 0 ? 'dash' : 'solid',
        gridwidth: t.axis.gridYWidth,
      },
      legend: {
        bgcolor: t.legend.background,
        bordercolor: t.legend.borderColor,
        borderwidth: t.legend.borderWidth,
        font: { family: t.fontFamily, size: t.legend.textSize, color: t.legend.textColor },
        itemwidth: t.legend.markerSize + 10,
      },
      hoverlabel: {
        bgcolor: t.tooltip.background,
        bordercolor: t.tooltip.borderColor,
        font: { family: t.fontFamily, size: t.fontSize, color: t.tooltip.textColor },
        align: 'left',
      },
    },
  }
}

/**
 * Compile and render a Plotly chart
 */
export async function compilePlotly(
  contentUrl: string,
  container: HTMLElement,
  shadowRoot: ShadowRoot,
  isStaticMode: boolean,
  onError: (error: Error) => void
): Promise<void> {
  const Plotly = await import('plotly.js-dist-min')
  const spec = JSON.parse(contentUrl)

  try {
    // Build theme from CSS variables
    const theme = buildPlotTheme(container)

    // Prepare data and layout based on theme
    let data, layout

    if (!theme) {
      // No theming: remove padder background and use spec as-is
      const padder = container.parentElement
      if (padder) {
        padder.style.backgroundColor = 'transparent'
      }
      data = spec.data
      layout = spec.layout
    } else {
      // Apply theme to spec
      const template = toPlotlyTemplate(theme)

      // Merge template with user layout, preserving axis styling using deep merge
      layout = {
        ...template.layout,
        ...spec.layout,
        xaxis: deepMerge(template.layout.xaxis, spec.layout?.xaxis),
        yaxis: deepMerge(template.layout.yaxis, spec.layout?.yaxis),
      }

      // Apply palette colors, shapes, and lineTypes to traces if not specified
      data = spec.data.map((trace: any, i: number) => {
        const color = theme.palette[i % theme.palette.length]
        const shape = theme.shapes[i % theme.shapes.length]
        const lineType = theme.lineTypes[i % theme.lineTypes.length]

        // Special handling for heatmaps: inject ramp colorscale if not specified
        if (trace.type === 'heatmap' && !trace.colorscale) {
          return {
            ...trace,
            colorscale: [
              [0, theme.ramp.start],
              [1, theme.ramp.end],
            ],
          }
        }

        const updatedTrace: any = {
          ...trace,
          marker: {
            ...trace.marker,
            color: trace.marker?.color ?? color,
            size: trace.marker?.size ?? theme.mark.pointSize,
            symbol: trace.marker?.symbol ?? mapShapeToPlotly(shape, theme.mark.pointOpacity),
          },
          line: {
            ...trace.line,
            color: trace.line?.color ?? color,
            width: trace.line?.width ?? theme.mark.lineWidth,
            dash: trace.line?.dash ?? mapLineTypeToPlotly(lineType),
          },
        }

        // Apply opacity to marker when pointOpacity > 0
        if (theme.mark.pointOpacity > 0) {
          updatedTrace.marker.opacity = theme.mark.pointOpacity
        }

        return updatedTrace
      })
    }

    // Configure for static mode if enabled
    const config = isStaticMode
      ? {
          ...spec.config,
          staticPlot: true,
          displayModeBar: false,
          scrollZoom: false,
          doubleClick: false,
          showTips: false,
          dragMode: false,
        }
      : { displayModeBar: false, ...spec.config }

    // Render the plot
    await Plotly.react(container, data, layout, config)

    // Inject plotly.js styles into shadow DOM
    const styleTags = Array.from(
      document.head.getElementsByTagName('style')
    ).filter((tag) => tag.id.startsWith('plotly.js'))

    let style = ''
    styleTags.forEach((tag) => {
      const sheet = tag.sheet
      Array.from(sheet.cssRules).forEach((rule) => {
        style += rule.cssText + '\n'
      })
    })
    style += '.plotly .modebar-btn { display: inline-block; }'

    const shadowStyle = shadowRoot.getElementById('plotly-css')
    shadowStyle.innerText = style
  } catch (error) {
    onError(error)
  }
}

/**
 * Render Plotly container
 */
export function renderPlotlyContainer() {
  const padderStyles = css`
    & {
      /* Apply theme padding around the entire plot (including axis labels/titles).
       * This matches the behavior of ECharts (containLabel: true), Vega-Lite (contains: 'padding'),
       * matplotlib (constrained_layout), ggplot2 (plot.margin), and R base (mar).
       * The background color must match --plot-background so the padding area has the correct color. */
      padding: var(--plot-padding-top) var(--plot-padding-right) var(--plot-padding-bottom) var(--plot-padding-left);
      background-color: var(--plot-background);
      box-sizing: border-box;
    }
  `
  const containerStyles = css`
    & {
      width: 100%;
      aspect-ratio: var(--plot-aspect-ratio);
      min-height: var(--plot-height-min);
      max-height: var(--plot-height-max);
    }
  `
  return html`
    <style id="plotly-css"></style>
    <div class=${padderStyles} slot="content">
      <div class=${containerStyles} id="stencila-plotly-container"></div>
    </div>
  `
}
