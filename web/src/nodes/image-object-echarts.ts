import { css } from '@twind/core'
import { html } from 'lit'

import { deepMerge } from '../utilities/deepMerge'
import { buildPlotTheme, PlotTokens } from '../utilities/plotTheme'

/**
 * Map Stencila shape names to ECharts symbol names
 *
 * When opacity = 0, uses open/empty variants for better overlap visibility.
 * When opacity > 0, uses filled variants with the specified opacity.
 */
function mapShapeToECharts(shape: string, opacity: number): string {
  // When opacity is 0, use open/empty variants
  if (opacity === 0) {
    const openMapping: Record<string, string> = {
      'circle': 'emptyCircle',
      'square': 'emptyRect',
      'triangle': 'triangle',
      'diamond': 'emptyDiamond',
      'cross': 'cross',
      'star': 'star',
      'pentagon': 'pentagon',
      'hexagon': 'hexagon',
    }
    return openMapping[shape] || 'emptyCircle'
  }

  // When opacity > 0, use filled variants
  const filledMapping: Record<string, string> = {
    'circle': 'circle',
    'square': 'rect',
    'triangle': 'triangle',
    'diamond': 'diamond',
    'cross': 'cross',
    'star': 'star',
    'pentagon': 'pentagon',
    'hexagon': 'hexagon',
  }
  return filledMapping[shape] || 'circle'
}

/**
 * Map Stencila line type names to ECharts line type format
 *
 * ECharts supports 'solid', 'dashed', 'dotted', or custom dash arrays [dash, gap, dash, gap, ...]
 */
function mapLineTypeToECharts(lineType: string): 'solid' | 'dashed' | 'dotted' | number[] {
  const mapping: Record<string, 'solid' | 'dashed' | 'dotted' | number[]> = {
    'solid': 'solid',
    'dashed': 'dashed',
    'dotted': 'dotted',
    'dashdot': [8, 4, 2, 4],
    'longdash': [16, 4],
    'twodash': [8, 4, 2, 4, 2, 4],
  }
  return mapping[lineType] || 'solid'
}

/**
 * Convert plot tokens to ECharts option base
 */
function toEchartsOptionsBase(t: PlotTokens): Record<string, unknown> {
  // Shared axis properties (labels, lines, ticks)
  const axisBase = {
    axisLabel: { color: t.textColor, fontSize: t.fontSize, fontFamily: t.fontFamily },
    nameTextStyle: {
      color: t.axis.titleColor,
      fontSize: t.axis.titleSize,
      fontFamily: t.fontFamily,
      fontWeight: t.axis.titleWeight,
    },
    axisLine: {
      show: true,
      lineStyle: { color: t.axis.lineColor, width: t.axis.lineWidth },
    },
    axisTick: {
      show: true,
      lineStyle: { color: t.axis.lineColor },
    },
    // Center axis names along the axis (consistent with matplotlib, ggplot2, Plotly, Vega-Lite).
    // ECharts defaults to 'end' which places names at top (y-axis) or right (x-axis).
    nameLocation: 'middle',
  }

  return {
    color: t.palette,
    backgroundColor: t.background,
    textStyle: { color: t.textColor, fontFamily: t.fontFamily, fontSize: t.fontSize },
    title: {
      textStyle: { color: t.textColor, fontSize: t.titleSize, fontFamily: t.fontFamily },
      subtextStyle: { color: t.textColor, fontSize: t.subtitleSize, fontFamily: t.fontFamily },
    },
    grid: {
      // Must be true for background to render
      show: true,
      backgroundColor: t.panel,
      borderColor: t.axis.lineColor,
      borderWidth: t.axis.lineWidth,
      left: t.padding.left,
      right: t.padding.right,
      top: t.padding.top,
      bottom: t.padding.bottom,
      containLabel: true,
    },
    xAxis: {
      ...axisBase,
      // Gap between axis name and axis labels (prevents overlap with tick labels below)
      // Note: With nameLocation:'middle', this pushes the name downward from the axis line
      nameGap: 25,
      splitLine: {
        show: t.axis.gridXWidth > 0,
        lineStyle: {
          color: t.axis.gridColor,
          width: t.axis.gridXWidth,
        },
      },
    },
    yAxis: {
      ...axisBase,
      // Gap between axis name and axis labels (prevents overlap with tick labels on the left)
      // Note: With nameLocation:'middle', this pushes the name leftward from the axis line.
      nameGap: 25,
      splitLine: {
        show: t.axis.gridYWidth > 0,
        lineStyle: {
          color: t.axis.gridColor,
          width: t.axis.gridYWidth,
        },
      },
    },
    legend: {
      ...(() => {
        // Map legend position to ECharts configuration
        const position = t.legend.position.toLowerCase()
        switch (position) {
          case 'left':
            return { orient: 'vertical', left: 10, top: 10 }
          case 'top':
            return { orient: 'horizontal', top: 10, left: 'center' }
          case 'bottom':
            return { orient: 'horizontal', bottom: 10, left: 'center' }
          case 'right':
            return { orient: 'vertical', right: 10, top: 10 }
          case 'none':
            return { show: false }
          case 'auto':
          default:
            return {}
        }
      })(),
      backgroundColor: t.legend.background,
      borderColor: t.legend.borderColor,
      borderWidth: t.legend.borderWidth,
      textStyle: { color: t.legend.textColor, fontSize: t.legend.textSize }
    },
    tooltip: {
      trigger: 'item',
      backgroundColor: t.tooltip.background,
      textStyle: { color: t.tooltip.textColor, fontFamily: t.fontFamily, fontSize: t.fontSize },
    },
  }
}

