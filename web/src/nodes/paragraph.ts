import { ContextConsumer } from '@lit/context'
import { NodeType } from '@stencila/types'
import { PropertyValueMap, html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'
import { EntityContext, entityContext } from '../ui/nodes/context'

import { Entity } from './entity'

import '../ui/nodes/node-card/on-demand/block'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/authorship'
import '../ui/nodes/properties/provenance/provenance'

/**
 * Web component representing a Stencila Schema `Paragraph` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/paragraph.md
 */
@customElement('stencila-paragraph')
@withTwind()
export class Paragraph extends Entity {
  /**
   * a list of parent nodes that can require different
   * behaviour/rendering of this node.
   */
  static subscribedParentNodes: NodeType[] = [
    'ListItem',
    'TableCell',
    'QuoteBlock',
    'Table',
    'Figure',
    'CodeChunk',
  ]

  /**
   * In static view just render the `content`.
   */
  override renderStaticView() {
    return html`<slot name="content"></slot>`
  }

  /**
   * The ancester node directly above this one in the tree
   */
  private directAncestor: NodeType

  /**
   * A consumer controller for the `EnityContext`,
   * used to subscribe to the parent node's `EnityContext` if needed.
   */
  private parentContext: ContextConsumer<
    { __context__: EntityContext },
    this
  > | null = null

  override connectedCallback() {
    super.connectedCallback()

    this.directAncestor = this.ancestors.split('.').reverse()[0] as NodeType

    /* 
      if this Paragraph needs to be subscribed to parent node
      creates a consumer for the `entityContext`, 
      this will subscribe to the nearest entityContext above this node.
    */
    if (
      Paragraph.subscribedParentNodes.includes(this.directAncestor) &&
      !this.parentContext
    ) {
      this.parentContext = new ContextConsumer(this, {
        context: entityContext,
        subscribe: true,
      })
      this.context = {
        ...this.context,
        cardOpen: this.parentContext.value?.cardOpen,
      }
    }
  }

  protected override update(
    changedProperties: PropertyValueMap<this> | Map<PropertyKey, unknown>
  ): void {
    super.update(changedProperties)

    if (this.parentContext) {
      /* 
        if `parentContext` is initiated,
        mirror the paragraph entity's context `cardOpen` status to the parent.
      */
      if (this.parentContext.value?.cardOpen !== this.context.cardOpen) {
        this.context = {
          ...this.context,
          cardOpen: this.parentContext.value.cardOpen,
        }
      }
    }
  }

  /**
   * In dynamic view render `content`, and `authors` and summary stats in a node card
   * that is shown on hover.
   */
  override renderDynamicView() {
    if (Paragraph.subscribedParentNodes.includes(this.directAncestor)) {
      return html`<slot name="content"></slot>`
    }

    // TODO: Add summary stats to card

    return html`
      <stencila-ui-block-on-demand
        type="Paragraph"
        view="dynamic"
        node-id=${this.id}
      >
        <div slot="body">
          <stencila-ui-node-authors type="Paragraph">
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
   * In source view render `authors` and summary stats in a node card. Do not render
   * `content` since that is visible in the source code.
   */
  override renderSourceView() {
    // TODO: Add summary stats to card

    return html`
      <stencila-ui-block-on-demand type="Paragraph" view="source">
        <div slot="body">
          <stencila-ui-node-authors type="Paragraph">
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
        </div>
      </stencila-ui-block-on-demand>
    `
  }
}
