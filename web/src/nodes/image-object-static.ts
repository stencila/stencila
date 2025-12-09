import { html, LitElement, PropertyValues } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'
import { unsafeSVG } from 'lit/directives/unsafe-svg.js'

import { compileCytoscape } from './image-object-cytoscape'
import { compileECharts } from './image-object-echarts'
import { compileLeaflet } from './image-object-leaflet'
import { compileMermaid } from './image-object-mermaid'
import { compilePlotly } from './image-object-plotly'
import { imageObjectStyles, MEDIA_TYPES } from './image-object-shared'
import { compileVegaLite } from './image-object-vegalite'

/**
 * Static version of the ImageObject web component
 *
 * This is a lightweight alternative for static views that extends LitElement
 * directly (not MediaObject) and uses plain CSS instead of Twind.
 *
 * It supports:
 * - Regular images (via <img> or <slot>)
 * - JS-based visualizations (Mermaid, Plotly, ECharts, Vega-Lite, Cytoscape, Leaflet)
 *
 * It excludes:
 * - Resize/theme change listeners (fixed at render time)
 * - Block-on-demand UI wrapper
 * - Code chunk error handling
 * - MediaObject/Entity inheritance chain
 */
@customElement('stencila-image-object')
export class ImageObjectStatic extends LitElement {
  /**
   * The media type of the image/visualization
   */
  @property({ attribute: 'media-type' })
  mediaType?: string

  /**
   * The content URL (data URI or URL)
   */
  @property({ attribute: 'content-url' })
  contentUrl?: string

  /**
   * The rendered Mermaid SVG
   */
  @state()
  private mermaid?: string

  /**
   * The processed Leaflet HTML content
   */
  @state()
  private leaflet?: string

  /**
   * Error message for failed rendering
   */
  @state()
  private error?: string

  /**
   * Instance references for cleanup
   */
  private cytoscapeInstance?: { destroy: () => void }
  private echartsInstance?: { resize: () => void; dispose: () => void }
  private vegaLiteInstance?: { finalize: () => void }

  /**
   * Shared styles for visualization containers
   */
  static override styles = imageObjectStyles

  override disconnectedCallback() {
    // Dispose of visualization library instances
    this.cytoscapeInstance?.destroy()
    this.echartsInstance?.dispose()
    this.vegaLiteInstance?.finalize()

    super.disconnectedCallback()
  }

  override async updated(properties: PropertyValues) {
    super.updated(properties)

    if (properties.has('contentUrl') || properties.has('mediaType')) {
      if (!this.contentUrl) {
        return
      }

      this.error = undefined

      switch (this.mediaType) {
        case MEDIA_TYPES.cytoscape:
          return await this.compileCytoscape()
        case MEDIA_TYPES.echarts:
          return await this.compileECharts()
        case MEDIA_TYPES.leaflet:
          return await this.compileLeaflet()
        case MEDIA_TYPES.mermaid:
          return await this.compileMermaid()
        case MEDIA_TYPES.plotly:
          return await this.compilePlotly()
        case MEDIA_TYPES.vegaLite:
          return await this.compileVegaLite()
        default:
          return
      }
    }
  }

  private async compileCytoscape() {
    const container = this.shadowRoot?.getElementById('cytoscape-container')
    if (!container) return

    this.cytoscapeInstance = (await compileCytoscape(
      this.contentUrl!,
      container,
      true // isStaticView
    )) as ImageObjectStatic['cytoscapeInstance']
  }

  private async compileECharts() {
    const container = this.shadowRoot?.getElementById('echarts-container')
    if (!container) return

    this.echartsInstance = await compileECharts(
      this.contentUrl!,
      container,
      this.echartsInstance,
      true, // isStaticView
      (error) => {
        this.error = error.message ?? error.toString()
      }
    )
  }

  private async compileLeaflet() {
    await compileLeaflet(
      this.contentUrl!,
      (html) => {
        this.leaflet = html
        this.error = undefined
      },
      (error) => {
        this.error = error
      }
    )
  }

  private async compileMermaid() {
    await compileMermaid(
      this.contentUrl!,
      this,
      (svg) => {
        this.mermaid = svg
        this.error = undefined
      },
      (error) => {
        this.error = error.message ?? error.toString()
      }
    )
  }

  private async compilePlotly() {
    const container = this.shadowRoot?.getElementById('plotly-container')
    if (!container || !this.shadowRoot) return

    await compilePlotly(
      this.contentUrl!,
      container,
      this.shadowRoot,
      true, // isStaticView
      (error) => {
        this.error = error.message ?? error.toString()
      }
    )
  }

  private async compileVegaLite() {
    const container = this.shadowRoot?.getElementById('vegalite-container')
    if (!container) return

    this.vegaLiteInstance = await compileVegaLite(
      this.contentUrl!,
      container,
      true, // isStaticView
      (error) => {
        this.error = error.message ?? error.toString()
      }
    )
  }

  override render() {
    if (this.error) {
      return html`<div class="error">${this.error}</div>`
    }

    switch (this.mediaType) {
      case MEDIA_TYPES.mermaid:
        return html`<div class="mermaid-container">
          ${this.mermaid ? unsafeSVG(this.mermaid) : ''}
        </div>`

      case MEDIA_TYPES.plotly:
        return html`
          <style id="plotly-css"></style>
          <div class="plotly-padder">
            <div class="viz-container" id="plotly-container"></div>
          </div>
        `

      case MEDIA_TYPES.echarts:
        return html`<div class="viz-container" id="echarts-container"></div>`

      case MEDIA_TYPES.vegaLite:
        return html`<div class="viz-container" id="vegalite-container"></div>`

      case MEDIA_TYPES.cytoscape:
        return html`<div class="cytoscape-container" id="cytoscape-container"></div>`

      case MEDIA_TYPES.leaflet:
        if (this.leaflet) {
          const blob = new Blob([this.leaflet], { type: 'text/html' })
          const blobUrl = URL.createObjectURL(blob)
          return html`<iframe class="leaflet-iframe" src=${blobUrl}></iframe>`
        }
        return html``

      default:
        // Regular image
        return html`
          <div class="image-container">
            ${this.contentUrl
              ? html`<img src=${this.contentUrl} />`
              : html`<slot></slot>`}
            <slot name="caption"></slot>
          </div>
        `
    }
  }
}
