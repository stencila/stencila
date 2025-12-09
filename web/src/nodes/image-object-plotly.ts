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
function toPlotlyTemplate(t: PlotTokens) {
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
    linecolor: t.axis.lineColor,
    linewidth: t.axis.lineWidth,
    showline: true,
    // Mirror axes to simulate panel borders if specified (will be same width and color as axes)
    mirror: t.panelBorder,
    // Enable automargin to let Plotly automatically calculate space for axis labels and titles.
    // This ensures labels/titles fit without being cut off, regardless of their size.
    automargin: true,
  }

  const legendPosition = t.legend.position.toLowerCase()

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
        gridwidth: t.axis.gridXWidth,
      },
      yaxis: {
        ...axisBase,
        showgrid: t.axis.gridYWidth > 0,
        gridcolor: t.axis.gridColor,
        gridwidth: t.axis.gridYWidth,
      },
      // Hide legend if position is 'none'
      legend: {
        ...(() => {
          // Map legend position to Plotly's coordinate system
          switch (legendPosition) {
            case 'left':
              return { x: -0.02, y: 1, xanchor: 'right', yanchor: 'top' }
            case 'top':
              return { x: 0.5, y: 1.02, xanchor: 'center', yanchor: 'bottom', orientation: 'h' }
            case 'bottom':
              return { x: 0.5, y: -0.02, xanchor: 'center', yanchor: 'top', orientation: 'h' }
            case 'right':
              return { x: 1.02, y: 1, xanchor: 'left', yanchor: 'top' }
            case 'none':
              return { showlegend: false }
            case 'auto':
            default:
              return {}
          }
        })(),
        bgcolor: t.legend.background,
        bordercolor: t.legend.borderColor,
        borderwidth: t.legend.borderWidth,
        font: { family: t.fontFamily, size: t.legend.textSize, color: t.legend.textColor },
      },
      hoverlabel: {
        bgcolor: t.tooltip.background,
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
  isStaticView: boolean,
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
      // eslint-disable-next-line
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

        // eslint-disable-next-line
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
    const config = isStaticView
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
 *
 * Uses `.plotly-padder` and `.viz-container` classes from shared styles (image-object-styles.ts)
 *
 * The padder applies theme padding around the entire plot (including axis labels/titles).
 * This matches the behavior of ECharts (containLabel: true), Vega-Lite (contains: 'padding'),
 * matplotlib (constrained_layout), ggplot2 (plot.margin), and R base (mar).
 */
export function renderPlotlyContainer() {
  return html`
    <style id="plotly-css"></style>
    <div class="plotly-padder">
      <div class="viz-container" id="stencila-plotly-container"></div>
    </div>
  `
}
