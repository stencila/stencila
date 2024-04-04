import '@shoelace-style/shoelace/dist/components/icon/icon'
import { apply } from '@twind/core'
import { PropertyValueMap, html } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import { withTwind } from '../../../twind'
import '../../animation/collapsible'
import { UIBaseClass } from '../mixins/uiBaseClass'
import '../../buttons/chevron'

/**
 * UI in-flow-block
 *
 * A component to render a node-card "in flow" - i.e. renders a block as is
 * without requiring user interaction
 */
@customElement('stencila-ui-block-in-flow')
@withTwind()
export class UIBlockInFlow extends UIBaseClass {
  /**
   * Disables showing content if slot has no content.
   */
  @state()
  displayContent: boolean = false

  /**
   * Indicates whether we need to have border separating header items and the
   * expand/collapse button.
   */
  @state()
  hasHeaderContent: boolean = false

  /**
   * If the component can be collapsed, track whether it is in a collapsed state.
   */
  @property({ type: Boolean })
  collapsed?: boolean = false

  /**
   * Allows us to switch the animation on/off.
   */
  @property({ type: Boolean, attribute: 'can-animate' })
  canAnimate: boolean = true

  override render() {
    const cardStyles = apply([
      'group',
      'transition duration-400',
      'border border-[rgba(255,255,255,0)]',
      'rounded',
      this.view === 'source' ? 'flex flex-col h-full' : 'my-2',
      this.ui.borderColour && `border-[${this.ui.borderColour}]`,
    ])

    const animationClasses = `${!this.canAnimate ? 'no-animate' : ''} ${this.collapsed ? '' : 'opened'}`

    return html`<div class=${`${cardStyles}`}>
      <div class="relative">
        ${this.renderHeader()}

        <stencila-ui-collapsible-animation class=${animationClasses}>
          <div>${this.renderBody()} ${this.renderContent()}</div>
        </stencila-ui-collapsible-animation>
      </div>
    </div>`
  }

  private renderHeader() {
    const { iconLibrary, icon, title, borderColour } = this.ui
    const headerTitle = (this.title && this.title) || title

    const headerStyles = apply([
      'flex items-center',
      'w-full',
      'px-4 py-2',
      'gap-x-2',
      `bg-[${borderColour}]`,
      `border border-[${borderColour}]`,
      this.view === 'source' ? '' : 'rounded-t',
      'font-medium',
      'cursor-pointer',
      'transition duration-100 ease-in',
      `hover:contrast-[103%]`,
    ])

    return html`<div class=${headerStyles}>
      <div
        class="flex items-center gap-x-2 grow"
        @click=${() => {
          this.collapsed = !this.collapsed
        }}
      >
        <span class="items-center flex grow-0 shrink-0">
          <sl-icon
            library=${iconLibrary}
            name=${icon}
            class="text-2xl"
          ></sl-icon>
        </span>
        <div class="flex justify-between items-center gap-x-2 grow">
          <span class="font-bold grow">${headerTitle}</span>
          <div class="relative z-[3]">
            <slot name="header-right"></slot>
          </div>
          ${this.renderCollapse()}
        </div>
      </div>
    </div>`
  }

  private renderBody() {
    const { colour, borderColour } = this.ui
    const bodyStyles = apply([
      'relative',
      'w-full h-full',
      `bg-[${colour}]`,
      `border border-[${borderColour}] rounded-b`,
    ])

    return html`<div class=${bodyStyles}>
      <slot name="body"></slot>
    </div>`
  }

  private renderCollapse() {
    const classes = apply([
      'flex items-center',
      'ml-3',
      `border-[${this.ui.borderColour}] brightness-75`,
      this.hasHeaderContent && 'pl-3 border-l-2',
    ])
    return html`<div class=${classes}>
      <stencila-chevron-button
        position=${this.collapsed ? 'left' : 'down'}
        class="inline-flex"
      ></stencila-chevron-button>
    </div>`
  }

  private renderContent() {
    const contentStyles = apply([
      'flex',
      'relative',
      'transition-[padding] ease-in-out duration-[250ms]',
      'px-3',
    ])

    return html`<div class=${contentStyles}>
      <slot name="content"></slot>
    </div>`
  }

  protected override update(
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    changedProperties: PropertyValueMap<any> | Map<PropertyKey, unknown>
  ) {
    super.update(changedProperties)
    const slots: NodeListOf<HTMLSlotElement> = this.shadowRoot.querySelectorAll(
      'slot[name="content"], slot[name="header-right"]'
    )

    for (const slot of slots) {
      const hasItems = slot.assignedElements({ flatten: true }).length !== 0

      switch (slot.name) {
        case 'content':
          if (hasItems !== this.displayContent) {
            this.displayContent = hasItems
          }
          break
        case 'header-right':
          if (hasItems !== this.hasHeaderContent) {
            this.hasHeaderContent = hasItems
          }
          break
        default:
          break
      }
    }
  }
}
