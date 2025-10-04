import { css } from '@twind/core'
import { html, PropertyValues } from 'lit'
import { customElement, state } from 'lit/decorators.js'

import { withTwind } from '../twind'

import { ExecutionMessage } from './execution-message'
import { MediaObject } from './media-object'

import {
  compileCytoscape,
  renderCytoscapeContainer,
} from './image-object-cytoscape'
import {
  compileECharts,
  renderEChartsContainer,
} from './image-object-echarts'
import {
  compileLeaflet,
  renderLeafletIframe,
} from './image-object-leaflet'
import {
  compileMermaid,
  renderMermaid,
} from './image-object-mermaid'
import {
  compilePlotly,
  renderPlotlyContainer,
} from './image-object-plotly'
import {
  compileVegaLite,
  renderVegaLiteContainer,
} from './image-object-vegalite'

/**
 * Web component representing a Stencila Schema `ImageObject` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/image-object.md
 */
@customElement('stencila-image-object')
@withTwind()
export class ImageObject extends MediaObject {
  /**
   * Image media types that are rendered in the browser
   */
  static MEDIA_TYPES = {
    cytoscape: 'application/vnd.cytoscape.v3+json',
    echarts: 'application/vnd.apache.echarts+json',
    leaflet: 'text/html',
    mermaid: 'text/vnd.mermaid',
    plotly: 'application/vnd.plotly.v1+json',
    vegaLite: 'application/vnd.vegalite.v5+json',
  } as const

  /**
   * The Cytoscape.js instance
   *
   * Rather than import cytoscape.Core just stub out what we need. This avoids
   * accidental bloat of the bundle if cytoscape is statically imported.
   */
  @state()
  private cytoscape?: { resize: () => void }

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


  override needsCompiling(): boolean {
    // @ts-expect-error re media type
    return Object.values(ImageObject.MEDIA_TYPES).includes(this.mediaType)
  }

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

  private onResize = () => {
    // If this component is resized then resize as needed
    this.cytoscape?.resize()
    this.echarts?.resize()
  }

  private onThemeChange = async () => {
    // Re-compile if necessary
    if (this.cytoscape && this.contentUrl) {
      await this.compileCytoscape()
    } else if (this.mermaid && this.contentUrl) {
      await this.compileMermaid()
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

    // Dispose of instances
    if (this.echarts) {
      this.echarts.dispose()
    }

    super.disconnectedCallback()
  }

  override async updated(properties: PropertyValues) {
    super.updated(properties)

    if (properties.has('contentUrl') || properties.has('mediaType')) {
      if (!this.contentUrl) {
        return
      }

      switch(this.mediaType) {
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

    const isStaticMode = window.STENCILA_STATIC_MODE === true

    this.cytoscape = await compileCytoscape(
      this.contentUrl,
      container,
      isStaticMode
    )
  }

  private async compileECharts() {
    const container = this.shadowRoot.getElementById(
      'stencila-echarts-container'
    )

    // Setup code chunk error handling if in a code chunk
    let codeChunk: HTMLElement | undefined
    if (this.parentNodeIs('CodeChunk')) {
      codeChunk = this.closestGlobally('stencila-code-chunk')
      this.clearCodeChunkMessages(codeChunk)
    }

    const isStaticMode = window.STENCILA_STATIC_MODE === true
    this.echarts = await compileECharts(
      this.contentUrl,
      container,
      this.echarts,
      isStaticMode,
      (error) => {
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
    )
  }

  private async compileLeaflet() {
    // Setup code chunk error handling if in a code chunk
    let codeChunk: HTMLElement | undefined
    if (this.parentNodeIs('CodeChunk')) {
      codeChunk = this.closestGlobally('stencila-code-chunk')
      this.clearCodeChunkMessages(codeChunk)
    }

    await compileLeaflet(
      this.contentUrl,
      (htmlContent) => {
        this.leaflet = htmlContent
        this.error = undefined
      },
      (error) => {
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

          this.addCodeChunkErrorMessage(message, error)
          messages.appendChild(message)
        } else {
          this.error = error
        }
      }
    )
  }

  private async compileMermaid() {
    // Setup code chunk error handling if in a code chunk
    let codeChunk: HTMLElement | undefined
    if (this.parentNodeIs('CodeChunk')) {
      codeChunk = this.closestGlobally('stencila-code-chunk')
      this.clearCodeChunkMessages(codeChunk)
    }

    // Compile with theme
    await compileMermaid(
      this.contentUrl,
      this,
      (svg) => {
        this.mermaid = svg
      },
      (error) => {
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
    )
  }

  private async compilePlotly() {
    const container = this.shadowRoot.getElementById(
      'stencila-plotly-container'
    )

    // Setup code chunk error handling if in a code chunk
    let codeChunk: HTMLElement | undefined
    if (this.parentNodeIs('CodeChunk')) {
      codeChunk = this.closestGlobally('stencila-code-chunk')
      this.clearCodeChunkMessages(codeChunk)
    }

    const isStaticMode = window.STENCILA_STATIC_MODE === true
    await compilePlotly(
      this.contentUrl,
      container,
      this.shadowRoot,
      isStaticMode,
      (error) => {
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
        }
      }
    )
  }

  private async compileVegaLite() {
    const container = this.shadowRoot.querySelector(
      'div#stencila-vega-container'
    ) as HTMLElement

    // Setup code chunk error handling if in a code chunk
    let codeChunk: HTMLElement | undefined
    if (this.parentNodeIs('CodeChunk')) {
      codeChunk = this.closestGlobally('stencila-code-chunk')
      this.clearCodeChunkMessages(codeChunk)
    }

    const isStaticMode = window.STENCILA_STATIC_MODE === true
    await compileVegaLite(this.contentUrl, container, isStaticMode, (error) => {
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
      return renderMermaid(this.mermaid)
    }

    if (this.mediaType === ImageObject.MEDIA_TYPES.cytoscape) {
      return renderCytoscapeContainer()
    }

    if (this.mediaType === ImageObject.MEDIA_TYPES.plotly) {
      return renderPlotlyContainer()
    }

    if (this.mediaType === ImageObject.MEDIA_TYPES.vegaLite) {
      return renderVegaLiteContainer()
    }

    if (this.mediaType === ImageObject.MEDIA_TYPES.echarts) {
      return renderEChartsContainer()
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
