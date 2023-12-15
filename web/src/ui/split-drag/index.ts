import { apply } from '@twind/core'
import { html } from 'lit'
import { customElement } from 'lit/decorators.js'
import { Ref, createRef, ref } from 'lit/directives/ref'
import Split from 'split.js'

import { withTwind } from '../../twind'
import { TWLitElement } from '../twind'

/**
 * Drag Split
 *
 * Provides a vertical split screen, allowing users to display content from 2
 * slots & drag the vertical pane between them.
 */
@customElement('stencila-ui-drag-split')
@withTwind()
export class DragSplit extends TWLitElement {
  /**
   * The ref used by the draggable slider.
   */
  dragRef: Ref<HTMLElement> = createRef()

  /**
   * The ref used by the left hand panel
   */
  leftRef: Ref<HTMLDivElement> = createRef()

  /**
   * The ref used by the right hand panel
   */
  rightRef: Ref<HTMLDivElement> = createRef()

  /**
   * Instance var to keep track of previous dragend time (in milliseconds)
   */
  private dragEnd: number | undefined = undefined

  /**
   * Const to determine the length of time between drags ending, whereby we
   * consider the user to have performed a double click/touch.
   */
  private readonly DOUBLE_CLICK = 500 as const

  override render() {
    return html`<div class="flex flex-row">
      <div class="max-h-screen overflow-scroll" ${ref(this.leftRef)}>
        <slot name="left"></slot>
      </div>
      ${this.renderDrag()}
      <div class="max-h-screen overflow-scroll" ${ref(this.rightRef)}>
        <slot name="right"></slot>
      </div>
    </div>`
  }

  private renderDrag() {
    const innerDrag = apply([
      'h-8 w-1',
      'rounded-full',
      'relative',
      'transition-colors ease-in',
      'bg-gray-shady',
      'group-hover:bg-gray-mine-shaft',
    ])

    return html`<div
      class="group w-10 flex items-center justify-center h-screen"
      ${ref(this.dragRef)}
    >
      <div class="${innerDrag}"></div>
    </div>`
  }

  firstUpdated() {
    // eslint-disable-next-line @typescript-eslint/no-this-alias
    const self = this

    const instance = Split([this.leftRef.value, this.rightRef.value], {
      sizes: [50, 50],
      direction: 'horizontal',
      gutter: () => {
        return this.dragRef.value
      },
      gutterStyle: () => {
        const gutterWidth = this.tw.theme('width.10') as string

        return {
          width: gutterWidth,
        }
      },
      onDragEnd() {
        const time = new Date().getTime()
        const diff = time - (self.dragEnd ?? 0)

        if (diff <= self.DOUBLE_CLICK) {
          instance.setSizes([50, 50])
        }

        self.dragEnd = time
      },
    })
  }
}
