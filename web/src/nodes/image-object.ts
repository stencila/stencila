import { css } from '@twind/core'
import { html, PropertyValues } from 'lit'
import { customElement, state } from 'lit/decorators.js'
import { unsafeSVG } from 'lit/directives/unsafe-svg'

import { withTwind } from '../twind'
import { colorToHex } from '../utilities/colorUtils'
import { getCSSVariable, getCSSVariables } from '../utilities/cssVariables'

import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/cards/inline-on-demand'

import { ExecutionMessage } from './execution-message'
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
   * Image media types that are rendered in the browser
   */
  static MEDIA_TYPES = {
    cytoscape: 'application/vnd.cytoscape.v3+json',
    mermaid: 'text/vnd.mermaid',
    plotly: 'application/vnd.plotly.v1+json',
    vegaLite: 'application/vnd.vegalite.v5+json',
    htmlMap: 'text/html',
  } as const

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
   *
   * Rather than import cytoscape.Core just stub out what we need. This avoids
   * accidental bloat of the bundle if cytoscape is statically imported.
   */
  private cytoscape?: { resize: () => void }

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

  private onResize() {
    // If this component is resized then resize the Cytoscape instance, if any
    this.cytoscape?.resize()
  }

  /**
   * Extract diagram theme variables from CSS custom properties
   * @returns Object containing theme configuration for diagrams
   */
  private computeDiagramTheme() {
    const cssVars = getCSSVariables(document.documentElement, {
      background: '--diagram-background',
      fontFamily: '--diagram-font-family',
      fontSize: '--diagram-font-size',
      nodeBackground: '--diagram-node-background',
      nodeText: '--diagram-node-text-color',
      nodeBackgroundSecondary: '--diagram-node-background-secondary',
      nodeBackgroundTertiary: '--diagram-node-background-tertiary',
      edgeColor: '--diagram-edge-color',
      textColor: '--diagram-text-color',
      nodeBorderColor: '--diagram-node-border-color',
      nodeBorderWidth: '--diagram-node-border-width',
      edgeWidth: '--diagram-edge-width',
      actorBackground: '--diagram-actor-background',
      actorBorder: '--diagram-actor-border',
      sequenceSignal: '--diagram-sequence-signal',
      clusterBackground: '--diagram-cluster-background',
      edgeLabelColor: '--diagram-edge-text-color'
    })

    return {
      background: colorToHex(cssVars.background),
      fontFamily: cssVars.fontFamily.replace(/['"]/g, ''),
      fontSize: cssVars.fontSize,
      primaryColor: colorToHex(cssVars.nodeBackground),
      primaryTextColor: colorToHex(cssVars.nodeText),
      secondaryColor: colorToHex(cssVars.nodeBackgroundSecondary),
      tertiaryColor: colorToHex(cssVars.nodeBackgroundTertiary),
      lineColor: colorToHex(cssVars.edgeColor),
      textColor: colorToHex(cssVars.textColor),
      nodeBorder: colorToHex(cssVars.nodeBorderColor),
      nodeBorderWidth: cssVars.nodeBorderWidth,
      edgeWidth: cssVars.edgeWidth,

      // Sequence diagram variables
      actorBkg: colorToHex(cssVars.nodeBackground), // Use consistent node background
      actorBorder: colorToHex(cssVars.nodeBorderColor), // Use consistent border
      actorTextColor: colorToHex(cssVars.nodeText), // Use consistent text color
      signalColor: colorToHex(cssVars.sequenceSignal),
      signalTextColor: colorToHex(cssVars.textColor),
      activationBkgColor: colorToHex(cssVars.nodeBackgroundSecondary),
      activationBorderColor: colorToHex(cssVars.nodeBorderColor),
      labelBoxBkgColor: 'transparent',
      labelTextColor: colorToHex(cssVars.edgeLabelColor),

      // State diagram variables
      stateBkg: colorToHex(cssVars.nodeBackground), // Use consistent node background
      stateLabelColor: colorToHex(cssVars.nodeText), // Use consistent text color
      transitionColor: colorToHex(cssVars.edgeColor),
      transitionLabelColor: colorToHex(cssVars.edgeLabelColor),
      specialStateColor: colorToHex(cssVars.nodeBorderColor),
      arrowheadColor: colorToHex(cssVars.edgeColor), // Ensure arrowheads match edges
      fillType0: colorToHex(cssVars.nodeBackground), // Additional state fills
      fillType1: colorToHex(cssVars.nodeBackgroundSecondary),
      fillType2: colorToHex(cssVars.nodeBackgroundTertiary),

      // Existing variables
      clusterBkg: colorToHex(cssVars.clusterBackground),
      edgeLabelBackground: 'transparent',
      primaryBorderColor: colorToHex(cssVars.nodeBorderColor)
    }
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
      }
      // URL resolution is handled by parent MediaObject class
    }
  }

  private async compileCytoscape() {
    const { default: cytoscape } = await import('cytoscape')

    const graph = JSON.parse(this.contentUrl)
    graph.container = this.shadowRoot.querySelector(
      'div#stencila-cytoscape-container'
    ) as HTMLElement

    // Configure for static mode if enabled
    const isStaticMode = window.STENCILA_STATIC_MODE === true
    if (isStaticMode) {
      graph.userZoomingEnabled = false
      graph.userPanningEnabled = false
      graph.boxSelectionEnabled = false
      graph.autoungrabify = true
    }

    // Apply theme styles from CSS custom properties
    const theme = this.computeDiagramTheme()
    const themeStyle = [
      {
        selector: 'node',
        style: {
          'background-color': theme.primaryColor,
          'border-color': theme.nodeBorder,
          'border-width': theme.nodeBorderWidth,
          'color': theme.primaryTextColor,
          'font-family': theme.fontFamily,
          'font-size': theme.fontSize
        }
      },
      {
        selector: 'edge',
        style: {
          'line-color': theme.lineColor,
          'width': theme.edgeWidth,
          'target-arrow-color': theme.lineColor,
          'color': theme.textColor,
          'font-family': theme.fontFamily,
          'font-size': theme.fontSize
        }
      }
    ]

    // Merge theme styles with existing styles if any
    if (graph.style) {
      if (Array.isArray(graph.style)) {
        graph.style = [...themeStyle, ...graph.style]
      } else {
        graph.style = [...themeStyle, graph.style]
      }
    } else {
      graph.style = themeStyle
    }

    this.cytoscape = cytoscape(graph)
  }

  private async compileMermaid() {
    // Import Mermaid dynamically, when it is required, rather than have
    // it bundled into the main JS file for the view
    // @ts-expect-error - Using ESM min build to avoid parser dependency issues
    const { default: mermaid } = await import('mermaid/dist/mermaid.esm.min.mjs')

    // Initialize Mermaid with theme from CSS custom properties
    const theme = this.computeDiagramTheme()
    mermaid.initialize({
      theme: 'base',
      themeVariables: theme
    })

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
      // Configure for static mode if enabled
      const isStaticMode = window.STENCILA_STATIC_MODE === true
      const config = isStaticMode ? {
        ...spec.config,
        staticPlot: true,
        displayModeBar: false,
        scrollZoom: false,
        doubleClick: false,
        showTips: false,
        dragMode: false
      } : spec.config

      await Plotly.react(container, spec.data, spec.layout, config)

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
    const isStaticMode = window.STENCILA_STATIC_MODE === true
    const embedOptions = {
      renderer: 'svg' as const,
      actions: false,
      mode: 'vega-lite' as const,
      ...(isStaticMode && {
        config: {
          view: { continuousWidth: 400, continuousHeight: 300 },
          axis: { domain: false, ticks: false },
          legend: { disable: true }
        }
      })
    }

    vegaEmbed(container, spec, embedOptions).catch((error) => {
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

        /* Fix Mermaid edge label backgrounds */
        /* Based on solution from: https://stephenkernan.com/blog/how-to-style-mermaid-edge-labels */
        /* and related issue: https://github.com/mermaid-js/mermaid/issues/3021 */
        foreignObject {
          display: flex;
          align-items: center;
          justify-content: center;
          &:has(.edgeLabel) {
            background-color: transparent;
            .edgeLabel,
            .labelBkg {
              background-color: transparent !important;
            }
            p {
              margin-inline: auto !important;
              max-width: max-content;
              color: ${getCSSVariable(document.documentElement, '--diagram-edge-text-color')};
            }
          }
        }
      }
    `

    return html`
      <div slot="content" class=${svgStyles}>${unsafeSVG(this.svg)}</div>
    `
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
