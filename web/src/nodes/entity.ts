import { provide } from '@lit/context'
import { NodeType } from '@stencila/types'
import { html, LitElement } from 'lit'
import { property, state } from 'lit/decorators'

import { nodePatchEvent, NodePatch } from '../clients/nodes'
import { DocumentAccess, DocumentView, NodeId } from '../types'
import { EntityContext, entityContext } from '../ui/nodes/context'
import { closestGlobally } from '../utilities/closestGlobally'

/**
 * Abstract base class for web components representing Stencila Schema `Entity` node types
 *
 * Given that the `Entity` node type is the ancestor of all other node types in the Stencila
 * Schema (other than "primitive" types), this is the ancestor class of all web components
 * for node types. As such it implements and few core methods commonly used.
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/entity.md
 *
 * Note that this does not need to have an `id` property (as in the `Entity` schema)
 * because `id` is already a property of `HTMLElement` from which this is derived.
 */
export abstract class Entity extends LitElement {
  /**
   * The Stencila Schema `id` property
   *
   * Not to be confused with the Rust `node_id` which is on every node (apart
   * from primitives) and in DOM HTML is represented as the `id` property (for
   * DOM reconciliation and morphing).
   */
  @property({ attribute: '_id' })
  $id?: string

  /**
   * Whether or not this is the root node in the node tree
   */
  @property({ type: Boolean })
  root: boolean = false

  /**
   * The depth of the node in the node tree
   *
   * The root node (e.g. `Article`) will have a depth of zero.
   */
  @property({ type: Number })
  depth: number

  /**
   * The dot separated list of the types of the ancestors of the node
   *
   * The root node will have an empty string for this property.
   * Other nodes will have a list of ancestor node types e.g. `Article.Paragraph.Emphasis`
   * for `Text` within an emphasis node in a paragraph of an article.
   */
  @property()
  ancestors: string

  /**
   * The Stencila Schema node type of the parent node
   */
  protected parentNodeType: NodeType

  /**
   * The Id of a child node that is/or contains,
   * a currently selected node in the sourceView
   */
  @property({ type: String, attribute: 'active-child' })
  activeChild: NodeId

  @provide({ context: entityContext })
  @state()
  protected context: EntityContext = {
    nodeId: this.id,
    cardOpen: false,
  }

  override connectedCallback(): void {
    super.connectedCallback()

    this.parentNodeType = this.ancestors.split('.').pop() as NodeType

    this.context.nodeId = this.id

    this.shadowRoot.addEventListener(
      `toggle-${this.id}`,
      (e: Event & { detail: EntityContext }) => {
        // only update the context for the relevant node
        if (e.detail.nodeId === this.id) {
          this.context = {
            ...this.context,
            cardOpen: e.detail.cardOpen,
          }
        }
      }
    )
  }

  /**
   * Whether the parent node is of the specified type
   */
  protected parentNodeIs(nodeType: NodeType): boolean {
    return this.parentNodeType === nodeType
  }

  /**
   * Whether the node is within (i.e has an ancestor) of the specified type
   */
  protected isWithin(nodeType: NodeType): boolean {
    return (
      this.parentNodeType === nodeType ||
      this.ancestors.includes(`${nodeType}.`)
    )
  }

  /**
   * Whether the node is within a chat message
   */
  protected isWithinUserChatMessage() {
    return (
      this.isWithin('ChatMessage') &&
      this.closestGlobally('stencila-chat-message[message-role="User"]') !==
        null
    )
  }

  /**
   * Select the closest element matching a selector
   */
  protected closestGlobally(selector: string): HTMLElement | null {
    return closestGlobally(this, selector)
  }

  /**
   * Get the name of the view that this web component is contained within
   *
   * This may be used by derived elements to alter their rendering and/or
   * behavior based on the view.
   */
  protected documentView(): DocumentView {
    return this.closestGlobally('[view]')?.getAttribute('view') as DocumentView
  }

  /**
   * Get the document access level of the view that this web component
   * is contained within
   *
   * This may be used by derived elements to alter their rendering and/or
   * behavior based on the view.
   */
  protected documentAccess(): DocumentAccess {
    return this.closestGlobally('[view]')?.getAttribute(
      'access'
    ) as DocumentAccess
  }

  /**
   * Patch the node that this web component represents
   *
   * Emits a `CustomEvent` containing a `NodePatch` which is forwarded by
   * the `NodesClient` to the document on the server.
   */
  protected patchNode(patch: NodePatch) {
    this.dispatchEvent(nodePatchEvent(patch))
  }

  override render() {
    return html`<slot></slot>`
  }
}
