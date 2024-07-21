import { ContextConsumer } from '@lit/context'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'
import '../ui/nodes/node-card/on-demand/block'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/provenance/provenance'
import { entityContext, EntityContext } from '../ui/nodes/context'
import { nodeUi } from '../ui/nodes/icons-and-colours'
import { getOrdinalString } from '../utility/ordinal'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `Section` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/section.md
 */
@customElement('stencila-section')
@withTwind()
export class Section extends Entity {
  @property({ attribute: 'section-type' })
  sectionType?: string

  /**
   * A consumer controller for the `EnityContext`,
   * used to subscribe to the parent node's `EnityContext` if needed.
   */
  private parentContext: ContextConsumer<
    { __context__: EntityContext },
    this
  > | null = null

  override render() {
    return this.sectionType === 'Iteration'
      ? this.renderIteration()
      : this.renderSection()
  }

  override connectedCallback(): void {
    super.connectedCallback()

    if (this.sectionType === 'Iteration') {
      this.parentContext = new ContextConsumer(this, {
        context: entityContext,
        subscribe: true,
      })
    }
  }

  /**
   * Render a normal section
   */
  private renderSection() {
    return html`
      <stencila-ui-block-on-demand
        type="Section"
        depth=${this.depth}
        ancestors=${this.ancestors}
      >
        <div slot="body">
          <stencila-ui-node-authors type="Section">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
        </div>
        <div slot="content">
          <slot name="content"></slot>
        </div>
      </stencila-ui-block-on-demand>
    `
  }

  /**
   * Render a section when it is an iteration of a `ForBlock`
   */
  private renderIteration() {
    const { colour, borderColour, textColour } = nodeUi('ForBlock')

    const siblings = [...this.parentElement.children]
    const index = siblings.findIndex((elem) => elem === this)

    const showHeader = this.parentContext && this.parentContext.value?.cardOpen

    return html`<div
        class="${showHeader
          ? 'flex'
          : 'hidden'} px-4 py-2 flex items-center text-[${textColour}] bg-[${colour}] border-[${borderColour}] font-sans text-sm"
      >
        ${getOrdinalString(index + 1)} Iteration
      </div>
      <div class="${showHeader ? 'p-3' : ''}">
        <slot name="content"></slot>
      </div>`
  }
}
