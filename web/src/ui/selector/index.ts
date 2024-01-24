import { apply, css } from '@twind/core'
import { html } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'
import { Ref, createRef, ref } from 'lit/directives/ref.js'

import type { DocumentView } from '../../types'
import { TWLitElement } from '../twind'

/**
 * UI selector
 *
 * A selector that updates some display portion of the UI
 */
@customElement('stencila-ui-selector')
export class UISelector extends TWLitElement {
  /**
   * Ref to allow us to close the details element when needed.
   */
  detailsRef: Ref<HTMLDetailsElement> = createRef()

  /**
   * Manages the open state of the open listbox
   */
  @state()
  private open: boolean = false

  /**
   * Label displayed when listbox is not open
   */
  @property()
  label: string = ''

  /**
   * List of values to render in the list
   */
  @property({ type: Array })
  list: [string, string][] = []

  /**
   * Event to call when a list element is selected
   */
  @property()
  clickEvent: (e: Event) => void | undefined

  /**
   * Target property in parent component to evaluate
   */
  @property()
  target: DocumentView | string

  override render() {
    return html`
      ${this.renderOverlay()}
      <details
        role="list"
        class="group text-gray-aluminium p-0 relative block flex-grow open:text-brand-blue open:border-b-brand-blue open:z-50"
        ${ref(this.detailsRef)}
      >
        ${this.renderSummary()} ${this.renderList()}
      </details>
    `
  }

  private renderSummary() {
    const styles = apply([
      'text-base font-bold',
      'leading-none',
      'select-none',
      'appearance-none ',
      'min-w-fit',
      'py-2 px-4',
      'bg-white',
      'border-b-4 border-b-transparent',
      'transition-all ease-in-out',
      'flex',
      'items-center',
      'group-hover:text-brand-blue group-hover:border-b-brand-blue',
    ])

    const hideMarker = css`
      &::marker {
        display: none;
        font-size: 0;
      }
    `

    return html`<summary
      aria-haspopup="listbox"
      role="button"
      class="${styles} ${hideMarker}"
      @click=${this.setOpen}
    >
      <span class="mr-2">${this.label}</span>${this.renderCarat()}
    </summary>`
  }

  private renderOverlay() {
    return this.open
      ? html`<div
          class="w-screen h-screen fixed z-10 top-0 left-0"
          @click=${this.toggleOverlay}
          aria-hidden="true"
        ></div>`
      : null
  }

  private renderCarat() {
    const styles = apply([
      'inline-block',
      'w-2',
      'h-2',
      '-mt-0.5',
      'text-aluminium',
      'transform',
      'rotate-45',
      'transition-all',
      'group-open:rotate-[225deg] group-open:mt-1',
    ])

    return html`<span class=${styles} aria-hidden="true">
      <i class="w-full h-full border-r-2 border-b-2 block"></i>
    </span>`
  }

  private renderList() {
    const styles = apply([
      'block',
      'py-2',
      'rounded-b-md border-t-4 border-t-brand-blue',
      'shadow-[0_8px_8px_hsla(0,0%,4%,.1)]',
      'absolute top-8',
      'flex flex-col',
      'bg-white',
    ])

    return html`<ul role="listbox" class=${styles}>
      ${this.list.map(([value, label]) => this.renderListItem(value, label))}
    </ul>`
  }

  private renderListItem(value: string, label: string) {
    const styles = apply([
      'block',
      'w-full',
      'py-2 pl-4 pr-12',
      'bg-white',
      'text-sm font-bold text-gray-aluminium',
      'text-left',
      'border-l-4',
      this.target === value ? 'border-brand-blue' : 'border-transparent',
      'hover:bg-gray-wild-sand hover:text-black',
    ])

    return html`<li class="block whitespace-nowrap">
      <button
        data-value="${value}"
        class=${styles}
        @click=${(e: Event) => {
          this.toggleOverlay()
          this.clickEvent && this.clickEvent(e)
        }}
      >
        ${label}
      </button>
    </li>`
  }

  private setOpen() {
    this.open = !this.open
  }

  private toggleOverlay() {
    this.setOpen()
    if (!this.open && this.detailsRef.value !== undefined) {
      this.detailsRef.value.open = false
    }
  }
}
