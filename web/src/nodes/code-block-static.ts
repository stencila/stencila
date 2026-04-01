import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import '../ui/nodes/properties/code/code-static'
import { booleanConverter } from '../utilities/booleanConverter'

/**
 * Static version of the CodeBlock web component
 *
 * This is a lightweight alternative for static views that does not render node
 * cards and uses Prism.js instead of CodeMirror for syntax highlighting.
 *
 * Uses Light DOM to allow text selection for site review functionality.
 */
@customElement('stencila-code-block')
export class CodeBlockStatic extends LitElement {
  @property()
  code: string

  @property({ attribute: 'programming-language' })
  programmingLanguage?: string

  @property({
    attribute: 'is-demo',
    converter: booleanConverter,
  })
  isDemo?: boolean

  /**
   * Demo content element detached from DOM before Lit renders,
   * to be re-appended after the code block in firstUpdated.
   */
  private demoContent?: Element

  /**
   * Use Light DOM so that Prism styles can be applied and
   * text selection works for site review
   */
  protected override createRenderRoot() {
    return this
  }

  override connectedCallback() {
    // Remove server-rendered <pre> content before Lit renders
    // (server includes <pre><code> for no-JS fallback)
    const existingPre = this.querySelector(':scope > pre')
    if (existingPre) {
      existingPre.remove()
    }

    // Detach demo content so Lit's template renders first (the code block),
    // then we re-append content after it in firstUpdated
    const contentSlot = this.querySelector(':scope > [slot="content"]')
    if (contentSlot) {
      this.demoContent = contentSlot
      contentSlot.remove()
    }

    super.connectedCallback()
  }

  protected override firstUpdated() {
    if (this.demoContent) {
      this.appendChild(this.demoContent)
      this.demoContent = undefined
    }
  }

  override render() {
    return html`
      <stencila-ui-node-code-static
        code=${this.code}
        language=${this.programmingLanguage}
      >
      </stencila-ui-node-code-static>
    `
  }
}
