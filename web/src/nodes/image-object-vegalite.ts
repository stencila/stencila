import { html } from 'lit'

/**
 * Compile and render a Vega-Lite visualization
 */
export async function compileVegaLite(
  contentUrl: string,
  container: HTMLElement,
  isStaticMode: boolean,
  onError: (error: Error) => void
): Promise<void> {
  const { default: vegaEmbed } = await import('vega-embed')
  const spec = JSON.parse(contentUrl)

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

  vegaEmbed(container, spec, embedOptions).catch((error) => {
    onError(error)
  })
}

/**
 * Render Vega-Lite container
 */
export function renderVegaLiteContainer() {
  return html`
    <div slot="content" class="overflow-x-auto">
      <div id="stencila-vega-container"></div>
    </div>
  `
}
