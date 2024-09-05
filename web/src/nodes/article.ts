import { provide } from '@lit/context'
import { LitElement, html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'
import { TOCContext, tocContext } from '../ui/document/context'
import { eventThrottle } from '../utilities/throttle'

import { Heading } from './heading'

import '../ui/nodes/properties/authors-provenance'
import '../ui/document/article-headings'

@customElement('stencila-article')
@withTwind()
export class StencilaArticle extends LitElement {
  /**
   * context provider for the TOC
   */
  @provide({ context: tocContext })
  tocContext: TOCContext = { scrolledHeadingIds: [] }

  /**
   * Array of all the `stencila-headings` in the document
   */
  headings: Heading[] | null

  /**
   * Set/reset the headings property if content slot changes
   */
  protected handleContentSlotChange() {
    const headers = this.querySelectorAll(
      'stencila-heading'
    ) as NodeListOf<Heading>

    if (headers.length > 0) {
      this.headings = Array.from(headers)
    }
  }

  /**
   * scroll event to track which headings have been scolled past
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
        scrolledHeadingIds.length !== this.tocContext.scrolledHeadingIds.length
      ) {
        this.tocContext = {
          scrolledHeadingIds,
        }
      }
    }
  }

  override connectedCallback() {
    super.connectedCallback()
    window.addEventListener(
      'scroll',
      eventThrottle(this.handleScroll.bind(this), 200)
    )
  }

  override disconnectedCallback() {
    super.disconnectedCallback()
    window.removeEventListener(
      'scroll',
      eventThrottle(this.handleScroll.bind(this), 200)
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

      <!-- TODO: <stencila-ui-article-headings> component -->
      <stencila-ui-article-headings>
        <slot name="headings"></slot>
      </stencila-ui-article-headings>

      <slot name="content" @slotchange=${this.handleContentSlotChange}></slot>
    `
  }
}
