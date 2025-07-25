import { apply, css } from '@twind/core'
import { type Core as CytoscapeCore } from 'cytoscape'
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
  static MEDIA_TYPES = {
    cytoscape: 'application/vnd.cytoscape.v3+json',
    mermaid: 'text/vnd.mermaid',
    plotly: 'application/vnd.plotly.v1+json',
    vegaLite: 'application/vnd.vegalite.v5+json',
    htmlMap: 'text/html',
  } as const

  /**
   * The media (MIME) type of the image
   */
  @property({ attribute: 'media-type' })
  mediaType?: string

  /**
   * The URL of the source of the image
   *
   * For HTTP and file URLs, equivalent to the `src` attribute in HTML. See `renderImg` for how this is used
   * to rewrite the URL if necessary, depending upon the context.
   *
   * For code-based, rendered images (e.g. Mermaid, Vega) this will be the code that needs to be
   * rendered into an image (see methods below).
   */
  @property({ attribute: 'content-url' })
  contentUrl?: string

  /**
   * The resolved URL of the <imd> `src` attribute, if applicable
   */
  @state()
  imgSrc?: string

  /**
   * The rendered SVG of the content, if applicable
   */
  @state()
  private svg?: string

  /**
   * The processed HTML map content, if applicable
   */
  @state()
  private htmlMapContent?: string

  /**
   * The Cytoscape.js instance (if relevant)
   */
  private cytoscape?: CytoscapeCore

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

  private onResize() {
    // If this component is resized then resize the Cytoscape instance, if any
    this.cytoscape?.resize()
  }

  override connectedCallback() {
    super.connectedCallback()
    window.addEventListener('resize', this.onResize)
  }

  override disconnectedCallback() {
    window.removeEventListener('resize', this.onResize)
    super.disconnectedCallback()
  }

  override async updated(properties: PropertyValues) {
    super.updated(properties)

    if (properties.has('contentUrl') || properties.has('mediaType')) {
      if (!this.contentUrl) {
        return
      }

      if (this.mediaType == ImageObject.MEDIA_TYPES.cytoscape) {
        await this.compileCytoscape()
      } else if (this.mediaType == ImageObject.MEDIA_TYPES.mermaid) {
        await this.compileMermaid()
      } else if (this.mediaType == ImageObject.MEDIA_TYPES.plotly) {
        await this.compilePlotly()
      } else if (this.mediaType == ImageObject.MEDIA_TYPES.vegaLite) {
        await this.compileVegaLite()
      } else if (this.mediaType == ImageObject.MEDIA_TYPES.htmlMap) {
        await this.compileHtmlMap()
      } else if (this.contentUrl.startsWith('data:')) {
        // This should not occur, but if it does, do nothing
      } else {
        if (
          this.contentUrl.startsWith('https://') ||
          this.contentUrl.startsWith('https://')
        ) {
          // Use HTTP URLs directly
          this.imgSrc = this.contentUrl
        } else {
          // If file path, and in VSCode webview, then prefix a file path with workspace URI
          const workspace = this.closestGlobally(
            'stencila-vscode-view'
          )?.getAttribute('workspace')

          this.imgSrc = workspace
            ? `${workspace}/${this.contentUrl}`
            : this.contentUrl
        }

        // Prefetch to check that URL is valid
        const response = await fetch(this.imgSrc, { method: 'HEAD' })
        if (response.ok) {
          this.error = undefined
        } else {
          let src = this.contentUrl
          if (src.length > 40) {
            src = src.slice(0, 40) + '\u2026'
          }
          const message = await response.text()
          this.error = `Error fetching image '${src}': ${message}`
        }
      }
    }
  }

  private async compileCytoscape() {
    const { default: cytoscape } = await import('cytoscape')

    const graph = JSON.parse(this.contentUrl)
    graph.container = this.shadowRoot.querySelector(
      'div#stencila-cytoscape-container'
    ) as HTMLElement

    this.cytoscape = cytoscape(graph)
  }

  private async compileMermaid() {
    // Import Mermaid dynamically, when it is required, rather than have
    // it bundled into the main JS file for the view
    const { default: mermaid } = await import('mermaid')

    const container = document.createElement('div')
    document.body.appendChild(container)

    let codeChunk
    if (this.parentNodeIs('CodeChunk')) {
      codeChunk = this.closestGlobally('stencila-code-chunk')
      this.clearCodeChunkMessages(codeChunk)
    }

    try {
      const id = 'stencila-' + Math.random().toString(36).substring(2)
      this.svg = (await mermaid.render(id, this.contentUrl, container)).svg
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

  private async compilePlotly() {
    const Plotly = await import('plotly.js-dist-min')
    const spec = JSON.parse(this.contentUrl)

    // create hidden container for initial plotly render
    const container = this.shadowRoot.getElementById(
      'stencila-plotly-container'
    )

    let codeChunk
    if (this.parentNodeIs('CodeChunk')) {
      codeChunk = this.closestGlobally('stencila-code-chunk')
      this.clearCodeChunkMessages(codeChunk)
    }

    try {
      await Plotly.react(container, spec.data, spec.layout, spec.config)

      // find plotly,js dynamically generated style tags
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
      const shadowStyle = this.shadowRoot.getElementById('plotly-css')
      shadowStyle.innerText = style
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
    const spec = JSON.parse(this.contentUrl)

    // clear `CodeChunk` messages
    let codeChunk: HTMLElement

    if (this.parentNodeIs('CodeChunk')) {
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

  private async compileHtmlMap() {
    let codeChunk: HTMLElement

    if (this.parentNodeIs('CodeChunk')) {
      codeChunk = this.closestGlobally('stencila-code-chunk')
      this.clearCodeChunkMessages(codeChunk)
    }

    try {
      // Check if the HTML content contains Leaflet indicators
      const htmlContent = this.contentUrl
      const isLeafletMap = htmlContent.includes('leaflet') || 
                          htmlContent.includes('L.map') ||
                          htmlContent.includes('leaflet.js')

      if (isLeafletMap) {
        // Store the HTML content for rendering
        this.htmlMapContent = htmlContent
        this.error = undefined
      } else {
        this.error = 'HTML content does not appear to contain a valid map'
      }
    } catch (error) {
      if (codeChunk) {
        let messages = codeChunk.querySelector('div[slot=messages]')
        if (!messages) {
          messages = document.createElement('div')
          messages.setAttribute('slot', 'execution-messages')
          codeChunk.appendChild(messages)
        }

        const message = document.createElement(
          'stencila-execution-message'
        ) as ExecutionMessage

        let str = error.message ?? error.toString()
        str = str.slice(str.lastIndexOf('-^\n')).trim()

        this.addCodeChunkErrorMessage(message, str)
        messages.appendChild(message)
      } else {
        this.error = error.message ?? error.toString()
      }
    }
  }

  override render() {
    if (this.isWithin('StyledBlock') || this.isWithinUserChatMessage()) {
      return this.renderContent()
    }

    if (this.isWithinModelChatMessage()) {
      return this.renderCardWithChatAction()
    }

    return this.renderCard()
  }

  override renderCard() {
    return this.parentNodeIs('CodeChunk')
      ? this.renderBlockOnDemand()
      : this.renderInlineOnDemand()
  }

  private renderBlockOnDemand() {
    return html`
      <stencila-ui-block-on-demand
        type="ImageObject"
        node-id=${this.id}
        depth=${this.depth}
        ?has-root=${this.hasRoot()}
      >
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
      return this.renderError()
    }

    if (this.mediaType === ImageObject.MEDIA_TYPES.cytoscape) {
      return this.renderCytoscape()
    }

    if (this.mediaType === ImageObject.MEDIA_TYPES.plotly) {
      return this.renderPlotly()
    }

    if (this.mediaType === ImageObject.MEDIA_TYPES.vegaLite) {
      return this.renderVega()
    }

    if (this.mediaType === ImageObject.MEDIA_TYPES.htmlMap) {
      return this.renderHtmlMap()
    }

    return this.svg ? this.renderSvg() : this.renderImg()
  }

  private renderError() {
    const classes = apply(
      'overflow-x-auto px-2 py-1',
      'rounded border border-red-200 bg-red-100',
      'text-red-900 text-sm whitespace-pre'
    )

    return html`<div slot="content">
      <pre class=${classes}><code>${this.error}</code></pre>
    </div>`
  }

  private renderSvg() {
    /**
     * Reset specific inherited styles on SVG
     *
     * Only reset properties that commonly cause issues with SVGs when inherited
     * from the document, while preserving essential SVG styling.
     * 
     * Previously we used `all: initial;` to do this but that was too aggressive
     * and was breaking mermaid rending on Chrome
     */
    const svgStyles = css`
      & svg {
        /* Reset text-related inherited properties */
        line-height: 1;
        font-size: inherit;
        font-family: inherit;
        text-align: initial;
        letter-spacing: normal;
        word-spacing: normal;
        
        /* Reset box model properties that might interfere */
        margin: 0;
        padding: 0;
        border: none;
        
        /* Ensure SVG displays properly */
        display: block;
        max-width: 100%;
        height: auto;
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
        ${this.imgSrc ? html`<img src=${this.imgSrc} />` : html`<slot></slot>`}
      </div>
    `
  }

  private renderCytoscape() {
    const containerStyles = css`
      & {
        position: relative;
        width: 100%;
        aspect-ratio: 1;
      }
    `
    return html`
      <div slot="content" class="overflow-x-auto">
        <div class=${containerStyles} id="stencila-cytoscape-container"></div>
      </div>
    `
  }

  private renderVega() {
    return html`
      <div slot="content" class="overflow-x-auto">
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

  private renderHtmlMap() {
    const mapStyles = css`
      & iframe {
        width: 100%;
        height: 400px;
        border: none;
      }
    `

    // Create a blob URL for the HTML content to safely render in an iframe
    const blob = new Blob([this.htmlMapContent], { type: 'text/html' })
    const blobUrl = URL.createObjectURL(blob)

    return html`
      <div slot="content" class="overflow-x-auto">
        <div class=${mapStyles}>
          <iframe src=${blobUrl} sandbox="allow-scripts allow-same-origin"></iframe>
        </div>
      </div>
    `
  }
}
