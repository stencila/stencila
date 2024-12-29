import { apply, css } from '@twind/core'
import { html, TemplateResult } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { patchValue } from '../clients/commands'
import { data, Model } from '../system'
import { withTwind } from '../twind'
import { iconMaybe } from '../ui/icons/icon'
import { NodeTypeUI, nodeUi } from '../ui/nodes/icons-and-colours'

import { Entity } from './entity'

import '@shoelace-style/shoelace/dist/components/divider/divider.js'
import '@shoelace-style/shoelace/dist/components/dropdown/dropdown.js'
import '@shoelace-style/shoelace/dist/components/option/option.js'
import '@shoelace-style/shoelace/dist/components/range/range.js'
import '@shoelace-style/shoelace/dist/components/select/select.js'

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
  modelIds?: string[] = []

  @property({ type: Number })
  replicates?: number

  @property({ attribute: 'quality-weight', type: Number })
  qualityWeight?: number

  @property({ attribute: 'cost-weight', type: Number })
  costWeight?: number

  @property({ attribute: 'speed-weight', type: Number })
  speedWeight?: number

  @property({ attribute: 'minimum-score', type: Number })
  minimumScore?: number

  @property({ type: Number })
  temperature?: number

  @property({ attribute: 'random-seed', type: Number })
  randomSeed?: number

  /**
   * UI settings of the parent node type
   *
   * Instantiated in `connectedCallback` to avoid getting on each render.
   */
  private parentNodeUI: NodeTypeUI

  /**
   * Model <select> options updated whenever model list is updated
   * rather than in `render()`
   */
  private modelOptions: TemplateResult[] = []

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
    // Group models by provider
    const providers: Record<string, Model[]> = {}
    for (const model of data.models) {
      if (model.provider in providers) {
        providers[model.provider].push(model)
      } else {
        providers[model.provider] = [model]
      }
    }

    const { textColour } = this.parentNodeUI

    // Render options
    this.modelOptions = Object.entries(providers).map(
      ([provider, models], index) => {
        return html`
          ${index !== 0 ? html`<sl-divider class="my-1"></sl-divider>` : ''}
          <div
            class="flex flex-row items-center gap-2 px-2 py-1 text-[${textColour}]"
          >
            <stencila-ui-icon
              slot="prefix"
              class="text-base"
              name=${iconMaybe(provider.toLowerCase()) ?? 'building'}
            ></stencila-ui-icon>
            ${provider}
          </div>
          ${models.map(
            (model) => html`
              <sl-option
                value=${model.id}
                style="--sl-spacing-x-small: 0.25rem;"
              >
                <span class="text-xs text-[${textColour}]">${model.id}</span>
              </sl-option>
            `
          )}
        `
      }
    )

    this.requestUpdate()
  }

  /**
   * On a change to the selected models send a patch to update the property
   */
  private onModelIdsChanged(event: InputEvent) {
    const value = (event.target as HTMLInputElement).value

    this.dispatchEvent(
      patchValue('ModelParameters', this.id, 'modelIds', value)
    )
  }

  /**
   * Event handler for changes quality, cost and speed weight properties
   */
  private onWeightChanged(
    event: InputEvent,
    changedWeight: ModelParametersWeightField
  ) {
    const newValue = parseInt((event.target as HTMLInputElement).value)

    this.balanceWeights(changedWeight, newValue)

    // Send patch for all weights
    // TODO: create/modify command so can send a patch with multiple operations
    // rather than send 3 separate patches as done here
    for (const weight of this.weightFields) {
      this.dispatchEvent(
        patchValue('ModelParameters', this.id, weight, this[weight])
      )
    }
  }

  /**
   * Intercept the native attributeChangedCallback to detect external (DOM)
   * changes to weight attributes and balance weights accordingly.
   *
   * Equivalent to `onWeightChanged` but for externally trigged changes,
   * not changes due to interactions with this component.
   */
  override attributeChangedCallback(
    name: string,
    oldValue: string | null,
    newValue: string | null
  ) {
    super.attributeChangedCallback(name, oldValue, newValue)

    if (name.endsWith('-weight')) {
      const weightField = name.replace(
        '-weight',
        'Weight'
      ) as ModelParametersWeightField
      if (newValue !== null)
        this.balanceWeights(weightField, parseInt(newValue))
      else this.balanceWeights('qualityWeight', this.qualityWeight)
    }
  }

  /**
   * On a change to a weight, adjust the other weights so that they
   * all sum to 100 and then send a patch to update each of the weights.
   */
  private balanceWeights(
    changedWeight: ModelParametersWeightField,
    newValue: number
  ) {
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
  }

  /**
   * On a change to a number property, send a patch to update that property
   */
  private onPropertyChanged(
    event: InputEvent,
    property: 'minimumScore' | 'replicates' | 'temperature'
  ) {
    this[property] = parseInt((event.target as HTMLInputElement).value)

    this.dispatchEvent(
      patchValue('ModelParameters', this.id, property, this[property])
    )
  }

  override connectedCallback() {
    super.connectedCallback()

    this.parentNodeUI = nodeUi(this.parentNodeType)

    data.addEventListener('models', this.onModelsUpdated.bind(this))
  }

  override disconnectedCallback() {
    super.disconnectedCallback()
    data.removeEventListener('models', this.onModelsUpdated.bind(this))
  }

  override render() {
    // Model id strings written by the user may be partial, so here match them with
    // the id in the model list. This is the same as done in Rust.
    const modelIds: string[] = []
    for (const modelId of this.modelIds ?? []) {
      for (const model of data.models) {
        if (model.id.includes(modelId)) {
          modelIds.push(model.id)
          break
        }
      }
    }

    const { colour, textColour, borderColour } = this.parentNodeUI

    const styles = apply(
      'flex flex-row items-center gap-x-2',
      'w-full',
      'px-3 py-1',
      `bg-[${colour}]`,
      `text-[${textColour}] text-xs leading-tight font-sans`,
      `border-t border-[${borderColour}]`
    )

    const selectStyles = css`
      &::part(tag__base) {
        background-color: white;
        border-color: ${borderColour};
        color: ${textColour};
      }
      &::part(clear-button) {
        color: ${textColour};
      }
    `

    return html`
      <div class=${styles}>
        Model
        <sl-select
          class="w-full ${selectStyles}"
          multiple
          clearable
          max-options-visible="2"
          size="small"
          value=${modelIds.join(' ')}
          @sl-change=${(e: InputEvent) => this.onModelIdsChanged(e)}
        >
          ${this.modelOptions}
        </sl-select>
        <sl-dropdown placement="bottom-end" distance="20">
          <div slot="trigger" class="cursor-pointer">
            <sl-tooltip
              content="Model settings"
              style="--show-delay: 500ms; --hide-delay: 100ms"
            >
              <stencila-ui-icon
                name="sliders"
                class="text-base"
              ></stencila-ui-icon>
            </sl-tooltip>
          </div>
          ${this.renderDropdown()}
        </sl-dropdown>
      </div>
    `
  }

  renderDropdown() {
    const { borderColour, textColour, colour } = this.parentNodeUI

    const headerClasses = apply(
      'flex flex-row items-center gap-2 mt-6 mb-2 text-xs'
    )
    const weightsClasses = apply('items-center my-1 w-full')
    const rangeStyle = `
      --sl-input-label-font-size-medium: 0.75rem;
      --sl-color-primary-600: ${textColour};
      --sl-color-primary-500: ${borderColour};
      --track-color-active: ${borderColour};
      --track-color-inactive: ${colour};
    `

    const help = (content: string) =>
      html`<sl-tooltip content=${content}>
        <stencila-ui-icon
          class="text-sm"
          name="questionCircle"
        ></stencila-ui-icon>
      </sl-tooltip>`

    return html`
      <div class="border rounded border-[${borderColour}] bg-white">
        <div class="bg-[${colour}]/20 min-w-[300px] p-4">
          <span class="${headerClasses} mt-0">
            <stencila-ui-icon
              class="text-lg"
              name="speedometer"
            ></stencila-ui-icon>
            Model selection weights
            ${help(
              'Weights used for selecting a model. Only apply if a model router is used.'
            )}
          </span>
          <div class="flex items-center gap-2">
            <span class="text-xs w-[7ch]">Quality</span>
            <sl-range
              class=${weightsClasses}
              style=${rangeStyle}
              min="0"
              max="100"
              value=${this.qualityWeight}
              @sl-change=${(e: InputEvent) =>
                this.onWeightChanged(e, 'qualityWeight')}
            ></sl-range>
          </div>
          <div class="flex items-center gap-2">
            <span class="text-xs w-[7ch]">Cost</span>
            <sl-range
              class=${weightsClasses}
              style=${rangeStyle}
              min="0"
              max="100"
              value=${this.costWeight}
              @sl-change=${(e: InputEvent) =>
                this.onWeightChanged(e, 'costWeight')}
            ></sl-range>
          </div>
          <div class="flex items-center gap-2">
            <span class="text-xs w-[7ch]">Speed</span>
            <sl-range
              class=${weightsClasses}
              style=${rangeStyle}
              min="0"
              max="100"
              value=${this.speedWeight}
              @sl-change=${(e: InputEvent) =>
                this.onWeightChanged(e, 'speedWeight')}
            ></sl-range>
          </div>

          <span class=${headerClasses}>
            <stencila-ui-icon
              class="text-lg"
              name="arrowBarUp"
            ></stencila-ui-icon>
            Model selection minimum score
            ${help(
              'Minimum weighted score for random model selection. Use 100 to always select the highest scoring model. Only applies if a model router is used.'
            )}
          </span>
          <sl-range
            class="w-full"
            style=${rangeStyle}
            min="0"
            max="100"
            value=${this.minimumScore ?? 100}
            @sl-change=${(e: InputEvent) =>
              this.onPropertyChanged(e, 'minimumScore')}
          ></sl-range>

          <span class=${headerClasses}>
            <stencila-ui-icon
              class="text-lg"
              name="thermometer"
            ></stencila-ui-icon>
            Model inference temperature
            ${help(
              'Amount of randomness in model suggestions. Use higher values for less analytical, more creative responses.'
            )}
          </span>
          <sl-range
            class="w-full"
            style=${rangeStyle}
            min="0"
            max="100"
            value=${this.temperature ?? 50}
            @sl-change=${(e: InputEvent) =>
              this.onPropertyChanged(e, 'temperature')}
          ></sl-range>

          <span class=${headerClasses}>
            <stencila-ui-icon
              class="text-lg"
              name="arrowRepeat"
            ></stencila-ui-icon>
            Suggestions per model
            ${help('Number of suggestions made by each model')}
          </span>
          <sl-range
            class="w-full"
            style=${rangeStyle}
            min="1"
            max="10"
            value=${this.replicates ?? 1}
            @sl-change=${(e: InputEvent) =>
              this.onPropertyChanged(e, 'replicates')}
          ></sl-range>
        </div>
      </div>
    `
  }
}
