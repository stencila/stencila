import { provide } from '@lit/context'
import { css, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'
import {
  DocumentHeadingsContext,
  documentHeadingsContext,
} from '../ui/document/context'

import '../ui/nodes/properties/authors-provenance'
import '../ui/nodes/properties/reference'
import '../ui/document/article-headings'

import { Entity } from './entity'
import { HEADING_VISIBILITY_EVENT, HeadingVisibilityEvent } from './heading'

@customElement('stencila-article')
@withTwind()
export class Article extends Entity {
  @property({ attribute: 'date-published' })
  datePublished?: string

  @property({ type: Array })
  identifiers?: unknown[]

  /**
   * Context provider for the visibility of the article's headings
   *
   * Maintains the ids of the headings that are currently
   * visible (in the viewport) to enable highlighting of links
   * to those headings in the `<stencila-ui-article-headings>` component.
   */
  @provide({ context: documentHeadingsContext })
  headingsContext: DocumentHeadingsContext = {}

  /**
   * Handle a change in the visibility of a heading by
   * updating the `headingsContext`
   *
   * Warning: it is necessary to replace the context for changes
   * to reactively propagate to consumers of the context
   */
  handleHeadingVisibility({
    detail: { id, position, isEnd },
  }: CustomEvent<HeadingVisibilityEvent>) {
    const visibility = this.headingsContext[id] ?? [-1, -1]
    visibility[isEnd ? 1 : 0] = position
    this.headingsContext = { ...this.headingsContext, [id]: visibility }
  }

  override connectedCallback(): void {
    super.connectedCallback()
    if (this.isRoot()) {
      this.addEventListener(
        HEADING_VISIBILITY_EVENT,
        this.handleHeadingVisibility.bind(this)
      )
    }
  }

  override disconnectedCallback(): void {
    super.disconnectedCallback()
    if (this.isRoot()) {
      this.removeEventListener(
        HEADING_VISIBILITY_EVENT,
        this.handleHeadingVisibility.bind(this)
      )
    }
  }

  override render() {
    if (this.isRoot()) {
      return this.renderAsRoot()
    } else {
      return this.renderAsReference()
    }
  }

  /**
   * Render the `Article` as the root node
   */
  private renderAsRoot() {
    return html`
      <stencila-ui-authors-provenance>
        <slot name="authors" slot="author"></slot>
        <slot name="provenance" slot="provenance"></slot>
      </stencila-ui-authors-provenance>

      <slot name="config"></slot>

      <stencila-ui-article-headings>
        <slot name="headings"></slot>
      </stencila-ui-article-headings>

      <slot name="content"></slot>
    `
  }

  /**
   * Render the `Article` as a reference
   */
  private renderAsReference() {
    return html`<stencila-ui-node-reference
      date=${this.datePublished}
      .identifiers=${this.identifiers}
      ><span slot="authors"><slot name="authors"></slot></span
      ><slot name="title" slot="title"></slot
    ></stencila-ui-node-reference>`
  }
}
