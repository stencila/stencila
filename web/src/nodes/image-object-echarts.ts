import { css } from '@twind/core'
import { html } from 'lit'

import { buildPlotTheme, PlotTokens } from '../utilities/plotTheme'

/**
 * Convert plot tokens to ECharts option base
 */
function toEchartsOptionsBase(t: PlotTokens): any {
  const axisBase = {
    axisLabel: { color: t.textColor, fontSize: t.fontSize, fontFamily: t.fontFamily },
    nameTextStyle: { color: t.axis.titleColor, fontSize: t.axisTitleSize, fontFamily: t.fontFamily },
    axisLine: {
      show: true,
      lineStyle: { color: t.axis.lineColor, width: t.axis.lineWidth },
    },
    axisTick: {
      show: true,
      length: t.axis.tickSize,
      lineStyle: { color: t.axis.tickColor, width: t.axis.tickWidth },
    },
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
      borderColor: t.panelBorderColor,
      borderWidth: t.panelBorderWidth,
      // Using padding tokens makes margins too narrow
      left: t.padding.left,
      right: t.padding.right,
      top: t.padding.top,
      bottom: t.padding.bottom,
      containLabel: true,
    },
    xAxis: {
      ...axisBase,
      splitLine: {
        show: t.axis.gridXWidth > 0,
        lineStyle: {
          color: t.axis.gridColor,
          width: t.axis.gridXWidth,
          type: t.axis.gridDash > 0 ? 'dashed' : 'solid',
        },
      },
    },
    yAxis: {
      ...axisBase,
      splitLine: {
        show: t.axis.gridYWidth > 0,
        lineStyle: {
          color: t.axis.gridColor,
          width: t.axis.gridYWidth,
          type: t.axis.gridDash > 0 ? 'dashed' : 'solid',
        },
      },
    },
    legend: {
      backgroundColor: t.legend.background,
      textStyle: { color: t.legend.textColor, fontSize: t.legendSize },
      itemWidth: t.legend.markerSize,
      itemHeight: t.legend.markerSize,
      borderColor: t.legend.borderColor,
      borderWidth: t.legend.borderWidth,
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
          axisLine: { ...themeAxis.axisLine, ...axis.axisLine },
          axisTick: { ...themeAxis.axisTick, ...axis.axisTick },
          splitLine: { ...themeAxis.splitLine, ...axis.splitLine },
        }))
      }

      // Single axis - deep merge axis components
      return {
        ...themeAxis,
        ...userAxis,
        axisLabel: { ...themeAxis.axisLabel, ...userAxis.axisLabel },
        axisLine: { ...themeAxis.axisLine, ...userAxis.axisLine },
        axisTick: { ...themeAxis.axisTick, ...userAxis.axisTick },
        splitLine: { ...themeAxis.splitLine, ...userAxis.splitLine },
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
