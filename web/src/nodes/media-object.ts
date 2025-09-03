import { apply } from '@twind/core'
import { html, PropertyValues, TemplateResult } from 'lit'
import { property, state } from 'lit/decorators.js'

import { Entity } from './entity'

/**
 * Base class for media objects (image, video, audio) with common properties and URL resolution
 */
export abstract class MediaObject extends Entity {
  /**
   * The media (MIME) type of the media object
   */
  @property({ attribute: 'media-type' })
  mediaType?: string

  /**
   * The URL of the source of the media object
   *
   * For HTTP and file URLs, equivalent to the `src` attribute in HTML. 
   * This will be rewritten if necessary, depending upon the context (e.g. VSCode).
   */
  @property({ attribute: 'content-url' })
  contentUrl?: string

  /**
   * The resolved URL of the media `src` attribute
   */
  @state()
  mediaSrc?: string

  /**
   * Any error message generated while attempting to load the media
   */
  @state()
  protected error?: string

  override async updated(properties: PropertyValues) {
    super.updated(properties)

    if (properties.has('contentUrl')) {
      if (!this.contentUrl) {
        return
      }

      await this.resolveMediaUrl()
    }
  }

  private async resolveMediaUrl() {
    if (this.contentUrl.startsWith('data:')) {
      // Data URLs can be used directly
      this.mediaSrc = this.contentUrl
    } else if (
      this.contentUrl.startsWith('https://') ||
      this.contentUrl.startsWith('http://')
    ) {
      // Use HTTP URLs directly
      this.mediaSrc = this.contentUrl
    } else {
      // If file path, and in VSCode webview, then prefix with workspace URI
      const workspace = this.closestGlobally(
        'stencila-vscode-view'
      )?.getAttribute('workspace')

      this.mediaSrc = workspace
        ? `${workspace}/${this.contentUrl}`
        : this.contentUrl
    }

    // Prefetch to check that URL is valid
    try {
      const response = await fetch(this.mediaSrc, { method: 'HEAD' })
      if (response.ok) {
        this.error = undefined
      } else {
        let src = this.contentUrl
        if (src.length > 40) {
          src = src.slice(0, 40) + '\u2026'
        }
        const message = await response.text()
        this.error = `Error fetching media '${src}': ${message}`
      }
    } catch (fetchError) {
      let src = this.contentUrl
      if (src.length > 40) {
        src = src.slice(0, 40) + '\u2026'
      }
      this.error = `Error fetching media '${src}': ${fetchError.message || fetchError}`
    }
  }

  // @ts-expect-error return type
  override render() {
    if (this.isWithin('StyledBlock') || this.isWithinUserChatMessage()) {
      return this.renderErrorOrContent()
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

  protected renderBlockOnDemand() {
    return html`
      <stencila-ui-block-on-demand
        type=${this.type()}
        node-id=${this.id}
        depth=${this.depth}
        ?has-root=${this.hasRoot()}
      >
        ${this.renderErrorOrContent()}
      </stencila-ui-block-on-demand>
    `
  }

  protected renderInlineOnDemand() {
    return html`
      <stencila-ui-inline-on-demand type=${this.type()}>
        <div slot="body">
          <stencila-ui-node-authors type=${this.type()}>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
        </div>
        ${this.renderErrorOrContent()}
      </stencila-ui-inline-on-demand>
    `
  }

  protected renderErrorOrContent() {
    return this.error ? this.renderError() : this.renderContent()
  }

  protected renderError() {
    const classes = apply(
      'overflow-x-auto px-2 py-1',
      'rounded border border-red-200 bg-red-100',
      'text-red-900 text-sm whitespace-pre'
    )

    return html`<div slot="content">
      <pre class=${classes}><code>${this.error}</code></pre>
    </div>`
  }

  protected abstract renderContent(): TemplateResult
}