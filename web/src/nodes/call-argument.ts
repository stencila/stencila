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
    const { colour, borderColour } = nodeUi('CallBlock')

    const code =
      this.code?.length > 0 ? this.code : this.value?.toString() ?? ''

    const language = this.programmingLanguage ?? this.code ? 'json5' : 'js'

    return html`
      <div class="flex flex-row items-center gap-x-3 px-3 py-1 bg-[${colour}]">
        <input
          class="w-1/3 rounded-sm border border-[${borderColour}] px-2 py-1 font-mono h-[2em] text-ellipsis outline-black"
          readonly
          value=${this.name}
        />
        <span class="font-mono">:</span>

        <stencila-ui-node-code
          type="CallBlock"
          code=${code}
          language=${language}
          read-only
          no-gutters
          class="flex items-center flex-grow max-w-[50%]"
          container-classes="inline-block w-full rounded-sm border border-[${borderColour}] overflow-hidden text-ellipsis"
        >
          <slot name="execution-messages"></slot>
        </stencila-ui-node-code>

        ${language
          ? html`<stencila-ui-node-programming-language
              programming-language=${language}
            ></stencila-ui-node-programming-language>`
          : ''}
      </div>
    `
  }
}
