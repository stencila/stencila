import { apply } from '@twind/core'
import { css, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { data } from '../system'
import { withTwind } from '../twind'
import { iconMaybe } from '../ui/icons/icon'
import '../ui/inputs/select'
import { nodeUi } from '../ui/nodes/icons-and-colours'

import { Entity } from './entity'

import '@shoelace-style/shoelace/dist/components/dropdown/dropdown.js'
import '@shoelace-style/shoelace/dist/components/range/range.js'
import '@shoelace-style/shoelace/dist/components/divider/divider.js'

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

  override connectedCallback() {
    super.connectedCallback()
    data.addEventListener('models', this.onModelsUpdated.bind(this))
  }

  override disconnectedCallback() {
    super.disconnectedCallback()
    data.removeEventListener('models', this.onModelsUpdated.bind(this))
  }

  onModelsUpdated() {
    this.requestUpdate()
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
            .options=${data.models.map((model) => ({
              value: model.id,
              icon: iconMaybe(model.provider.toLowerCase()) ?? 'robot',
              label: `${model.name} ${model.version}`,
            }))}
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
