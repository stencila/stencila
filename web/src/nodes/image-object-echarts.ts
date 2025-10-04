import { html } from 'lit'
import { css } from '@twind/core'
import { buildPlotTheme, PlotTokens } from '../utilities/plotTheme'

/**
 * Convert plot tokens to ECharts option base
 */
function toEchartsOptionsBase(t: PlotTokens): any {
  //console.log(JSON.stringify(t, null, "  "))

  const axisBase = {
    axisLabel: { color: t.textColor, fontSize: t.fontSize, fontFamily: t.fontFamily },
    axisLine: {
      lineStyle: { color: t.axis.lineColor, width: t.axis.lineWidth },
    },
    axisTick: {
      length: t.axis.tickSize,
      lineStyle: { color: t.axis.tickColor, width: t.axis.tickWidth },
    },
    splitLine: {
      show: true,
      lineStyle: {
        color: t.axis.gridColor,
        width: t.axis.gridWidth,
        type: t.axis.gridDash > 0 ? 'dashed' : 'solid',
      },
    },
  }

  return {
    color: t.palette,
    backgroundColor: t.panel,
    textStyle: { color: t.textColor, fontFamily: t.fontFamily, fontSize: t.fontSize },
    grid: {
      left: t.padding.left,
      right: t.padding.right,
      top: t.padding.top,
      bottom: t.padding.bottom,
      containLabel: true,
    },
    xAxis: axisBase,
    yAxis: axisBase,
    legend: {
      backgroundColor: t.legend.background,
      textStyle: { color: t.legend.textColor, fontSize: t.legendSize },
      itemWidth: t.legend.markerSize,
      itemHeight: t.legend.markerSize,
      borderColor: t.legend.borderColor,
      // Do not set border properties because otherwise get a small box at
      // bottom of chart even when legend is empty. Instead, if users want
      // a border they need to specify it in their spec
      // borderWidth: t.legend.borderWidth,
    },
    tooltip: {
      trigger: 'item',
      backgroundColor: t.tooltip.background,
      borderColor: t.tooltip.borderColor,
      borderWidth: t.tooltip.borderWidth,
      textStyle: { color: t.tooltip.textColor, fontFamily: t.fontFamily, fontSize: t.fontSize },
      padding: [t.tooltip.padY, t.tooltip.padX, t.tooltip.padY, t.tooltip.padX],
    },
    animationDuration: t.motion.duration,
    animationEasing: t.motion.ease,
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
    const themeOptions = toEchartsOptionsBase(theme)

    // Configure for static mode if enabled
    if (isStaticMode) {
      // Disable interactions in static mode
      if (!spec.toolbox) {
        spec.toolbox = {}
      }
      spec.toolbox.show = false

      // Disable zoom and data zoom
      if (spec.dataZoom) {
        spec.dataZoom = spec.dataZoom.map((dz: any) => ({
          ...dz,
          disabled: true,
        }))
      }
    }

    // Deep merge axes to preserve theme styling when user provides axis config
    const mergeAxis = (themeAxis: any, userAxis: any) => {
      if (!userAxis) return themeAxis

      // Handle array of axes
      if (Array.isArray(userAxis)) {
        return userAxis.map((axis: any) => ({
          ...themeAxis,
          ...axis,
          axisLabel: { ...themeAxis.axisLabel, ...axis.axisLabel },
        }))
      }

      // Single axis - deep merge axisLabel
      return {
        ...themeAxis,
        ...userAxis,
        axisLabel: { ...themeAxis.axisLabel, ...userAxis.axisLabel },
      }
    }

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
    const mergedOptions = {
      ...themeOptions,
      ...spec,
      xAxis: mergeAxis(themeOptions.xAxis, spec.xAxis),
      yAxis: mergeAxis(themeOptions.yAxis, spec.yAxis),
      ...(visualMap && { visualMap }),
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
      height: 400px;
    }
  `
  return html`
    <div slot="content" class="overflow-x-auto">
      <div class=${containerStyles} id="stencila-echarts-container"></div>
    </div>
  `
}
