import { consume } from '@lit/context'
import { apply } from '@twind/core'
import { html, LitElement, PropertyValues } from 'lit'
import { customElement, state } from 'lit/decorators'

import { withTwind } from '../../twind'

import { DocumentHeadingsContext, documentHeadingsContext } from './context'

@customElement('stencila-ui-article-headings')
@withTwind()
export class ArticleHeadings extends LitElement {
  /**
   * The context containing the list of visible headings
   */
  @consume({ context: documentHeadingsContext, subscribe: true })
  @state()
  context: DocumentHeadingsContext

  /**
   * The <nav> element inside the <slot> which contains the
   * nested list of links to headings.
   */
  navElem: Element | HTMLElement | null = null

  /**
   * Override to handle changes in which headings are visible, setting
   * class accordingly.
   *
   * This needs to be done here, rather than as part of the `render`
   * method, because this component does not control rendering of
   * elements within the `navElement`
   */
  protected override update(changedProperties: PropertyValues): void {
    super.update(changedProperties)

    if (changedProperties.has('context')) {
      if (this.navElem) {
        this.navElem.querySelectorAll('a').forEach((a) => {
          const hrefSplit = a.href.split('#')
          const target = hrefSplit[hrefSplit.length - 1]

          if (this.context.visibleHeadingIds.includes(target)) {
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
    const navElem = (e.target as HTMLSlotElement).assignedElements({
      flatten: true,
    })[0]

    if (navElem && navElem.tagName.toLowerCase() === 'nav') {
      this.navElem = navElem
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
