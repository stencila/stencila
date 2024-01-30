import { apply, css } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'
import { Ref, ref, createRef } from 'lit/directives/ref'

import { withTwind } from '../../twind'

/**
 * UI Icon button
 *
 * A button rendered with an icon (as seen in the application chrome).
 */
@customElement('stencila-ui-icon-button')
@withTwind()
export class UIIconButton extends LitElement {
  /**
   * The ref used by the draggable slider.
   */
  private ref: Ref<HTMLElement> = createRef()

  /**
   * Name of the custom icon to use
   */
  @property()
  icon: string

  /**
   * Any custom classes to pass to the button element
   */
  @property()
  customClasses?: string

  /**
   * Disable interaction with this button
   */
  @property({ type: Boolean })
  disabled: boolean = false

  /**
   * When this button has been clicked, it should be displayed as active.
   */
  @property({ type: Boolean })
  active: boolean = false

  /**
   * Internally manage the active state.
   */
  @state()
  isActive: boolean

  constructor() {
    super()

    this.isActive = this.active
  }

  override render() {
    const classes = apply(['group', this.disabled ? 'pointer-events-none' : ''])
    const styles = css`
      &::part(base) {
        border: none;
        line-height: 0;
        min-height: 0;
        background: none;
      }

      &::part(label) {
        padding: 0;
      }
    `

    return html`<sl-button
      class="${classes} ${styles} ${this.customClasses}"
      ${ref(this.ref)}
      >${this.renderIcon(this.icon)}</sl-button
    >`
  }

  /**
   * Render the icon & apply appropriate styles.
   */
  private renderIcon(icon: string) {
    const state = this.getButtonState()
    const stateColour = {
      disabled: 'fill-grey-200',
      active: 'fill-brand-blue',
      default: 'fill-grey-700',
    }
    const classes = apply([
      'text-xl',
      'transition-all duration-300 ease-in-out',
      'stroke-none',
      stateColour[state],
      state !== 'active' ? 'group-hover:fill-grey-900' : '',
    ])

    return html`<sl-icon
      library="stencila"
      name="${icon}"
      class="${classes}"
    ></sl-icon>`
  }

  /**
   * Find the button's state - disabled, active or 'default'.
   */
  private getButtonState() {
    if (this.disabled) {
      return 'disabled'
    }

    if (this.isActive) {
      return 'active'
    }

    return 'default'
  }

  /**
   * Add a click event to manage change of active state
   */
  override firstUpdated() {
    this.ref.value.addEventListener('click', () => {
      this.isActive = !this.isActive
    })
  }
}
