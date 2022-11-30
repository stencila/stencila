import { html } from 'lit'
import { property, state } from 'lit/decorators'
import { TW } from 'twind'

import { isCodeWriteable } from '../../mode'
import Entity from './entity'

import '../base/icon-button'
import '../editors/code-editor/code-language'

/**
 * A base component to represent the `CodeStatic` node type
 *
 * This component is independent of, but similar to `StencilaCodeExecutable`
 * component. It is currently simpler in that is does not allow for
 * guessing of language or changing code visibility across all instances.
 *
 * @slot code The `CodeStatic.code` property
 */
export default class StencilaCodeStatic extends Entity {
  /**
   * The `CodeStatic.programmingLanguage` property
   */
  @property({ attribute: 'programming-language', reflect: true })
  programmingLanguage = ''

  /**
   * The `CodeStatic.code` property
   */
  @property({ reflect: true })
  public code?: string

  /**
   * An observer to update `code` from the slot
   */
  private codeObserver: MutationObserver

  /**
   * Handle a change, including on initial load, of the `code` slot
   */
  protected onCodeSlotChange(event: Event) {
    const textElem = (event.target as HTMLSlotElement).assignedElements({
      flatten: true,
    })[0]

    if (textElem) {
      this.code = textElem.textContent ?? ''

      this.codeObserver = new MutationObserver(() => {
        this.code = textElem.textContent ?? ''
      })
      this.codeObserver.observe(textElem, {
        subtree: true,
        characterData: true,
      })
    }
  }

  /**
   * Render a language menu
   */
  protected renderLanguageMenu(tw: TW) {
    const readOnly = !isCodeWriteable()

    return html`<stencila-code-language
      class=${tw`ml-0.5 text(base blue-500)`}
      programming-language=${this.programmingLanguage}
      ?disabled=${readOnly}
    ></stencila-code-language>`
  }
}
