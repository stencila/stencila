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
   * Scroll event handler for oversized contents block
   * it will scroll down with page instead of adding another scrollbar
   */
  protected handleScroll() {
    const sidebar = this.shadowRoot.querySelector('#sidebar') as HTMLElement
    const sidebarHeight = sidebar.scrollHeight
    const windowHeight = window.innerHeight
    const scrollTop = window.scrollY

    if (sidebarHeight > windowHeight) {
      const maxScroll = sidebarHeight - windowHeight
      const scrollAmount = Math.min(scrollTop, maxScroll)

      sidebar.style.transform = `translateY(-${scrollAmount}px)`
    }
  }

  /**
   * Resets elements default y position if screen resized
   */
  protected handleResize() {
    const sidebar = this.shadowRoot.querySelector('#sidebar') as HTMLElement
    sidebar.style.transform = `translateY(0px)`
  }

  override connectedCallback(): void {
    super.connectedCallback()
    window.addEventListener('scroll', this.handleScroll.bind(this))
    window.addEventListener('resize', this.handleResize.bind(this))
  }

  override disconnectedCallback(): void {
    super.disconnectedCallback()
    window.removeEventListener('scroll', this.handleResize.bind(this))
    window.addEventListener('resize', this.handleResize.bind(this))
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
      'w-[200px]',
    ])

    return html`
      <div id="sidebar" class=${containerClasses}>
        <div class="sticky top-0 border-l border-black/20">
          <slot @slotchange=${this.handleSlotChange}></slot>
        </div>
      </div>
    `
  }
}
