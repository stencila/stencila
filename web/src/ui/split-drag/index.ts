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
  private readonly DOUBLE_CLICK = 150 as const

  /**
   * When the splitInstance gets recreated, the touch/drag events on the gutter
   * handler don't appear to be removed. As such, each drag start/end will be
   * called multiple times. We use this MIN_DRAG to look for event handlers that
   * are clearly not part of the "real" split.js instance.
   */
  private readonly MIN_DRAG = 50 as const

  /**
   * Keep track of the split instance so we can modify when changing between
   * vertical & horizontal (i.e. the window size is too small to accommodate
   * the split view)
   */
  private splitInstance: Split.Instance | null = null

  /**
   * Keep track of direction of split as we resize the browser.
   */
  private direction: 'horizontal' | 'vertical' = 'horizontal'

  private readonly directionSizes = [50, 50]

  override render() {
    const panel = apply([
      'w-full min-w-full',
      'h-[calc(50vh-32px)]',
      'lg:(max-h-screen min-h-screen min-w-0)',
    ])

    return html`<div class="flex h-screen flex-col lg:flex-row">
      <div class="${panel} overflow-scroll" ${ref(this.leftRef)}>
        <slot name="left"></slot>
      </div>
      ${this.renderDrag()}
      <div class="${panel} overflow-y-scroll" ${ref(this.rightRef)}>
        <slot name="right"></slot>
      </div>
    </div>`
  }

  private renderDrag() {
    const innerDrag = apply([
      'h-1 w-8',
      'rounded-full',
      'relative',
      'transition-colors ease-in',
      'bg-gray-shady',
      'group-hover:bg-gray-mine-shaft',
      'lg:(h-8 w-1)',
    ])

    return html`<div
      class="group flex items-center justify-center h-10 w-full mx-auto lg:(h-screen my-auto w-10)"
      ${ref(this.dragRef)}
    >
      <div class="${innerDrag}"></div>
    </div>`
  }

  private handleWindowResize(self: DragSplit) {
    return () => {
      const lgBreakpoint = self.tw.theme('screens.lg') as string
      const breakpoint = parseInt(lgBreakpoint)
      const newDirection =
        window.innerWidth >= breakpoint ? 'horizontal' : 'vertical'
      const oldDirection = self.direction
      self.direction = newDirection

      if (newDirection !== oldDirection) {
        self.createSplitInstance()
      }
    }
  }

  private createSplitInstance() {
    // eslint-disable-next-line @typescript-eslint/no-this-alias
    const self = this
    const panels = [self.leftRef.value, self.rightRef.value]

    /**
     * If the component hasn't rendered yet, this is moot.
     */
    if (panels.some((p) => p === undefined)) {
      return
    }

    /**
     * The Split package does not define a method to easily switch directions.
     * The approach appears to be to instead destroy the existing instance &
     * create a new one.
     */
    if (self.splitInstance !== null) {
      self.splitInstance.destroy()
      self.dragEnd = 0
    }

    self.splitInstance = Split(panels, {
      sizes: self.directionSizes,
      direction: self.direction,
      gutter: () => {
        return self.dragRef.value
      },
      gutterStyle: () => {
        const size = self.tw.theme('width.10') as string

        switch (self.direction) {
          case 'horizontal':
            return {
              width: size,
              height: '100%',
            }

          case 'vertical':
            return {
              height: size,
              width: '100%',
            }
        }
      },
      onDragStart() {
        self.dragEnd = new Date().getTime()
      },
      onDragEnd() {
        const time = new Date().getTime()
        const diff = time - (self.dragEnd ?? 0)

        if (diff <= self.DOUBLE_CLICK && diff > self.MIN_DRAG) {
          self.splitInstance.setSizes(self.directionSizes)
        }

        self.dragEnd = time
      },
    })
  }

  firstUpdated() {
    this.createSplitInstance()
  }

  connectedCallback() {
    super.connectedCallback()
    window.addEventListener('resize', this.handleWindowResize(this))
    window.dispatchEvent(new Event('resize'))
  }

  disconnectedCallback() {
    window.removeEventListener('resize', this.handleWindowResize(this))
    super.disconnectedCallback()
  }
}
