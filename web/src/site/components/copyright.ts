import { LitElement } from 'lit'
import { customElement } from 'lit/decorators.js'

/**
 * Copyright component with auto-updating year
 *
 * This component updates the copyright year on the client-side if the
 * server-rendered year is stale. It looks for a `.year` element with
 * `data-end` attribute containing the year at render time.
 *
 * If the current year is greater than `data-end`, the year text is updated:
 * - If `data-start` exists: shows "{start}-{currentYear}"
 * - Otherwise: shows "{currentYear}"
 */
@customElement('stencila-copyright')
export class StencilaCopyright extends LitElement {
  /**
   * Use Light DOM - the server renders the content, we just update it
   */
  protected override createRenderRoot() {
    return this
  }

  override connectedCallback() {
    super.connectedCallback()
    this.updateYear()
  }

  /**
   * Update the year if it's stale
   */
  private updateYear() {
    const yearElement = this.querySelector('.year')
    if (!yearElement) {
      return
    }

    const dataEnd = yearElement.getAttribute('data-end')
    if (!dataEnd) {
      return
    }

    const endYear = parseInt(dataEnd, 10)
    const currentYear = new Date().getFullYear()

    // Only update if the year has changed
    if (currentYear > endYear) {
      const dataStart = yearElement.getAttribute('data-start')

      if (dataStart) {
        yearElement.textContent = `${dataStart}-${currentYear}`
      } else {
        yearElement.textContent = `${currentYear}`
      }

      // Update the data-end attribute
      yearElement.setAttribute('data-end', `${currentYear}`)
    }
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'stencila-copyright': StencilaCopyright
  }
}
