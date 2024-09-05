import { provide } from '@lit/context'
import { LitElement, html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'
import {
  DocumentHeadingsContext,
  documentHeadingsContext,
} from '../ui/document/context'
import { eventThrottle } from '../utilities/throttle'

import { Heading } from './heading'

import '../ui/nodes/properties/authors-provenance'
import '../ui/document/article-headings'

@customElement('stencila-article')
@withTwind()
export class StencilaArticle extends LitElement {
  /**
   * Context provider for the article's `headings` navigation (TOC)
   *
   * Maintains a list of ids of the headings that are currently
   * in the viewport to enable highlighting of those headings
   * in the `<stencila-ui-article-headings>` component.
   *
   * See the `handleScroll` method for where this list is updated.
   */
  @provide({ context: documentHeadingsContext })
  headingsContext: DocumentHeadingsContext = { visibleHeadingIds: [] }

  /**
   * Array of all the `<stencila-heading>` elements in the article
   *
   * Used to update the `visibleHeadingIds` in the `headingsContext`.
   */
  headings: Heading[] | null

  /**
   * Update the `headings` property on content slot changes
   */
  protected handleContentSlotChange() {
    const headings = this.querySelectorAll(
      'stencila-heading'
    ) as NodeListOf<Heading>

    if (headings.length > 0) {
      this.headings = Array.from(headings)
    }
  }

  /**
   * Handle scroll events to update which headings are visible
   */
  private handleScroll() {
    if (this.headings) {
      const scrolledHeadingIds: string[] = []

      this.headings.forEach((h) => {
        if (h.getRectTop() < 0) {
          scrolledHeadingIds.push(h.id)
        }
      })
      if (
        scrolledHeadingIds.length !==
        this.headingsContext.visibleHeadingIds.length
      ) {
        this.headingsContext = {
          visibleHeadingIds: scrolledHeadingIds,
        }
      }
    }
  }

  override connectedCallback() {
    super.connectedCallback()
    window.addEventListener(
      'scroll',
      eventThrottle(this.handleScroll.bind(this), 100)
    )
  }

  override disconnectedCallback() {
    super.disconnectedCallback()
    window.removeEventListener(
      'scroll',
      eventThrottle(this.handleScroll.bind(this), 100)
    )
  }

  override render() {
    return html`
      <aside class="min-w-80 max-w-prose mx-auto">
        <stencila-ui-authors-provenance>
          <div class="flex flex-col gap-y-4">
            <div slot="authors">
              <slot name="authors"></slot>
            </div>
            <div slot="provenance">
              <slot name="provenance"></slot>
            </div>
          </div>
        </stencila-ui-authors-provenance>
      </aside>

      <stencila-ui-article-headings>
        <slot name="headings"></slot>
      </stencila-ui-article-headings>

      <slot name="content" @slotchange=${this.handleContentSlotChange}></slot>
    `
  }
}
