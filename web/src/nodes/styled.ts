import { property } from 'lit/decorators.js'

import { Entity } from './entity'

/**
 * Abstract base class for web components representing Stencila Schema `Styled` node types
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/styled/styled.md
 */
export abstract class Styled extends Entity {
  @property()
  code: string

  @property({ attribute: 'style-language' })
  styleLanguage?: string

  @property()
  css?: string

  // Alternative name to avoid name clash with read-only `Element.classList`
  @property({ attribute: 'class-list' })
  classes?: string

  /**
   * Derived classes should call this in the render function
   */
  protected adoptCss() {
    if (this.css) {
      const stylesheet = new CSSStyleSheet()
      // Wrap the CSS in a class so that the CSS applies to only the element
      // with class=styled. Note that this requires CSS nesting: https://caniuse.com/css-nesting
      // which is supported in major browsers.
      stylesheet.replaceSync(`.styled { ${this.css} }`)

      const base = this.shadowRoot.adoptedStyleSheets[0]
      this.shadowRoot.adoptedStyleSheets = base
        ? [base, stylesheet]
        : [stylesheet]
    }
  }
}
