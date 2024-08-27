import { consume } from '@lit/context'
import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, state } from 'lit/decorators.js'
import { Ref, createRef, ref } from 'lit/directives/ref'

import { withTwind } from '../../../twind'
import {
  DocPreviewContext,
  documentPreviewContext,
} from '../../document/context'

import '../../animation/collapsible'

/**
 * UI Authors and Provenance
 *
 * A collapsible section displayed at the top of the page to show authors &
 * provenance.
 */
@customElement('stencila-ui-authors-provenance')
@withTwind()
export class UIAuthorsProvenance extends LitElement {
  @consume({ context: documentPreviewContext, subscribe: true })
  @state()
  context: DocPreviewContext

  @state()
  collapsed: boolean = true

  /**
   * ref used to manage mouse events.
   */
  private buttonRef: Ref<HTMLDivElement> = createRef()

  protected override render() {
    return html`
      <div
        class=${`${this.context.showAuthorProvenance ? 'mb-8' : 'mb-0'} transition-all ease-in duration-200 group pointer-events-none`}
      >
        <stencila-ui-collapsible-animation
          class=${`pointer-events-auto ${this.context.showAuthorProvenance ? 'opened' : ''}`}
        >
          ${this.renderBody()}
        </stencila-ui-collapsible-animation>
      </div>
    `
  }

  /**
   * Output the header element which acts as a event handler for click events.
   */
  // @ts-expect-error TODO determine if this unused function is still needed
  private renderHeader() {
    const collapsedClasses = !this.context.showAuthorProvenance
      ? [
          'text-black/50',
          'border-t-black/20 border-l-black/0 border-r-black/0 rounded-t-none',
        ]
      : [
          'text-black/20',
          'border-l-black/0 border-t-black/0 border-r-black/0 rounded-t',
          'hover:bg-[rgba(0,0,0,0.025)] hover:contrast-[105%]',
          'group-hover:border-l-black group-hover:border-r-black',
        ]
    const containerClasses = apply([
      ...collapsedClasses,
      [
        'relative z-[1]',
        'flex gap-x-2 items-center',
        'border border-b-white group-hover:border-t-black',
        'group-hover:text-black',
        'p-2 -mb-px',
        'max-w-fit',
        'transform-gpu',
        'transition-all duration-200',
        'cursor-pointer pointer-events-auto',
        'after:content-[""] after:block',
        'after:w-[calc(100%-2px)] after:h-px',
        'after:bg-white',
        'after:absolute after:bottom-0 after:left-[1px]',
      ],
    ])

    return html`<div class=${containerClasses} ${ref(this.buttonRef)}>
      <stencila-ui-icon name="feather" class="text-xs"></stencila-ui-icon
      ><span class="font-sans text-2xs leading-none block"
        >Authors and Provenance</span
      >
      ${this.renderCollapse()}
    </div>`
  }

  private renderBody() {
    const classes = apply([
      'p-4 text-sans',
      'border border-black/0 rounded-tl-none rounded-b rounded-tr',
      // 'group-hover:border-black',
      'transition-all ease-in duration-200',
    ])

    return html`<div class=${classes}>
      <slot></slot>
    </div>`
  }

  /**
   * Render the collapse card under the header.
   *
   * Note: this code is inspired by @linkcode `stencila-ui-base-card` and could
   * be abstracted further.
   */
  private renderCollapse() {
    const classes = apply(['flex items-center', `brightness-75`])

    return html`<div class=${classes}>
      <stencila-chevron-button
        default-pos=${this.context.showAuthorProvenance ? 'up' : 'down'}
        .disableEvents=${true}
        class="inline-flex text-xs"
      ></stencila-chevron-button>
    </div>`
  }
}
