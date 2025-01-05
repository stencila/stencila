import { apply } from '@twind/core'
import { html, PropertyValueMap } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

import { ChatMessage } from '../../../nodes/chat-message'
import { SuggestionBlock } from '../../../nodes/suggestion-block'
import { withTwind } from '../../../twind'
import { IconName } from '../../icons/icon'
import { UIBaseClass } from '../mixins/ui-base-class'

import '../../animation/collapsible'
import '../../buttons/chevron'

/**
 * UI Base Card
 *
 * Provides base rendering options for a node-card (as implemented via `in-flow` and `on-demand` instances).
 */
@customElement('stencila-ui-base-card')
@withTwind()
export class UIBaseCard extends UIBaseClass {
  /**
   * The icon to use in the header
   */
  @property({ attribute: 'header-icon' })
  headerIcon?: IconName

  /**
   * The title to use in the header
   */
  @property({ attribute: 'header-title' })
  headerTitle?: string

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

  /**
   * The collapsable/accordion style component can be disabled via this
   * property. If set to false, the header can no longer be collapsed.
   */
  @property({ type: Boolean, attribute: 'can-collapse' })
  canCollapse?: boolean = false

  /**
   * Indicates whether we need to have border separating header items and the
   * expand/collapse button.
   */
  @state()
  hasHeaderContent: boolean = false

  /**
   * Disables showing content if slot has no content.
   */
  @state()
  displayContent: boolean = false

  /**
   * Allow for child classes to determine if the title of the card should be
   * restricted (i.e. rendered with an overflow-ellipsis) - helpful for
   * tooltips.
   */
  protected restrictTitleWidth: boolean = false

  /**
   * Renders the header element of the card
   * @param {string[]} extraTwindClasses additional `twind` classes for the header container
   * @returns
   */
  protected renderHeader(extraTwindClasses?: string | string[]) {
    const { title, borderColour } = this.ui
    const headerTitle = this.headerTitle ? this.headerTitle : title

    const additionalStyles: string[] = extraTwindClasses
      ? Array.isArray(extraTwindClasses)
        ? extraTwindClasses
        : [extraTwindClasses]
      : []

    const headerStyles = apply([
      `font-sans not-italic text-[${this.ui.textColour}]`,
      'flex items-center',
      'w-full',
      'px-4 py-2',
      'gap-x-2',
      'rounded-t-sm',
      `bg-[${borderColour}]`,
      'font-medium',
      'transition duration-100 ease-in',
      this.canCollapse && `cursor-pointer hover:bg-[${borderColour}]/90`,
      ...additionalStyles,
    ])

    const canClose =
      this.depth > 0 &&
      !(
        ChatMessage.shouldExpand(this, this.type) ||
        SuggestionBlock.shouldExpand(this, this.type)
      )

    const icon = this.headerIcon ?? this.ui.icon

    return html`<div class=${headerStyles}>
      <div
        class="flex items-center gap-x-2 grow"
        @click=${() => {
          if (!this.canCollapse) {
            return
          }
          this.toggleCardBodyCollapse()
        }}
      >
        <span class="items-center flex grow-0 shrink-0">
          <stencila-ui-icon name=${icon} class="text-xl"></stencila-ui-icon>
        </span>
        <div class="flex justify-between items-center gap-x-2 grow">
          <div
            class=${`flex grow pr-4 ${this.restrictTitleWidth ? 'max-w-[22rem]' : ''}`}
          >
            <span
              class="font-semibold text-sm inline-block overflow-hidden text-ellipsis whitespace-nowrap"
              >${headerTitle}</span
            >
          </div>
          <div class="relative z-[3]">
            <slot name="header-right"></slot>
          </div>
          ${this.canCollapse ? this.renderCollapse() : null}
          ${canClose ? this.renderClose() : ''}
        </div>
      </div>
    </div>`
  }

  /**
   * Render the collapse card icon on the right hand side of the header.
   */
  private renderCollapse() {
    const classes = apply([
      'flex items-center',
      'ml-3',
      `border-[${this.ui.borderColour}] brightness-75`,
      this.hasHeaderContent && 'pl-3 border-l-2',
    ])

    return html`<div class=${classes}>
      <stencila-ui-chevron-button
        default-pos=${this.collapsed ? 'left' : 'down'}
        .disableEvents=${true}
        class="inline-flex"
      ></stencila-ui-chevron-button>
    </div>`
  }

  protected renderClose() {
    return html``
  }

  protected renderBody() {
    return html``
  }

  protected renderContent() {
    return html``
  }

  /**
   * Displays the content, wrapped in a `collapsible-animation` component.
   */
  protected renderAnimatedCardBody() {
    const animationClasses = `${!this.canAnimate ? 'no-animate' : ''} ${this.collapsed ? '' : 'opened'}`

    return html`<stencila-ui-collapsible-animation class=${animationClasses}>
      <div>${this.renderBody()}</div>
    </stencila-ui-collapsible-animation>`
  }

  /**
   * This function is called when the `collapse` click event is triggered.
   */
  protected toggleCardBodyCollapse(): void {
    this.collapsed = !this.collapsed
  }

  /**
   * on update, check whether there are header or content slots available.
   */
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
            // TODO: consider al alternative to disabling this lint error
            // eslint-disable-next-line lit/no-property-change-update
            this.displayContent = hasItems
          }
          break
        case 'header-right':
          if (hasItems !== this.hasHeaderContent) {
            // TODO: consider al alternative to disabling this lint error
            // eslint-disable-next-line lit/no-property-change-update
            this.hasHeaderContent = hasItems
          }
          break
        default:
          break
      }
    }
  }
}
