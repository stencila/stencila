import { css } from '@twind/core'
import { PropertyValues, html } from 'lit'
import { customElement, state } from 'lit/decorators'

import { withTwind } from '../twind'

import { ExecutionMessage } from './execution-message'
import {
  compileCytoscape,
  renderCytoscapeContainer,
} from './image-object-cytoscape'
import { compileECharts, renderEChartsContainer } from './image-object-echarts'
import { compileLeaflet, renderLeafletIframe } from './image-object-leaflet'
import { compileMermaid, renderMermaid } from './image-object-mermaid'
import { compilePlotly, renderPlotlyContainer } from './image-object-plotly'
import { MEDIA_TYPES, imageObjectStyles } from './image-object-shared'
import {
  compileVegaLite,
  renderVegaLiteContainer,
} from './image-object-vegalite'
import { MediaObject } from './media-object'

/**
 * Web component representing a Stencila Schema `ImageObject` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/image-object.md
 */
@customElement('stencila-image-object')
@withTwind()
export class ImageObject extends MediaObject {
  /**
   * Shared styles for visualization containers
   */
  static override styles = imageObjectStyles

  /**
   * Image media types that are rendered in the browser
   */
  static MEDIA_TYPES = MEDIA_TYPES

  /**
   * The Cytoscape.js instance
   *
   * Rather than import cytoscape.Core just stub out what we need. This avoids
   * accidental bloat of the bundle if cytoscape is statically imported.
   */
  @state()
  private cytoscape?: { resize: () => void; destroy: () => void }

  /**
   * The ECharts instance
   *
   * Rather than import echarts types just stub out what we need. This avoids
   * accidental bloat of the bundle if echarts is statically imported.
   */
  @state()
  private echarts?: { resize: () => void; dispose: () => void }

  /**
   * The processed Leaflet map content
   */
  @state()
  private leaflet?: string

  /**
   * The rendered Mermaid SVG
   */
  @state()
  private mermaid?: string

  /**
   * The Vega-Lite embed result
   *
   * Rather than import vega-embed types just stub out what we need. This avoids
   * accidental bloat of the bundle if vega-embed is statically imported.
   */
  @state()
  private vegaLite?: { finalize: () => void }

  override needsCompiling(): boolean {
    // @ts-expect-error re media type
    return Object.values(ImageObject.MEDIA_TYPES).includes(this.mediaType)
  }

  /**
   * Setup code chunk error handling if this image is in a code chunk
   * @returns The code chunk element if found, undefined otherwise
   */
  private setupCodeChunkErrorHandling(): HTMLElement | undefined {
    if (this.parentNodeIs('CodeChunk')) {
      const codeChunk = this.closestGlobally('stencila-code-chunk')
      this.clearCodeChunkMessages(codeChunk)
      return codeChunk
    }
    return undefined
  }

  /**
   * Clear all existing messages from a code chunk
   * @param codeChunk The code chunk element to clear messages from
   */
  private clearCodeChunkMessages(codeChunk: HTMLElement) {
    // Clear any existing messages
    const messages = codeChunk.querySelector('div[slot=messages]')
    if (messages) {
      while (messages.firstChild) {
        messages.removeChild(messages.firstChild)
      }
    }
  }

  /**
   * Set error attributes on an execution message element
   * @param element The execution message element to configure
   * @param message The error message text
   * @param codeLocation Optional code location array [startLine, startCol, endLine, endCol]
   */
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

  /**
   * Get or create the messages slot div for a code chunk
   * @param codeChunk The code chunk element
   * @returns The messages div element
   */
  private getOrCreateMessagesSlot(codeChunk: HTMLElement): Element {
    let messages = codeChunk.querySelector('div[slot=messages]')
    if (!messages) {
      messages = document.createElement('div')
      messages.setAttribute('slot', 'execution-messages')
      codeChunk.appendChild(messages)
    }
    return messages
  }

