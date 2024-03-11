import '@shoelace-style/shoelace/dist/components/icon/icon'
import { html } from 'lit'
import { property } from 'lit/decorators.js'
import moment from 'moment'

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

  /**
   * Render the fields displaying when the
   * last run was, and the duration of the last run
   */
  protected renderTimeFields() {
    return html`
      <stencila-basic-node-field icon-name="clock" icon-library="default">
        <span slot="content">${moment(this.executionEnded).fromNow()}</span>
      </stencila-basic-node-field>
      <stencila-basic-node-field
        icon-name="clock-history"
        icon-library="default"
      >
        <span slot="content">${this.executionDuration}ms</span>
      </stencila-basic-node-field>
    `
  }
}
