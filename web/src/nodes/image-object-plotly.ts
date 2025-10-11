import { css } from '@twind/core'
import { html } from 'lit'

import { deepMerge } from '../utilities/deepMerge'
import { buildPlotTheme, PlotTokens } from '../utilities/plotTheme'

/**
 * Convert plot tokens to Plotly template
 */
function toPlotlyTemplate(t: PlotTokens): Partial<any> {
  const axisBase = {
    color: t.textColor,
    title: {
      font: { family: t.fontFamily, size: t.axisTitleSize, color: t.axis.titleColor },
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
        font: { family: t.fontFamily, size: t.legendSize, color: t.legend.textColor },
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
    const template = toPlotlyTemplate(theme)

    // Merge template with user layout, preserving axis styling using deep merge
    const layout = {
      ...template.layout,
      ...spec.layout,
      xaxis: deepMerge(template.layout.xaxis, spec.layout?.xaxis),
      yaxis: deepMerge(template.layout.yaxis, spec.layout?.yaxis),
    }

    // Apply palette colors to traces if not specified
    const data = spec.data.map((trace: any, i: number) => {
      const color = theme.palette[i % theme.palette.length]

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

      return {
        ...trace,
        marker: {
          ...trace.marker,
          color: trace.marker?.color ?? color,
          size: trace.marker?.size ?? theme.mark.pointSize,
        },
        line: {
          ...trace.line,
          color: trace.line?.color ?? color,
          width: trace.line?.width ?? theme.mark.lineWidth,
          dash: theme.mark.lineDash > 0 ? 'dash' : 'solid',
        },
      }
    })

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

    await Plotly.react(container, data, layout, config)

    // find plotly.js dynamically generated style tags
    const styleTags = Array.from(
      document.head.getElementsByTagName('style')
    ).filter((tag) => {
      return tag.id.startsWith('plotly.js')
    })

    let style = ''
    // copy rules from each style tag's `sheet` object
    styleTags.forEach((tag) => {
      const sheet = tag.sheet
      Array.from(sheet.cssRules).forEach((rule) => {
        style += rule.cssText + '\n'
      })
    })
    // patch style rule for correct modebar display
    style += '.plotly .modebar-btn { display: inline-block; }'

    // add rules to shadow dom style tag
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