  /**
   * Add an error message to a code chunk
   * @param codeChunk The code chunk element
   * @param errorString The error message string
   * @param codeLocation Optional code location array [startLine, startCol, endLine, endCol]
   */
  private addErrorMessage(
    codeChunk: HTMLElement,
    errorString: string,
    codeLocation?: number[]
  ): void {
    const messages = this.getOrCreateMessagesSlot(codeChunk)
    const message = document.createElement(
      'stencila-execution-message'
    ) as ExecutionMessage

    this.addCodeChunkErrorMessage(message, errorString, codeLocation)
    messages.appendChild(message)
  }

  /**
   * Format an error object to a readable string
   * @param error The error object
   * @returns Formatted error string
   */
  private formatErrorString(error: Error | { message?: string; toString: () => string }): string {
    let str = error.message ?? error.toString()
    const lastCaret = str.lastIndexOf('-^\n')
    if (lastCaret !== -1) {
      str = str.slice(lastCaret).trim()
    }
    return str
  }

  private onResize = () => {
    // If this component is resized then resize as needed
    this.cytoscape?.resize()
    this.echarts?.resize()
  }

  private onThemeChange = async () => {
    // Re-compile if necessary
    if (this.cytoscape && this.contentUrl) {
      await this.compileCytoscape()
    } else if (this.echarts && this.contentUrl) {
      await this.compileECharts()
    } else if (this.mermaid && this.contentUrl) {
      await this.compileMermaid()
    } else if (this.mediaType === ImageObject.MEDIA_TYPES.plotly && this.contentUrl) {
      await this.compilePlotly()
    } else if (this.vegaLite && this.contentUrl) {
      await this.compileVegaLite()
    }

    // Trigger Lit re-render
    this.requestUpdate()
  }

  override connectedCallback() {
    super.connectedCallback()
    window.addEventListener('resize', this.onResize)
    window.addEventListener('stencila-color-scheme-changed', this.onThemeChange)
    window.addEventListener('stencila-theme-changed', this.onThemeChange)
  }

  override disconnectedCallback() {
    window.removeEventListener('resize', this.onResize)
    window.removeEventListener(
      'stencila-color-scheme-changed',
      this.onThemeChange
    )
    window.removeEventListener('stencila-theme-changed', this.onThemeChange)

    // Dispose of visualization library instances
    if (this.cytoscape) {
      this.cytoscape.destroy()
    }

    if (this.echarts) {
      this.echarts.dispose()
    }

    if (this.vegaLite) {
      this.vegaLite.finalize()
    }

    super.disconnectedCallback()
  }

  override async updated(properties: PropertyValues) {
    super.updated(properties)

    if (properties.has('contentUrl') || properties.has('mediaType')) {
      if (!this.contentUrl) {
        return
      }

      switch (this.mediaType) {
        case ImageObject.MEDIA_TYPES.cytoscape:
          return await this.compileCytoscape()
        case ImageObject.MEDIA_TYPES.echarts:
          return await this.compileECharts()
        case ImageObject.MEDIA_TYPES.leaflet:
          return await this.compileLeaflet()
        case ImageObject.MEDIA_TYPES.mermaid:
          return await this.compileMermaid()
        case ImageObject.MEDIA_TYPES.plotly:
          return await this.compilePlotly()
        case ImageObject.MEDIA_TYPES.vegaLite:
          return await this.compileVegaLite()
        default:
          return
      }
    }
  }

  private async compileCytoscape() {
    const container = this.shadowRoot.querySelector(
      'div#stencila-cytoscape-container'
    ) as HTMLElement
    const isStaticView = this.documentView() == 'static'

    this.cytoscape = await compileCytoscape(
      this.contentUrl,
      container,
      isStaticView
    ) as ImageObject['cytoscape']
  }

  private async compileECharts() {
    const container = this.shadowRoot.getElementById(
      'stencila-echarts-container'
    )
    const codeChunk = this.setupCodeChunkErrorHandling()
    const isStaticView = this.documentView() == 'static'

    this.echarts = await compileECharts(
      this.contentUrl,
      container,
      this.echarts,
      isStaticView,
      (error) => {
        if (codeChunk) {
          this.addErrorMessage(codeChunk, this.formatErrorString(error))
        } else {
          this.error = error.message ?? error.toString()
        }
      }
    )
  }

