import { css } from '@twind/core'
import { html } from 'lit'

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
    showgrid: t.axis.gridWidth > 0,
    gridcolor: t.axis.gridColor,
    griddash: t.axis.gridDash > 0 ? 'dash' : 'solid',
    gridwidth: t.axis.gridWidth,
    tickcolor: t.axis.tickColor,
    ticklen: t.axis.tickSize,
    tickwidth: t.axis.tickWidth,
    zerolinecolor: t.zero,
  }

  return {
    layout: {
      colorway: t.palette,
      paper_bgcolor: t.background,
      plot_bgcolor: t.panel,
      font: { family: t.fontFamily, size: t.fontSize, color: t.textColor },
      margin: {
        // Using padding tokens makes margins too narrow
        //t: t.padding.top,
        //r: t.padding.right,
        //b: t.padding.bottom,
        //l: t.padding.left,
      },
      xaxis: axisBase,
      yaxis: axisBase,
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

    // Deep merge axes to preserve theme styling when user provides axis config
    const mergeAxis = (themeAxis: any, userAxis: any) => {
      if (!userAxis) return themeAxis
      return { ...themeAxis, ...userAxis }
    }

    // Merge template with user layout, preserving axis styling
    const layout = {
      ...template.layout,
      ...spec.layout,
      xaxis: mergeAxis(template.layout.xaxis, spec.layout?.xaxis),
      yaxis: mergeAxis(template.layout.yaxis, spec.layout?.yaxis),
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
  const containerStyles = css`
    & {
      width: 100%;
    }
  `
  return html`
    <style id="plotly-css"></style>
    <div slot="content">
      <div class=${containerStyles} id="stencila-plotly-container"></div>
    </div>
  `
}
