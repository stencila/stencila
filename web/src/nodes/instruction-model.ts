import { apply } from '@twind/core'
import { css, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../twind'
import { nodeUi } from '../ui/nodes/icons-and-colours'

import { Entity } from './entity'

import '@shoelace-style/shoelace/dist/components/dropdown/dropdown.js'
import '@shoelace-style/shoelace/dist/components/range/range.js'
import '@shoelace-style/shoelace/dist/components/divider/divider.js'

import '../ui/inputs/select'

// const models = [
//   'stencila/router',
//   'anthropic/claude-3-5-haiku-20241022',
//   'anthropic/claude-3-5-sonnet-20240620',
//   'anthropic/claude-3-5-sonnet-20241022',
//   'anthropic/claude-3-haiku-20240307',
//   'anthropic/claude-3-opus-20240229',
//   'anthropic/claude-3-sonnet-20240229',
//   'cloudflare/deepseek-coder-6.7b-instruct-awq',
//   'cloudflare/llama-3.1-70b-instruct',
//   'cloudflare/llama-3.1-8b-instruct',
//   'cloudflare/llama-3.2-3b-instruct ',
//   'google/gemini-1.0-pro-001',
//   'google/gemini-1.5-flash-001',
//   'google/gemini-1.5-pro-001',
//   'mistral/codestral-2405',
//   'mistral/codestral-mamba-2407',
//   'mistral/mistral-large-2407',
//   'mistral/mistral-medium-2312',
//   'mistral/mistral-small-2402',
//   'mistral/mistral-tiny-2312',
//   'mistral/mistral-tiny-2407',
//   'mistral/open-mistral-nemo-2407',
//   'mistral/open-mixtral-8x22b-2404',
//   'openai/dall-e-2',
//   'openai/dall-e-3',
//   'openai/gpt-3.5-turbo-1106',
//   'openai/gpt-3.5-turbo-instruct-0914',
//   'openai/gpt-4-0613',
//   'openai/gpt-4-turbo-2024-04-09',
//   'openai/gpt-4o-2024-05-13',
//   'openai/gpt-4o-2024-08-06',
//   'openai/gpt-4o-mini-2024-07-18',
//   'openai/o1-mini-2024-09-12',
//   'openai/o1-preview-2024-09-12',
// ]

const TOTAL_COLLECTIVE_WEIGHT_SUM = 100

type ModelWeightFields = 'speedWeight' | 'costWeight' | 'qualityWeight'

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
  qualityWeight?: number = 70

  @property({ type: Number })
  speedWeight?: number = 15

  @property({ type: Number })
  costWeight?: number = 15

  @property({ type: Number })
  minimumScore?: number

  @property({ type: Number })
  temperature?: number

  static override styles = css`
    sl-divider {
      border-top: solid var(--width) var(--color);
      margin: 0.5rem 0;
    }
    sl-menu-label::part(base) {
      padding: 0 var(--sl-spacing-2x-small);
    }
    sl-range::part(form-control-label) {
      font-size: 0.875rem;
    }
  `

  private readonly modelWeightFields: ModelWeightFields[] = [
    'costWeight',
    'qualityWeight',
    'speedWeight',
  ]

  private onSelectionWeightChange(
    event: InputEvent,
    changedField: ModelWeightFields
  ) {
    const newValue = (event.target as HTMLInputElement).value

    const diff = TOTAL_COLLECTIVE_WEIGHT_SUM - +newValue

    const otherFields = this.modelWeightFields.filter((f) => f !== changedField)
    const otherValues = otherFields.map((f) => this[f])

    const sumOtherValues = otherFields.reduce(
      (acc, field) => (acc += this[field]),
      0
    )
    let total = +newValue
    otherFields.forEach((field, i) => {
      let val = Math.round(
        Math.max(0, Math.min(100, diff * (otherValues[i] / sumOtherValues)))
      )
      total += val

      if (
        i === otherFields.length - 1 &&
        total !== TOTAL_COLLECTIVE_WEIGHT_SUM
      ) {
        val += TOTAL_COLLECTIVE_WEIGHT_SUM - total
      }
      this[field] = val
    })

    this[changedField] = +newValue
  }

  override render() {
    const { borderColour, colour } = nodeUi('InstructionBlock')
    const styles = apply(
      'flex flex-row items-center',
      'w-full',
      'px-3 py-4',
      `bg-[${colour}]`,
      'text-xs leading-tight font-sans',
      `border-t border-[${borderColour}]`
    )

    return html`
      <div class=${styles}>
        <div class="flex items-center justify-center max-w-4xl w-full mx-auto">
          <span class="font-bold pr-2">Model:</span>
          <ui-select-input
            class="w-1/2"
            ?multi=${true}
            ?clearable=${true}
            .options=${[
              { value: 'chat-gpt' },
              { value: 'claude' },
              { value: 'ollama' },
              { value: 'gemini' },
            ]}
          >
          </ui-select-input>
          <div>
            <sl-dropdown placement="bottom-end" distance="20">
              <div slot="trigger" class="ml-4 cursor-pointer">
                <sl-tooltip
                  content="Model settings"
                  style="--show-delay: 500ms; --hide-delay: 100ms"
                >
                  <stencila-ui-icon
                    name="gear"
                    class="text-base"
                  ></stencila-ui-icon>
                </sl-tooltip>
              </div>
              <div>${this.renderDropdown()}</div>
            </sl-dropdown>
          </div>
        </div>
      </div>
    `
  }

  renderDropdown() {
    return html`
      <div class="p-4 bg-white border rounded min-w-[300px]">
        <sl-menu-label>Model selection weights</sl-menu-label>
        <div class="flex flex-row items-center">
          <div class="mr-2">
            <stencila-ui-icon
              class="text-lg"
              name="starFill"
            ></stencila-ui-icon>
          </div>
          <sl-range
            class="w-full"
            label="Quality"
            min="0"
            max="100"
            value=${this.qualityWeight}
            @sl-change=${(e: InputEvent) =>
              this.onSelectionWeightChange(e, 'qualityWeight')}
          ></sl-range>
        </div>
        <div class="flex flex-row items-center">
          <div class="mr-2">
            <stencila-ui-icon
              class="text-lg"
              name="speedometer"
            ></stencila-ui-icon>
          </div>
          <sl-range
            class="w-full"
            label="Speed"
            min="0"
            max="100"
            value=${this.speedWeight}
            @sl-change=${(e: InputEvent) =>
              this.onSelectionWeightChange(e, 'speedWeight')}
          ></sl-range>
        </div>
        <div class="flex flex-row items-center">
          <div class="mr-2">
            <stencila-ui-icon
              class="text-lg"
              name="currencyDollar"
            ></stencila-ui-icon>
          </div>
          <sl-range
            class="w-full"
            label="Cost"
            min="0"
            max="100"
            value=${this.costWeight}
            @sl-change=${(e: InputEvent) =>
              this.onSelectionWeightChange(e, 'costWeight')}
          ></sl-range>
        </div>
        <sl-divider></sl-divider>
        <sl-menu-label>Model inference temperature</sl-menu-label>
        <div class="flex flex-row items-center">
          <div class="mr-2">
            <stencila-ui-icon
              class="text-lg"
              name="thermometer"
            ></stencila-ui-icon>
          </div>
          <sl-range
            class="w-full"
            min="0"
            max="100"
            value=${this.temperature ?? 1}
          ></sl-range>
        </div>
      </div>
    `
  }
}
