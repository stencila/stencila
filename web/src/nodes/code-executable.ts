import '@shoelace-style/shoelace/dist/components/icon/icon'
import { html } from 'lit'
import { property } from 'lit/decorators.js'

import { Executable } from './executable'

/**
 * Abstract base class for web components representing Stencila Schema `CodeExecutable` node types
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-executable.md
 */
export abstract class CodeExecutable extends Executable {
  @property()
  code: string = ''

  @property({ attribute: 'programming-language' })
  programmingLanguage?: string

  /**
   * In dynamic view, the executable code can be read and run, but not changed.
   * So display programming language read only and provide buttons for actions
   */
  protected renderExecutableButtons() {
    return html`<span>
      <sl-icon name="play"></sl-icon>
  </span>`
  }

  /**
   * In visual view, the executable code can be edited and run. So provide
   * a selector for programming language and buttons for actions.
   */
  protected renderVisualViewHeader() {
    return html`<div>
      <select>
        <option selected>${this.programmingLanguage}</option>
      </select>
    </div>`
  }

  /**
   * In source view, the same interactions are possible as in the
   * visual view.
   */
  protected renderSourceViewHeader() {
    return this.renderVisualViewHeader()
  }
}

