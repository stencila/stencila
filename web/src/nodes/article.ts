import { provide } from '@lit/context'
import { LitElement, css, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'
import {
  DocumentHeadingsContext,
  documentHeadingsContext,
} from '../ui/document/context'

import '../ui/nodes/properties/authors-provenance'
import '../ui/document/article-headings'

import { HEADING_VISIBILITY_EVENT, HeadingVisibilityEvent } from './heading'

@customElement('stencila-article')
@withTwind()
export class StencilaArticle extends LitElement {
  /**
   * Indicates that this is the root node of the document
   */
  @property({ type: Boolean })
  root: boolean

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
    this.addEventListener(
      HEADING_VISIBILITY_EVENT,
      this.handleHeadingVisibility.bind(this)
    )
  }

  override disconnectedCallback(): void {
    super.disconnectedCallback()
    this.removeEventListener(
      HEADING_VISIBILITY_EVENT,
      this.handleHeadingVisibility.bind(this)
    )
  }

  /**
   * Provide some formatting rules for the authors and provenance elements
   */
  static override styles = css`
    [slot='authors'] > slot::slotted(*) {
      display: flex;
      flex-direction: column;
      gap: 0.75rem; // gap-3
    }

    [slot='provenance'] > slot::slotted(*) {
      display: flex;
      flex-direction: row;
      gap: 0.75rem; // gap-3
    }
  `

  override render() {
    return html`
      <aside class="min-w-80 max-w-prose mx-auto">
        <stencila-ui-authors-provenance>
          <div class="flex flex-col gap-y-4">
            <div slot="authors">
              <label class="block text-sm mb-4">Contributors</label>
              <slot name="authors"></slot>
            </div>
            <div slot="provenance">
              <label class="block text-sm mb-4">Provenance</label>
              <slot name="provenance"></slot>
            </div>
          </div>
        </stencila-ui-authors-provenance>
      </aside>

      <slot name="config"></slot>

      <stencila-ui-article-headings>
        <slot name="headings"></slot>
      </stencila-ui-article-headings>

      <slot name="content"></slot>
    `
  }
}
