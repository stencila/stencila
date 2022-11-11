import { SlRadioGroup } from '@shoelace-style/shoelace'
import { css, html } from 'lit'
import { customElement, property, state } from 'lit/decorators'
import { isCodeWriteable } from '../../mode'
import StencilaInput from '../base/input'
import { twSheet } from '../utils/css'

import StencilaExecutable from './executable'

const { tw, sheet } = twSheet()

/**
 * A component for a Stencila `Form` node
 */
@customElement('stencila-form')
export default class StencilaForm extends StencilaExecutable {
  static styles = [
    sheet.target,
    css`
      sl-radio::part(base) {
        font-size: 0.9em;
      }
    `,
  ]

  static color = 'blue'

  static formats = ['markdown', 'yaml', 'json']

  /**
   * The `Parameter.derivFrom` property
   */
  @property({ attribute: 'derive-from', reflect: true })
  deriveFrom: string

  /**
   * The `Parameter.deriveAction` property
   */
  @property({ attribute: 'derive-action', reflect: true })
  deriveAction: string

  /**
   * The `Parameter.deriveItem` property
   */
  @property({ attribute: 'derive-item', reflect: true })
  deriveItem: string

  /**
   * Whether or not the `content` slot has content
   */
  @state()
  private hasContent = false

  /**
   * An observer to update `hasContent`
   */
  private contentObserver: MutationObserver

  /**
   * Handle a change to the `content` slot
   *
   * Initializes `hasContent` and a `MutationObserver` to watch for changes
   * to the number of elements in the slot and change `hasContent` accordingly.
   */
  private onContentSlotChange(event: Event) {
    const contentElem = (event.target as HTMLSlotElement).assignedElements({
      flatten: true,
    })[0] as HTMLElement | undefined

    if (contentElem) {
      this.hasContent = contentElem.childElementCount > 0

      this.contentObserver = new MutationObserver(() => {
        this.hasContent = contentElem.childElementCount > 0
      })
      this.contentObserver.observe(contentElem, {
        childList: true,
      })
    }
  }

  protected renderDeriveFromInput() {
    const readOnly = !isCodeWriteable()

    const update = (event: Event) => {
      const input = event.target as StencilaInput

      let deriveFrom: string | undefined = input.getValue().trim()
      if (deriveFrom.length == 0) {
        deriveFrom = undefined
      }

      if (
        (event.type === 'sl-change' || event.type === 'stencila-ctrl-enter') &&
        input.isValid()
      ) {
        this.changeProperty('deriveFrom', deriveFrom)
      }
    }

    return html`<stencila-input
      type="text"
      label="Derive from"
      size="small"
      value=${this.deriveFrom}
      ?disabled=${readOnly}
      @sl-input=${update}
      @sl-change=${update}
      @stencila-ctrl-enter=${(event: Event) => {
        update(event)
        this.assemble()
      }}
    ></stencila-input>`
  }

  protected renderDeriveAction() {
    const readOnly = !isCodeWriteable()

    const update = (event: Event) => {
      const input = event.target as SlRadioGroup
      if (input.value !== this.deriveAction) {
        this.changeProperty('deriveAction', input.value)
      }
    }

    return html`<div>Derive action</div>
      <sl-radio-group
        label="Derive action"
        value=${this.deriveAction}
        @sl-change=${update}
      >
        <sl-radio size="small" value="Create" ?disabled=${readOnly}
          >Create</sl-radio
        >
        <sl-radio size="small" value="Update" ?disabled=${readOnly}
          >Update</sl-radio
        >
        <sl-radio size="small" value="Delete" ?disabled=${readOnly}
          >Delete</sl-radio
        >
        <sl-radio size="small" value="UpdateOrDelete" ?disabled=${readOnly}
          >Update or delete</sl-radio
        >
      </sl-radio-group>`
  }

  protected renderSettingsDropdown() {
    // Note that if `derivedFrom` is set then `default` and `validator`
    // should be read only.
    const readOnly = !isCodeWriteable() || this.deriveFrom?.length > 0

    return html`<sl-dropdown
      class=${tw`ml-1`}
      distance="10"
      placement="bottom-end"
    >
      <stencila-icon-button
        slot="trigger"
        name="three-dots-vertical"
        color=${StencilaForm.color}
      ></stencila-icon-button>
      <div
        class=${tw`flex flex-col gap-2 rounded border(& ${StencilaForm.color}-200)
            bg-${StencilaForm.color}-50 p-2 text(sm ${StencilaForm.color}-700)`}
      >
        ${this.renderDeriveFromInput()} ${this.renderDeriveAction()}
      </div>
    </sl-dropdown>`
  }

  protected renderErrorsContainer() {
    return html`<div
      part="errors"
      class=${this.hasErrors
        ? tw`border(t ${StencilaForm.color}-200)`
        : tw`hidden`}
    >
      <slot
        name="errors"
        @slotchange=${(event: Event) => this.onErrorsSlotChange(event)}
      ></slot>
    </div>`
  }

  protected renderContentContainer() {
    return html`<div
      part="content"
      class=${this.hasContent
        ? tw`border(t ${StencilaForm.color}-200) p-2 `
        : tw`hidden`}
    >
      ${!this.hasContent
        ? html`<p class=${tw`text(center gray-300)`}>No content</p>`
        : ''}
      <slot
        name="content"
        @slotchange=${(event: Event) => this.onContentSlotChange(event)}
      ></slot>
    </div>`
  }

  protected render() {
    const toggleSelected = () => this.toggleSelected()

    return html`<div
      part="base"
      class=${tw`my-4 rounded overflow-hidden border(& ${
        StencilaForm.color
      }-200) ${this.selected ? `ring-1` : ''}`}
      @mousedown=${toggleSelected}
    >
      <div
        part="header"
        class=${tw`flex items-center justify-between bg-${StencilaForm.color}-50 p-1
                   font(mono bold) text(sm ${StencilaForm.color}-700)`}
      >
        <span class=${tw`flex items-center`}>
          <span class=${tw`flex items-center text-base mr-2`}>
            <stencila-icon name="pencil-square"></stencila-icon>
          </span>
          <span class=${tw`mr-2`}>form</span>
        </span>
        <span class=${tw`flex items-center`}>
          ${this.renderSettingsDropdown()}
          ${this.renderExpandButton(tw, StencilaForm.color)}
        </span>
      </div>

      ${this.renderErrorsContainer()} ${this.renderContentContainer()}

      <div
        part="footer"
        class=${tw`grid justify-items-end items-center bg-${StencilaForm.color}-50
                   border(t ${StencilaForm.color}-200) p-1 text(sm ${StencilaForm.color}-700)`}
      >
        ${this.renderDownloadButton(StencilaForm.formats, StencilaForm.color)}
      </div>
    </div>`
  }
}
