import { sentenceCase } from 'change-case'
import { html } from 'lit'
import { property, state } from 'lit/decorators'

import Executable from './executable'

/**
 * A base component to represent the `CodeExecutable` node type
 */
export default class StencilaCodeExecutable extends Executable {
  @property({
    attribute: 'programming-language',
  })
  programmingLanguage: string

  @state()
  protected isCodeVisible: boolean

  private onCodeVisibilityChanged(event: CustomEvent) {
    this.isCodeVisible = event.detail.isVisible
  }

  protected onCodeVisibilityClicked(event: PointerEvent) {
    if (event.shiftKey) {
      this.emit('stencila-code-visibility-change', {
        isVisible: !this.isCodeVisible,
      })
    } else {
      this.isCodeVisible = !this.isCodeVisible
    }
  }

  connectedCallback() {
    super.connectedCallback()

    window.addEventListener(
      'stencila-code-visibility-change',
      this.onCodeVisibilityChanged.bind(this)
    )
  }

  disconnectedCallback() {
    super.disconnectedCallback()

    window.removeEventListener(
      'stencila-code-visibility-change',
      this.onCodeVisibilityChanged.bind(this)
    )
  }
}
