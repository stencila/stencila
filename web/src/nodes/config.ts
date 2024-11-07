import { LitElement, PropertyValues } from 'lit'
import { customElement, property } from 'lit/decorators.js'

/**
 * Web component representing a Stencila Schema `Config` node
 *
 * Currently simply a placeholder with the view to allowing document level
 * configuration options to be set via this component.
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/config.md
 */
@customElement('stencila-config')
export class Config extends LitElement {
  @property()
  theme?: string

  override updated(changedProperties: PropertyValues) {
    if (changedProperties.has('theme')) {
      this.updateThemeLink()
    }
  }

  /**
   * Update the theme link in the `<head>` of the page when the theme changes
   */
  updateThemeLink() {
    // TODO Currently not used pending finding a way to be able to resolve the
    // full stylesheet URL properly.
    return

    const head = document.head
    let link = head.querySelector('link[title="theme"][rel="stylesheet"]')

    if (!link) {
      link = document.createElement('link')
      link.setAttribute('title', 'theme')
      link.setAttribute('rel', 'stylesheet')
      link.setAttribute('type', 'text/css')
      head.appendChild(link)
    }

    link.setAttribute('href', `${this.theme}.css`)
  }
}
