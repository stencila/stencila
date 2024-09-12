import { css } from '@twind/core'
import { html, PropertyValues } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'
import { unsafeSVG } from 'lit/directives/unsafe-svg'

import { withTwind } from '../twind'

import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/cards/inline-on-demand'
import '../ui/nodes/properties/authors'

import { Entity } from './entity'

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

    try {
      const id = 'stencila-' + Date.now()
      this.svg = (await mermaid.render(id, this.content, container)).svg
    } catch (error) {
      // TODO: if this.ancestors.endsWith('CodeChunk') then add a compilation
      // message to that code chunk. Otherwise, render a pre element
      // with error

      // TODO parse error messages to get line and colum e.g.
      /*
      Parse error on line 2:
      graph LR    A -> B
      --------------^
      Expecting 'SEMI', 'NEWLINE'
      */
      this.error = error.message ?? error.toString()
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
