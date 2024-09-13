import { css } from '@twind/core'
import { html, PropertyValues } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'
import { unsafeSVG } from 'lit/directives/unsafe-svg'

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

  override async update(properties: PropertyValues) {
    super.update(properties)

    if (properties.has('content') || properties.has('mediaType')) {
      if (this.content && this.mediaType == 'text/vnd.mermaid') {
        await this.compileMermaid()
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

      if (codeChunk) {
        // Clear any existing messages
        const messages = codeChunk.querySelector('div[slot=messages]')
        if (messages) {
          while (messages.firstChild) {
            messages.removeChild(messages.firstChild)
          }
        }
      }
    }

    try {
      const id = 'stencila-' + Date.now()
      this.svg = (await mermaid.render(id, this.content, container)).svg
    } catch (error) {
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
        message.setAttribute('level', 'Error')
        message.setAttribute('error-type', 'ParseError')

        const expected = error.hash?.expected
        let str: string
        if (expected) {
          str = `expected ${expected.join(', ')}`
        } else {
          str = error.message ?? error.toString()
          str = str.slice(str.lastIndexOf('-^\n')).trim()
        }
        message.setAttribute('message', str)

        const loc = error.hash?.loc
        const startLine = (loc.first_line ?? 1) - 1
        const startCol = (loc.first_column ?? 0) + 1
        const endLine = (loc.last_line ?? 1) - 1
        const endCol = (loc.last_column ?? 0) + 1
        if (loc) {
          message.setAttribute(
            'code-location',
            `[${startLine},${startCol},${endLine},${endCol}]`
          )
        }

        messages.appendChild(message)
      } else {
        // Otherwise, render a <pre> element with error
        this.error = error.message ?? error.toString()
      }
    }

    container.remove()
  }

  override render() {
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
    return this.error
      ? this.renderErrors()
      : this.svg
        ? this.renderSvg()
        : this.renderImg()
  }

  private renderErrors() {
    return html`<div slot="content">
      <pre class="whitespace-pre overflow-x-auto"><code>${this
        .error}</code></pre>
    </div>`
  }

  private renderSvg() {
    return html`<div slot="content">${unsafeSVG(this.svg)}</div>`
  }

  private renderImg() {
    const imgStyles = css`
      & img {
        width: 100%;
      }
    `
    return html`<div slot="content" class=${imgStyles}>
      <slot></slot>
    </div>`
  }
}
