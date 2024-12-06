import { css } from '@twind/core'
import { html, PropertyValues } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'
import { unsafeSVG } from 'lit/directives/unsafe-svg'

import 'plotly.js-dist-min/'

import { withTwind } from '../twind'

import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/cards/inline-on-demand'
import '../ui/nodes/properties/authors'

import { Entity } from './entity'
import { ExecutionMessage } from './execution-message'

/**
 * Web component representing a Stencila Schema `ImageObject` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/image-object.md
 */
@customElement('stencila-image-object')
@withTwind()
export class ImageObject extends Entity {
  /**
   * The media (MIME) type of the image
   */
  @property({ attribute: 'media-type' })
  mediaType?: string

  /**
   * The code content of the image
   *
   * For binary images (e.g. PNG, JPG) this should be empty and instead
   * there will be an <img> element in the <slot>. For code-based, rendered
   * images (e.g. Mermaid, Vega) this will be the code that needs to be
   * rendered into an image (see methods below).
   */
  @property()
  content?: string

  /**
   * The rendered SVG of the content, if applicable
   */
  @state()
  private svg?: string

  /**
   * Any error message generated while attempting to render the content
   */
  @state()
  private error?: string

  private clearCodeChunkMessages(codeChunk: HTMLElement) {
    // Clear any existing messages
    const messages = codeChunk.querySelector('div[slot=messages]')
    if (messages) {
      while (messages.firstChild) {
        messages.removeChild(messages.firstChild)
      }
    }
  }

  private addCodeChunkErrorMessage(
    element: HTMLElement,
    message: string,
    codeLocation?: number[]
  ) {
    element.setAttribute('level', 'Error')
    element.setAttribute('error-type', 'ParseError')

    element.setAttribute('message', message)
    if (codeLocation) {
      element.setAttribute('code-location', `[${codeLocation.join(',')}]`)
    }
  }

  override async update(properties: PropertyValues) {
    super.update(properties)

    if (properties.has('content') || properties.has('mediaType')) {
      if (this.content) {
        if (this.mediaType == 'text/vnd.mermaid') {
          await this.compileMermaid()
        } else if (this.mediaType == 'application/vnd.plotly.v1+json') {
          await this.compilePlotly()
        } else if (this.mediaType == 'application/vnd.vegalite.v5+json') {
          await this.compileVegaLite()
        }
      }
    }
  }

  private async compileMermaid() {
    // Import Mermaid dynamically, when it is required, rather than have
    // it bundled into the main JS file for the view
    const { default: mermaid } = await import('mermaid')

    const container = document.createElement('div')
    document.body.appendChild(container)

    let codeChunk
    if (this.ancestors.endsWith('CodeChunk')) {
      codeChunk = this.closestGlobally('stencila-code-chunk')
      this.clearCodeChunkMessages(codeChunk)
    }

    try {
      const id = 'stencila-' + Math.random().toString(36).substring(2)
      this.svg = (await mermaid.render(id, this.content, container)).svg
    } catch (error) {
      // Hide the container so that the Mermaid bomb message does not appear
      container.style.display = 'none'

      if (codeChunk) {
        // Get the messages slot, adding one if necessary
        let messages = codeChunk.querySelector('div[slot=messages]')
        if (!messages) {
          messages = document.createElement('div')
          messages.setAttribute('slot', 'execution-messages')
          codeChunk.appendChild(messages)
        }

        // Add the message
        const message = document.createElement(
          'stencila-execution-message'
        ) as ExecutionMessage

        const expected = error.hash?.expected
        let str: string
        if (expected) {
          str = `expected ${expected.join(', ')}`
        } else {
          str = error.message ?? error.toString()
          str = str.slice(str.lastIndexOf('-^\n')).trim()
        }

        const loc = error.hash?.loc
        let codeLocation
        if (loc) {
          const startLine = (loc.first_line ?? 1) - 1
          const startCol = (loc.first_column ?? 0) + 1
          const endLine = (loc.last_line ?? 1) - 1
          const endCol = (loc.last_column ?? 0) + 1
          codeLocation = [startLine, startCol, endLine, endCol]
        }
        this.addCodeChunkErrorMessage(message, str, codeLocation)
        messages.appendChild(message)
      } else {
        // Otherwise, render a <pre> element with error
        this.error = error.message ?? error.toString()
      }
    }

    container.remove()
  }