  private async compileLeaflet() {
    const codeChunk = this.setupCodeChunkErrorHandling()

    await compileLeaflet(
      this.contentUrl,
      (html) => {
        this.leaflet = html
        this.error = undefined
      },
      (error) => {
        if (codeChunk) {
          this.addErrorMessage(codeChunk, error)
        } else {
          this.error = error
        }
      }
    )
  }

  private async compileMermaid() {
    const codeChunk = this.setupCodeChunkErrorHandling()

    await compileMermaid(
      this.contentUrl,
      this,
      (svg) => {
        this.mermaid = svg
        this.error = undefined
      },
      (error) => {
        if (codeChunk) {
          const expected = error.hash?.expected
          let str: string
          if (expected) {
            str = `expected ${expected.join(', ')}`
          } else {
            str = this.formatErrorString(error)
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

          this.addErrorMessage(codeChunk, str, codeLocation)
        } else {
          this.error = error.message ?? error.toString()
        }
      }
    )
  }

  private async compilePlotly() {
    const container = this.shadowRoot.getElementById(
      'stencila-plotly-container'
    )
    const codeChunk = this.setupCodeChunkErrorHandling()
    const isStaticView = this.documentView() == 'static'

    await compilePlotly(
      this.contentUrl,
      container,
      this.shadowRoot,
      isStaticView,
      (error) => {
        if (codeChunk) {
          this.addErrorMessage(codeChunk, this.formatErrorString(error))
        }
      }
    )
  }

  private async compileVegaLite() {
    const container = this.shadowRoot.querySelector(
      'div#stencila-vega-container'
    ) as HTMLElement
    const codeChunk = this.setupCodeChunkErrorHandling()
    const isStaticView = this.documentView() == 'static'

    this.vegaLite = await compileVegaLite(this.contentUrl, container, isStaticView, (error) => {
      if (codeChunk) {
        this.addErrorMessage(codeChunk, this.formatErrorString(error))
      }
    })
  }

  override renderMediaElem() {
    const spanStyles = css`
      & img,
      & ::slotted(img) {
        display: inline;
        max-height: 1.2em;
        vertical-align: middle;
      }
    `
    return this.mediaSrc ?
      html`<span class=${spanStyles}><img src=${this.mediaSrc} /></span>` :
      html`<span class=${spanStyles}><slot></slot></span>`
  }

  override renderCardContent() {
    if (this.mediaType === ImageObject.MEDIA_TYPES.mermaid) {
      return html`<div slot="content">${renderMermaid(this.mermaid)}</div>`
    }

    if (this.mediaType === ImageObject.MEDIA_TYPES.cytoscape) {
      return html`<div slot="content">${renderCytoscapeContainer()}</div>`
    }

    if (this.mediaType === ImageObject.MEDIA_TYPES.plotly) {
      return html`<div slot="content">${renderPlotlyContainer()}</div>`
    }

    if (this.mediaType === ImageObject.MEDIA_TYPES.vegaLite) {
      return html`<div slot="content">${renderVegaLiteContainer()}</div>`
    }

    if (this.mediaType === ImageObject.MEDIA_TYPES.echarts) {
      return html`<div slot="content">${renderEChartsContainer()}</div>`
    }

    if (this.mediaType === ImageObject.MEDIA_TYPES.leaflet) {
      return renderLeafletIframe(this.leaflet)
    }

    return this.renderImg()
  }

  private renderImg() {
    const imgStyles = css`
      & {
        display: block;
        max-width: 100%;
        height: auto;
        margin: 1rem auto;
      }
    `
    return html`
      <div slot="content">
        ${this.mediaSrc ?
          html`<img class=${imgStyles} src=${this.mediaSrc} />` :
          html`<slot></slot>`}
        <div>
          <slot name="caption"></slot>
        </div>
      </div>
    `
  }
}
