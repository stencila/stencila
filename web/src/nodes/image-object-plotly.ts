import { html } from 'lit'

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
      : spec.config

    await Plotly.react(container, spec.data, spec.layout, config)

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
  return html`
    <style id="plotly-css"></style>
    <div slot="content" class="overflow-x-auto">
      <div id="stencila-plotly-container" class="w-full"></div>
    </div>
  `
}
