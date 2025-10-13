import { css } from '@twind/core'
import { html } from 'lit'

import { deepMerge } from '../utilities/deepMerge'
import { buildPlotTheme, PlotTokens } from '../utilities/plotTheme'

/**
 * Map Stencila shape names to Vega-Lite symbol names
 *
 * Maps the 8 cross-library compatible shapes to Vega-Lite's symbol names.
 */
function mapShapeToVegaLite(shape: string): string {
  const mapping: Record<string, string> = {
    'circle': 'circle',
    'square': 'square',
    'triangle': 'triangle-up',
    'diamond': 'diamond',
    'cross': 'cross',
    'star': 'triangle-down',
    'pentagon': 'pentagon',
    'hexagon': 'hexagon',
  }
  return mapping[shape] || 'circle'
}

/**
 * Map Stencila line type names to Vega-Lite strokeDash arrays
 *
 * Vega-Lite uses strokeDash arrays [dash, gap, dash, gap, ...] where values are in pixels.
 * An empty array [] or null means solid line.
 */
function mapLineTypeToVegaLite(lineType: string): number[] | null {
  const mapping: Record<string, number[] | null> = {
    'solid': null,  // null means solid in Vega-Lite
    'dashed': [4, 2],
    'dotted': [1, 1],
    'dashdot': [4, 2, 1, 2],
    'longdash': [8, 2],
    'twodash': [4, 2, 1, 2, 1, 2],
  }
  return mapping[lineType] !== undefined ? mapping[lineType] : null
}

/**
 * Convert plot tokens to Vega-Lite config
 */
function toVegaLiteConfig(t: PlotTokens): Record<string, unknown> {
  const axisBase = {
    labelColor: t.textColor,
    labelFontSize: t.fontSize,
    titleColor: t.axis.titleColor,
    titleFontSize: t.axis.titleSize,
    titleFontWeight: t.axis.titleWeight,
    domainColor: t.axis.lineColor,
    domainWidth: t.axis.lineWidth,
    gridColor: t.axis.gridColor,
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
      ...(() => {
        // Map legend position to Vega-Lite's orient property
        const position = t.legend.position.toLowerCase()
        switch (position) {
          case 'left':
            return { orient: 'left' }
          case 'top':
            return { orient: 'top' }
          case 'bottom':
            return { orient: 'bottom' }
          case 'right':
            return { orient: 'right' }
          case 'none':
            return { disable: true }
          case 'auto':
          default:
            return {}
        }
      })(),
      labelColor: t.legend.textColor,
      labelFontSize: t.legend.textSize,
      titleColor: t.legend.textColor,
      titleFontSize: t.legend.textSize + 1,
      gradientStrokeColor: t.axis.gridColor,
      // Note: Vega-Lite doesn't support legend box borders (borderColor/borderWidth).
      // The gradientStrokeColor property above only affects gradient legend outlines.
    },
    title: { color: t.textColor, fontSize: t.titleSize },
    view: {
      fill: t.panel,
      stroke: t.axis.lineColor,
      // Apply panel border if specified (will be same width and color as axes)
      strokeWidth: t.panelBorder ? t.axis.lineWidth : 0,
    },
    range: {
      category: t.palette,
      symbol: t.shapes.map(mapShapeToVegaLite),
      strokeDash: t.lineTypes.map(mapLineTypeToVegaLite),
      heatmap: [t.ramp.start, t.ramp.end],
    },
    header: {
      labelColor: t.textColor,
      titleColor: t.textColor,
      labelFontSize: t.fontSize,
      titleFontSize: t.titleSize,
    },
    // Set default stroke width for marks
    // Note: filled and fillOpacity are set per-mark-type below to avoid affecting line marks
    mark: {
      strokeWidth: t.mark.lineWidth,
    },

    // Set default colors and sizes for mark types when no encoding is specified
    // Note: size in Vega-Lite is area in square pixels, so we square the linear dimension
    point: {
      color: t.palette[0],
      size: t.mark.pointSize ** 2,
      filled: t.mark.pointOpacity > 0,
      fillOpacity: t.mark.pointOpacity,
    },
    circle: {
      color: t.palette[0],
      size: t.mark.pointSize ** 2,
      filled: t.mark.pointOpacity > 0,
      fillOpacity: t.mark.pointOpacity,
    },
    square: {
      color: t.palette[0],
      size: t.mark.pointSize ** 2,
      filled: t.mark.pointOpacity > 0,
      fillOpacity: t.mark.pointOpacity,
    },
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

  // If theme is null (--plot-theme: none), use spec as-is with minimal defaults
  const themedSpec = !theme
    ? {
        ...spec,
        width: 'container',
        height: 'container',
        autosize: { type: 'fit', contains: 'padding' },
      }
    : {
        ...spec,
        config: deepMerge(toVegaLiteConfig(theme), spec.config),
        width: 'container',
        height: 'container',
        autosize: { type: 'fit', contains: 'padding' },
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
      aspect-ratio: var(--plot-aspect-ratio);
      min-height: var(--plot-height-min);
      max-height: var(--plot-height-max);
    }
  `
  return html`
    <div slot="content">
      <div class=${containerStyles} id="stencila-vega-container"></div>
    </div>
  `
}
