import { IntersectionController } from '@lit-labs/observers/intersection-controller'
import { html, LitElement } from 'lit'
import { customElement, property } from 'lit/decorators'

import { NodeId } from '../types'
import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/authorship'
import '../ui/nodes/properties/provenance'

import { Entity } from './entity'

/**
 * The name of the `CustomEvent` emitted when the visibility of a heading changes
 */
export const HEADING_VISIBILITY_EVENT = 'stencila-heading-visibility'

/**
 * The details of a heading visibility custom event
 */
export type HeadingVisibilityEvent = {
  id: NodeId
  position: -1 | 0 | 1
  isEnd: boolean
}

/**
 * Web component representing a Stencila Schema `Heading` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/heading.md
 */
@customElement('stencila-heading')
export class Heading extends Entity {
  @property({ type: Number })
  level: number

  // @ts-expect-error observer is never read
  private observer = new IntersectionController(this, {
    config: {
      threshold: 0.5,
    },
    callback: ([entry]) =>
      this.dispatchEvent(
        new CustomEvent<HeadingVisibilityEvent>(HEADING_VISIBILITY_EVENT, {
          bubbles: true,
          detail: {
            id: this.id,
            position: entry.isIntersecting
              ? 0
              : entry.boundingClientRect.top <= 0
                ? 1
                : -1,
            isEnd: false,
          },
        })
      ),
  })

  override render() {
    if (this.ancestors.includes('StyledBlock')) {
      return html`<slot name="content"></slot>`
    }

    return html`
      <stencila-ui-block-on-demand type="Heading" node-id=${this.id}>
        <div slot="body">
          <stencila-ui-node-authors type="Heading">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
        </div>
        <slot name="content" slot="content"></slot>
      </stencila-ui-block-on-demand>
    `
  }
}

/**
 * Web component marking the end of a section started by a heading
 */
@customElement('stencila-heading-end')
export class HeadingEnd extends LitElement {
  /**
   * The id of the heading that this is the end for
   */
  @property()
  heading: string

  // @ts-expect-error observer is never read
  private observer = new IntersectionController(this, {
    config: {
      // Use a full threshold and negative root margin to make this
      // element invisible when at top of viewport
      threshold: 1.0,
      rootMargin: '-10px 0px 0px 0px',
    },
    callback: ([entry]) =>
      this.dispatchEvent(
        new CustomEvent<HeadingVisibilityEvent>(HEADING_VISIBILITY_EVENT, {
          bubbles: true,
          detail: {
            id: this.heading,
            position: entry.isIntersecting
              ? 0
              : entry.boundingClientRect.top <= 0
                ? 1
                : -1,
            isEnd: true,
          },
        })
      ),
  })
}
