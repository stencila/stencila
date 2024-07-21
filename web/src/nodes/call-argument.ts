import { html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../twind'
import '../ui/nodes/card'
import { nodeUi } from '../ui/nodes/icons-and-colours'

import { Parameter } from './parameter'

/**
 * Web component representing a Stencila Schema `CallArgument` node within a `CallBlock`
 *
 * Note that a `CallArgument` extends a `Parameter` but in addition has `code` and
 * `programmingLanguage` properties. As such the actual value of the arguments is
 * either:
 *
 * - a static `value` (inherited from `Parameter`), or
 *
 * - a dynamic `code` expression.
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/call-argument.md
 */
@customElement('stencila-call-argument')
@withTwind()
export class CallArgument extends Parameter {
  @property()
  code?: string

  @property({ attribute: 'programming-language' })
  programmingLanguage?: string

  override render() {
    const { borderColour } = nodeUi('CallBlock')

    const code =
      this.code?.length > 0 ? this.code : this.value?.toString() ?? ''
    const language = this.programmingLanguage ?? this.code ? 'json' : 'js'

    return html`
      <div class="flex flex-row items-center p-1 gap-2 bg-[${borderColour}]">
        <span>•</span>

        <input
          class="flex-grow rounded-sm px-2 py-1 font-mono text-sm h-[2em]"
          readonly
          value=${this.name}
        />

        <span>=</span>

        <stencila-ui-node-code
          type="CallBlock"
          code=${code}
          language=${language}
          read-only
          no-gutters
          containerClasses="inline-block w-full border border-[${borderColour}] rounded-sm overflow-hidden"
        >
          <slot name="execution-messages"></slot>
        </stencila-ui-node-code>
      </div>
    `
  }
}