  /**
   * Recieve a plotly spec from and render in preview with plotly.js
   */
  private async compilePlotly() {
    const Plotly = await import('plotly.js-dist-min')
    const spec = JSON.parse(this.content)

    // create hidden container for initial plotly render
    const container = this.shadowRoot.getElementById(
      'stencila-plotly-container'
    )

    let codeChunk
    if (this.ancestors.endsWith('CodeChunk')) {
      codeChunk = this.closestGlobally('stencila-code-chunk')
      this.clearCodeChunkMessages(codeChunk)
    }

    try {
      await Plotly.newPlot(container, spec.data, spec.layout, spec.config)

      const styleTags = Array.from(
        document.head.getElementsByTagName('style')
      ).filter((tag) => {
        return tag.id.startsWith('plotly.js')
      })

      let style = ''
      styleTags.forEach((tag) => {
        const sheet = tag.sheet
        Array.from(sheet.cssRules).forEach((rule) => {
          style += rule.cssText + '\n'
        })
      })
      style += '.plotly .modebar-btn { display: inline-block; }'

      const shadowStyle = this.shadowRoot.getElementById('plotly-css')
      shadowStyle.innerText = style

      // creates encoded svg string, width and hieght are required
      // const svgString = await Plotly.toImage(container, {
      //   format: 'svg',
      //   width: 600,
      //   height: 600,
      // })

      // // decode the svg string, removing the leading data URI string
      // const decodedSvgString = decodeURIComponent(svgString.split(',')[1])

      // const parser = new DOMParser()
      // const svgDocument = parser.parseFromString(
      //   decodedSvgString,
      //   'image/svg+xml'
      // )

      // const svgEl = svgDocument.documentElement

      // // remove preset dimensions for responsive rendering
      // svgEl.removeAttribute('width')
      // svgEl.removeAttribute('height')

      // this.svg = svgEl.outerHTML
      // container.remove()
    } catch (error) {
      if (codeChunk) {
        let messages = codeChunk.querySelector('div[slot=messages]')
        if (!messages) {
          messages = document.createElement('div')
          messages.setAttribute('slot', 'execution-messages')
          codeChunk.appendChild(messages)
        }

        // TODO check error structure for plotly.js and add more fields if needed
        // Add the message
        const message = document.createElement(
          'stencila-execution-message'
        ) as ExecutionMessage

        let str
        str = error.message ?? error.toString()
        str = str.slice(str.lastIndexOf('-^\n')).trim()

        this.addCodeChunkErrorMessage(message, str)

        messages.appendChild(message)
      }
    }
  }

  private async compileVegaLite() {
    const { default: vegaEmbed } = await import('vega-embed')
    const spec = JSON.parse(this.content)

    // clear `CodeChunk` messages
    let codeChunk: HTMLElement

    if (this.ancestors.endsWith('CodeChunk')) {
      codeChunk = this.closestGlobally('stencila-code-chunk')
      this.clearCodeChunkMessages(codeChunk)
    }
    // attach to shadow dom container element
    const container = this.shadowRoot.querySelector(
      'div#stencila-vega-container'
    ) as HTMLElement

    // embed the figure as svg
    vegaEmbed(container, spec, {
      renderer: 'svg',
      actions: false,
      mode: 'vega-lite',
    }).catch((error) => {
      if (codeChunk) {
        let messages = codeChunk.querySelector('div[slot=messages]')
        if (!messages) {
          messages = document.createElement('div')
          messages.setAttribute('slot', 'execution-messages')
          codeChunk.appendChild(messages)
        }

        // TODO check error structure for vega-lite/embed and get code location if possible
        const message = document.createElement(
          'stencila-execution-message'
        ) as ExecutionMessage

        let str
        str = error.message ?? error.toString()
        str = str.slice(str.lastIndexOf('-^\n')).trim()

        this.addCodeChunkErrorMessage(message, str)

        messages.appendChild(message)
      }
    })
  }

  override render() {
    if (this.ancestors.includes('StyledBlock')) {
      return this.renderContent()
    }

    return this.ancestors.endsWith('CodeChunk')
      ? this.renderBlockOnDemand()
      : this.renderInlineOnDemand()
  }

  private renderBlockOnDemand() {
    return html`
      <stencila-ui-block-on-demand type="ImageObject">
        ${this.renderContent()}
      </stencila-ui-block-on-demand>
    `
  }

  private renderInlineOnDemand() {
    return html`
      <stencila-ui-inline-on-demand type="ImageObject">
        <div slot="body">
          <stencila-ui-node-authors type="ImageObject">
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
        </div>
        ${this.renderContent()}
      </stencila-ui-inline-on-demand>
    `
  }

  private renderContent() {
    if (this.error) {
      return this.renderErrors()
    }

    // render vega plot
    if (this.mediaType === 'application/vnd.vegalite.v5+json') {
      return this.renderVega()
    }

    if (this.mediaType === 'application/vnd.plotly.v1+json') {
      return this.renderPlotly()
    }

    return this.svg ? this.renderSvg() : this.renderImg()
  }

  private renderErrors() {
    return html`<div slot="content">
      <pre class="whitespace-pre overflow-x-auto"><code>${this
        .error}</code></pre>
    </div>`
  }

  private renderSvg() {
    /**
     * Reset styles on SVG
     *
     * Sets all properties to their default values as defined by the CSS specification.
     * This effectively strips away any inherited styles or previously applied styles,
     * resetting the SVG to its most basic, unstyled state.
     *
     * We do this to prevent inherited properties (e.g. line-height) from the current
     * theme for the document.
     */
    const svgStyles = css`
      & svg {
        all: initial;
      }
    `

    return html`
      <div slot="content" class=${svgStyles}>${unsafeSVG(this.svg)}</div>
    `
  }

  private renderImg() {
    const imgStyles = css`
      & img {
        width: 100%;
      }
    `
    return html`
      <div slot="content" class=${imgStyles}>
        <slot></slot>
      </div>
    `
  }

  private renderVega() {
    return html`
      <div slot="content">
        <div id="stencila-vega-container"></div>
      </div>
    `
  }

  private renderPlotly() {
    return html`
      <style id="plotly-css"></style>
      <div slot="content" class="overflow-x-auto">
        <div id="stencila-plotly-container" class="w-full"></div>
      </div>
    `
  }
}
