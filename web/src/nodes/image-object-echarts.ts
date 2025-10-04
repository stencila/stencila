import { html } from 'lit'
import { css } from '@twind/core'

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

    // Set the chart options
    chartInstance.setOption(spec)

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
