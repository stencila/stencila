import { consume } from '@lit/context'
import { apply } from '@twind/core'
import { html, LitElement, PropertyValues } from 'lit'
import { customElement, state } from 'lit/decorators'

import { withTwind } from '../../twind'

import { TOCContext, tocContext } from './context'

@customElement('stencila-ui-article-headings')
@withTwind()
export class ArticleHeadings extends LitElement {
  @consume({ context: tocContext, subscribe: true })
  @state()
  context: TOCContext

  headerNav: Element | HTMLElement | null = null

  protected override update(changedProperties: PropertyValues): void {
    super.update(changedProperties)

    if (changedProperties.has('context')) {
      if (this.headerNav) {
        this.headerNav.querySelectorAll('a').forEach((a) => {
          const target = a.href.split('#')[1]
          if (this.context.scrolledHeadingIds.includes(target)) {
            a.classList.add('active')
          } else {
            a.classList.remove('active')
          }
        })
      }
    }
  }

  /**
   * Assigns the `headerNav` property when the nav element on slot change
   */
  handleSlotChange(e: Event) {
    const navEl = (e.target as HTMLSlotElement).assignedElements({
      flatten: true,
    })[0]

    if (navEl && navEl.tagName.toLowerCase() === 'nav') {
      this.headerNav = navEl
    }
  }

  protected override render() {
    const containerClasses = apply([
      'hidden lg:block',
      'fixed top-0 right-0',
      'pt-10',
      'h-screen w-[200px]',
      'overflow-y-auto',
      'border-l border-black/20',
    ])

    return html`
      <div class=${containerClasses}>
        <slot @slotchange=${this.handleSlotChange}></slot>
      </div>
    `
  }
}
