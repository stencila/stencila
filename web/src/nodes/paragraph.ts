import { ContextConsumer } from '@lit/context'
import { NodeType } from '@stencila/types'
import { PropertyValueMap, html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'
import { EntityContext, entityContext } from '../ui/nodes/context'

import { Entity } from './entity'

import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/authorship'
import '../ui/nodes/properties/provenance'

/**
 * Web component representing a Stencila Schema `Paragraph` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/paragraph.md
 */
@customElement('stencila-paragraph')
@withTwind()
export class Paragraph extends Entity {
  /**
   * A list of parent nodes types that require different
   * rendering of this node.
   */
  private static parentNodeTypesSubscribedTo: NodeType[] = [
    'Admonition',
    'Claim',
    'CodeChunk',
    'Figure',
    'ListItem',
    'QuoteBlock',
    'Table',
    'TableCell',
  ]

  /**
   * The node type of the parent node
   */
  private parentNodeType: NodeType

  /**
   * A consumer controller for the `EntityContext`,
   * used to subscribe to the parent node's `EntityContext` if needed.
   */
  private parentContext: ContextConsumer<
    { __context__: EntityContext },
    this
  > | null = null

  override connectedCallback() {
    super.connectedCallback()

    this.parentNodeType = this.ancestors.split('.').reverse()[0] as NodeType

    /*
      If this Paragraph needs to be subscribed to the parent node
      creates a consumer for the `entityContext`,
      this will subscribe to the nearest entityContext above this node.
    */
    if (
      Paragraph.parentNodeTypesSubscribedTo.includes(this.parentNodeType) &&
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

  override render() {
    if (
      Paragraph.parentNodeTypesSubscribedTo.includes(this.parentNodeType) ||
      this.ancestors.includes('StyledBlock') ||
      this.isUserChatMessageNode()
    ) {
      return html`<slot name="content"></slot>`
    }

    return html`
      <stencila-ui-block-on-demand
        type="Paragraph"
        node-id=${this.id}
        depth=${this.depth}
        ancestors=${this.ancestors}
        ?isRootNode=${this.root}
      >
        <div slot="body">
          <stencila-ui-node-authors type="Paragraph">
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
