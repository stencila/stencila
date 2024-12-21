import { NodeType } from '@stencila/types'
import { apply } from '@twind/core'
import { css, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { documentCommandEvent } from '../clients/commands'
import { data } from '../system'
import { withTwind } from '../twind'
import { iconMaybe } from '../ui/icons/icon'
import '../ui/inputs/select'
import { nodeUi } from '../ui/nodes/icons-and-colours'

import { Entity } from './entity'

import '@shoelace-style/shoelace/dist/components/dropdown/dropdown.js'
import '@shoelace-style/shoelace/dist/components/range/range.js'
import '@shoelace-style/shoelace/dist/components/divider/divider.js'

type ModelParametersWeightField = 'speedWeight' | 'costWeight' | 'qualityWeight'

/**
 * Web component representing a Stencila Schema `ModelParameters` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/edit/model-parameters.md
 */
@customElement('stencila-model-parameters')
@withTwind()
export class ModelParameters extends Entity {
  @property({ attribute: 'model-ids', type: Array })
  modelIds?: string[]

  @property({ type: Number })
  replicates?: number = 1

  @property({ attribute: 'quality-weight', type: Number })
  qualityWeight?: number = 70

  @property({ attribute: 'cost-weight', type: Number })
  costWeight?: number = 15

  @property({ attribute: 'speed-weight', type: Number })
  speedWeight?: number = 15

  @property({ attribute: 'minimum-score', type: Number })
  minimumScore?: number

  @property({ type: Number })
  temperature?: number

  @property({ attribute: 'random-seed', type: Number })
  randomSeed?: number

  private readonly weightFields: ModelParametersWeightField[] = [
    'costWeight',
    'qualityWeight',
    'speedWeight',
  ]

  /**
   * On a change to the global list of models request an
   * update (re-render) of this component
   */
  private onModelsUpdated() {
    this.requestUpdate()
  }

  /**
   * On a change to a weight, adjust the other weights so that they
   * all sum to 100 and then send a patch update all the weights.
   */
  private onWeightChanged(
    event: InputEvent,
    changedWeight: ModelParametersWeightField
  ) {
    const newValue = parseInt((event.target as HTMLInputElement).value)

    const remainingWeight = 100 - newValue
    const otherWeights = this.weightFields.filter(
      (weight) => weight !== changedWeight
    )
    const otherSum = this[otherWeights[0]] + this[otherWeights[1]]

    // Adjust other weights to guarantee integers that sum to 100
    let firstWeight
    let secondWeight
    if (otherSum > 0) {
      // Distribute the remaining weight proportionally to the other weights
      firstWeight = Math.round(
        (this[otherWeights[0]] / otherSum) * remainingWeight
      )
      secondWeight = remainingWeight - firstWeight
    } else {
      // If otherSum is zero, distribute equally
      firstWeight = Math.floor(remainingWeight / 2)
      secondWeight = remainingWeight - firstWeight
    }

    this[otherWeights[0]] = firstWeight
    this[otherWeights[1]] = secondWeight
    this[changedWeight] = newValue

    // Send patch for all weights
    // TODO: create/modify command so can send a patch with multiple operations
    // rather than send 3 separate patches as done here
    for (const weight of this.weightFields) {
      this.dispatchEvent(
        documentCommandEvent({
          command: 'patch-node',
          nodeType: 'ModelParameters',
          nodeIds: [this.id],
          nodeProperty: [weight, this[weight]],
        })
      )
    }
  }

  /**
   * On a change to a number property, patch that property in the document
   */
  private onPropertyChanged(
    event: InputEvent,
    property: 'minimumScore' | 'replicates' | 'temperature'
  ) {
    this[property] = parseInt((event.target as HTMLInputElement).value)

    this.dispatchEvent(
      documentCommandEvent({
        command: 'patch-node',
        nodeType: 'ModelParameters',
        nodeIds: [this.id],
        nodeProperty: [property, this[property]],
      })
    )
  }

  override connectedCallback() {
    super.connectedCallback()
    data.addEventListener('models', this.onModelsUpdated.bind(this))
  }

  override disconnectedCallback() {
    super.disconnectedCallback()
    data.removeEventListener('models', this.onModelsUpdated.bind(this))
  }

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

  override render() {
    const parentNodeType = this.ancestors.split('.').pop() as NodeType
    const { borderColour, colour } = nodeUi(parentNodeType)
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
        <div class="flex flex-row items-center justify-between w-full">
          <div class="flex flex-row items-center w-11/12">
            <span class="pr-2">Model</span>
            <ui-select-input
              class="w-full"
              ?multi=${true}
              ?clearable=${true}
              .options=${data.models.map((model) => ({
                value: model.id,
                icon: iconMaybe(model.provider.toLowerCase()) ?? 'robot',
                label: `${model.name} ${model.version}`,
              }))}
            >
            </ui-select-input>
          </div>
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
        <sl-menu-label>Model selection: weights</sl-menu-label>
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
              this.onWeightChanged(e, 'qualityWeight')}
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
              this.onWeightChanged(e, 'speedWeight')}
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
              this.onWeightChanged(e, 'costWeight')}
          ></sl-range>
        </div>

        <sl-divider></sl-divider>

        <sl-menu-label>Model selection: minimum score</sl-menu-label>
        <div class="flex flex-row items-center">
          <div class="mr-2">
            <stencila-ui-icon
              class="text-lg"
              name="speedometer"
            ></stencila-ui-icon>
          </div>
          <sl-range
            class="w-full"
            min="0"
            max="100"
            value=${this.minimumScore ?? 1}
            @sl-change=${(e: InputEvent) =>
              this.onPropertyChanged(e, 'minimumScore')}
          ></sl-range>
        </div>

        <sl-divider></sl-divider>

        <sl-menu-label>Replicate runs per model</sl-menu-label>
        <div class="flex flex-row items-center">
          <div class="mr-2">
            <stencila-ui-icon
              class="text-lg"
              name="arrowClockwise"
            ></stencila-ui-icon>
          </div>
          <sl-range
            class="w-full"
            min="1"
            max="10"
            value=${this.replicates ?? 1}
            @sl-change=${(e: InputEvent) =>
              this.onPropertyChanged(e, 'replicates')}
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
            value=${this.temperature ?? 0}
            @sl-change=${(e: InputEvent) =>
              this.onPropertyChanged(e, 'temperature')}
          ></sl-range>
        </div>
      </div>
    `
  }
}
