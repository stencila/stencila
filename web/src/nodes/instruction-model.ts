import { apply } from '@twind/core'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../twind'
import { nodeUi } from '../ui/nodes/icons-and-colours'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `InstructionModel` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/edit/instruction-model.md
 */
@customElement('stencila-instruction-model')
@withTwind()
export class InstructionModel extends Entity {
  @property()
  namePattern?: string

  @property({ type: Number })
  qualityWeight?: number

  @property({ type: Number })
  speedWeight?: number

  @property({ type: Number })
  costWeight?: number

  @property({ type: Number })
  minimumScore?: number

  @property({ type: Number })
  temperature?: number

  override render() {
    const { borderColour, colour } = nodeUi('InstructionBlock')

    const styles = apply(
      'flex flex-row items-center gap-3',
      'px-3 py-1.5',
      `bg-[${colour}]`,
      'text-xs leading-tight font-sans',
      `border-t border-[${borderColour}]`
    )

    const inputStyles = apply([
      `border border-[${borderColour}] rounded-sm`,
      `outline-[${borderColour}]/50`,
      'text-sm text-gray-600',
      'ml-2 p-0.5',
    ])

    return html`
      <div class=${styles}>
        <span>Model: </span>

        <span class="flex flex-row items-center">
          <sl-tooltip content="Model selection quality weighting">
            <stencila-ui-icon
              class="text-base"
              name="starFill"
            ></stencila-ui-icon>
            <input
              class="${inputStyles}"
              type="number"
              min="0"
              max="100"
              value=${this.qualityWeight ?? 1}
              readonly
              disabled
            />
          </sl-tooltip>
        </span>

        <span class="flex flex-row items-center">
          <sl-tooltip content="Model selection speed weighting">
            <stencila-ui-icon
              class="text-base"
              name="speedometer"
            ></stencila-ui-icon>
            <input
              class="${inputStyles}"
              type="number"
              min="0"
              max="100"
              value=${this.speedWeight ?? 1}
              readonly
              disabled
            />
          </sl-tooltip>
        </span>

        <span class="flex flex-row items-center">
          <sl-tooltip content="Model selection cost weighting">
            <stencila-ui-icon
              class="text-base"
              name="currencyDollar"
            ></stencila-ui-icon>
            <input
              class="${inputStyles}"
              type="number"
              min="0"
              max="100"
              value=${this.costWeight ?? 1}
              readonly
              disabled
            />
          </sl-tooltip>
        </span>

        <span class="flex flex-row items-center">
          <sl-tooltip content="Model inference temperature">
            <stencila-ui-icon
              class="text-base"
              name="thermometer"
            ></stencila-ui-icon>
            <input
              class="${inputStyles}"
              type="number"
              min="0"
              max="100"
              value=${this.temperature ?? 1}
              readonly
              disabled
            />
          </sl-tooltip>
        </span>
      </div>
    `
  }
}
