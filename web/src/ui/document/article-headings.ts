import { apply } from '@twind/core'
import { html, LitElement } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../../twind'

@customElement('stencila-ui-article-headings')
@withTwind()
export class ArticleHeadings extends LitElement {
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
    window.removeEventListener('scroll', this.handleScroll.bind(this))
    window.removeEventListener('resize', this.handleResize.bind(this))
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
        <div class="sticky top-0">
          <slot></slot>
        </div>
      </div>
    `
  }
}
