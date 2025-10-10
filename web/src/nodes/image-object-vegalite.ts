import { css } from '@twind/core'
import { html } from 'lit'

import { buildPlotTheme, PlotTokens } from '../utilities/plotTheme'

/**
 * Convert plot tokens to Vega-Lite config
 */
function toVegaLiteConfig(t: PlotTokens): any {
  // Parse grid dash from string like "4 2" to array [4, 2]
  const gridDash = t.axis.gridDash > 0 ? [t.axis.gridDash, t.axis.gridDash / 2] : []

  const axisBase = {
    labelColor: t.textColor,
    labelFontSize: t.fontSize,
    titleColor: t.axis.titleColor,
    titleFontSize: t.axisTitleSize,
    domainColor: t.axis.lineColor,
    domainWidth: t.axis.lineWidth,
    tickColor: t.axis.tickColor,
    tickSize: t.axis.tickSize,
    tickWidth: t.axis.tickWidth,
    gridColor: t.axis.gridColor,
    gridDash: gridDash,
    labelLimit: 100,
  }

  return {
    background: t.background,
    font: t.fontFamily,
    axisX: {
      ...axisBase,
      gridWidth: t.axis.gridXWidth,
    },
    axisY: {
      ...axisBase,
      gridWidth: t.axis.gridYWidth,
    },
    legend: {
      labelColor: t.legend.textColor,
      labelFontSize: t.legendSize,
      titleColor: t.legend.textColor,
      titleFontSize: t.legendSize + 1,
      gradientStrokeColor: t.axis.gridColor,
      padding: t.legend.gap,
      // Note: Vega-Lite doesn't support legend box borders (borderColor/borderWidth).
      // The gradientStrokeColor property above only affects gradient legend outlines.
    },
    title: { color: t.textColor, fontSize: t.titleSize },
    view: {
      fill: t.panel,
      stroke: t.axis.lineColor,
      // Vega-Lite renders SVG strokes centered on paths, causing anti-aliasing blur
      // at integer widths. Halving the width produces crisp 1px borders.
      strokeWidth: t.panelBorderWidth * 0.5,
    },
    range: {
      category: t.palette,
      heatmap: [t.mark.heatMin, t.mark.heatMax],
    },
    header: {
      labelColor: t.textColor,
      titleColor: t.textColor,
      labelFontSize: t.fontSize,
      titleFontSize: t.titleSize,
    },
    mark: { opacity: t.mark.opacity, strokeWidth: t.mark.lineWidth },

    // Set default colors for mark types when no color encoding is specified
    point: { color: t.palette[0], filled: true },
    circle: { color: t.palette[0] },
    square: { color: t.palette[0] },
    bar: { color: t.palette[0] },
    line: { color: t.palette[0], strokeWidth: t.mark.lineWidth },
    area: { color: t.palette[0], opacity: t.mark.areaOpacity },
    rect: { color: t.palette[0] },
    tick: { color: t.palette[0] },
  }
}

/**
 * Compile and render a Vega-Lite visualization
 */
export async function compileVegaLite(
  contentUrl: string,
  container: HTMLElement,
  isStaticMode: boolean,
  onError: (error: Error) => void
): Promise<{ finalize: () => void } | undefined> {
  const { default: vegaEmbed } = await import('vega-embed')
  const spec = JSON.parse(contentUrl)

  // Build theme from CSS variables
  const theme = buildPlotTheme(container)
  const themeConfig = toVegaLiteConfig(theme)

  // Merge theme config with user spec
  const themedSpec = {
    ...spec,
    config: { ...themeConfig, ...(spec.config ?? {}) },
    width: 'container',
    autosize: { type: 'fit-x', contains: 'padding' },
    padding: {
      top: theme.padding.top,
      right: theme.padding.right,
      bottom: theme.padding.bottom,
      left: theme.padding.left,
    },
  }

  // embed the figure as svg
  const embedOptions = {
    renderer: 'svg' as const,
    actions: false,
    mode: 'vega-lite' as const,
    ...(isStaticMode && {
      config: {
        view: { continuousWidth: 400, continuousHeight: 300 },
        axis: { domain: false, ticks: false },
        legend: { disable: true },
      },
    }),
  }

  try {
    const result = await vegaEmbed(container, themedSpec, embedOptions)
    return result
  } catch (error) {
    onError(error)
    return undefined
  }
}

/**
 * Render Vega-Lite container
 */
export function renderVegaLiteContainer() {
  const containerStyles = css`
    & {
      width: 100%;
    }
  `
  return html`
    <div slot="content">
      <div class=${containerStyles} id="stencila-vega-container"></div>
    </div>
  `
}