/**
 * Compile and render an ECharts visualization
 *
 * @returns The ECharts instance (for resize/dispose)
 */
export async function compileECharts(
  contentUrl: string,
  container: HTMLElement,
  existingInstance: { dispose: () => void } | undefined,
  isStaticMode: boolean,
  onError: (error: Error) => void
): Promise<{ resize: () => void; dispose: () => void }> {
  const echarts = await import('echarts')
  const spec = JSON.parse(contentUrl)

  // Dispose of any existing chart instance
  if (existingInstance) {
    existingInstance.dispose()
  }

  try {
    // Initialize the chart
    const chartInstance = echarts.init(container)

    // Build theme from CSS variables
    const theme = buildPlotTheme(container)

    // Configure for static mode if enabled
    if (isStaticMode) {
      // Disable interactions in static mode
      if (!spec.toolbox) {
        spec.toolbox = {}
      }
      spec.toolbox.show = false

      // Disable zoom and data zoom
      if (spec.dataZoom) {
        spec.dataZoom = spec.dataZoom.map((dz: Record<string, unknown>) => ({
          ...dz,
          disabled: true,
        }))
      }
    }

    // If theme is null (--plot-theme: none), use spec as-is
    if (!theme) {
      chartInstance.setOption(spec, true)
      return chartInstance
    }

    // Apply theme to spec
    const themeOptions = toEchartsOptionsBase(theme)

    // If visualMap exists (for heatmaps), inject ramp colors if not specified
    let visualMap = spec.visualMap
    if (visualMap && !visualMap.inRange?.color) {
      visualMap = {
        ...visualMap,
        inRange: {
          ...visualMap.inRange,
          color: [theme.ramp.start, theme.ramp.end],
        },
      }
    }

    // Merge theme with user spec, preserving axis styling
    // ECharts supports arrays of axes (e.g., multiple x-axes). This helper applies
    // the theme to each axis in the array, rather than replacing the entire array.
    const mergeAxis = (
      themeAxis: Record<string, unknown>,
      userAxis: Record<string, unknown> | Record<string, unknown>[]
    ) => {
      if (Array.isArray(userAxis)) {
        return userAxis.map((axis) => deepMerge(themeAxis, axis))
      }
      return deepMerge(themeAxis, userAxis)
    }
    const mergedOptions = {
      ...themeOptions,
      ...spec,
      xAxis: mergeAxis(themeOptions.xAxis as Record<string, unknown>, spec.xAxis),
      yAxis: mergeAxis(themeOptions.yAxis as Record<string, unknown>, spec.yAxis),
      legend: spec.legend
        ? deepMerge(themeOptions.legend as Record<string, unknown>, spec.legend)
        : themeOptions.legend,
      ...(visualMap && { visualMap }),
    }

    // Apply default symbol size and shape to scatter/line series if not specified
    if (mergedOptions.series) {
      const seriesArray = Array.isArray(mergedOptions.series)
        ? mergedOptions.series
        : [mergedOptions.series]

      mergedOptions.series = seriesArray.map((series: Record<string, unknown>, i: number) => {
        const updatedSeries: Record<string, unknown> = { ...series }

        // Apply symbolSize and symbol to scatter charts and line charts with symbols
        if (
          series.type === 'scatter' ||
          series.type === 'effectScatter' ||
          (series.type === 'line' && series.showSymbol !== false)
        ) {
          const shape = theme.shapes[i % theme.shapes.length]
          updatedSeries.symbolSize = series.symbolSize ?? theme.mark.pointSize
          updatedSeries.symbol = series.symbol ?? mapShapeToECharts(shape, theme.mark.pointOpacity)

          // Apply opacity to itemStyle when pointOpacity > 0
          if (theme.mark.pointOpacity > 0) {
            updatedSeries.itemStyle = {
              ...((series.itemStyle as Record<string, unknown>) || {}),
              opacity: theme.mark.pointOpacity,
            }
          }
        }

        // Apply lineType to line charts
        if (series.type === 'line') {
          const lineType = theme.lineTypes[i % theme.lineTypes.length]
          updatedSeries.lineStyle = {
            ...((series.lineStyle as Record<string, unknown>) || {}),
            type: (series.lineStyle as any)?.type ?? mapLineTypeToECharts(lineType),
          }
        }

        return updatedSeries
      })
    }

    // Set the merged options
    chartInstance.setOption(mergedOptions, true)

    return chartInstance
  } catch (error) {
    onError(error)
    return null
  }
}

/**
 * Render ECharts container
 */
export function renderEChartsContainer() {
  const containerStyles = css`
    & {
      width: 100%;
      aspect-ratio: var(--plot-aspect-ratio);
      min-height: var(--plot-height-min);
      max-height: var(--plot-height-max);
    }
  `
  return html`
    <div slot="content" class="overflow-x-auto">
      <div class=${containerStyles} id="stencila-echarts-container"></div>
    </div>
  `
}
