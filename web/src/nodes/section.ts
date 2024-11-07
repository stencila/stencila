import { ContextConsumer } from '@lit/context'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'
import { entityContext, EntityContext } from '../ui/nodes/context'

import '../ui/nodes/for-block-iteration'
import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/provenance'

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
   * A consumer controller for the `EntityContext`,
   * used to subscribe to the parent node's `EntityContext` if needed.
   */
  private parentContext: ContextConsumer<
    { __context__: EntityContext },
    this
  > | null = null

  override connectedCallback(): void {
    super.connectedCallback()

    // If the section is a `ForBlock` iteration,
    // consume the context from the parent
    if (this.sectionType === 'Iteration') {
      this.parentContext = new ContextConsumer(this, {
        context: entityContext,
        subscribe: true,
      })
    }
  }

  override render() {
    return this.sectionType === 'Iteration'
      ? this.renderIteration()
      : this.renderSection()
  }

  /**
   * Render a normal section
   */
  private renderSection() {
    if (this.ancestors.includes('StyledBlock')) {
      return html`<slot name="content"></slot>`
    }

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
    const siblings = [...this.parentElement.children]
    const index = siblings.findIndex((elem) => elem === this)

    return html`
      <stencila-ui-for-block-iteration
        ?show-header=${this.parentContext && this.parentContext.value?.cardOpen}
        iteration-index=${index}
        ?last-iteration=${index === siblings.length - 1}
      >
        <slot name="content"></slot>
      </stencila-ui-for-block-iteration>
    `
  }
}
