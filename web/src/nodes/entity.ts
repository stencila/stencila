import { provide } from '@lit/context'
import { NodeType } from '@stencila/types'
import { apply, css } from '@twind/core'
import { html, LitElement } from 'lit'
import { property, state } from 'lit/decorators'

import { DocumentAccess, DocumentView, NodeId } from '../types'
import { EntityContext, entityContext } from '../ui/nodes/entity-context'
import { closestGlobally } from '../utilities/closestGlobally'
import { getModeParam } from '../utilities/getModeParam'

import '../ui/nodes/node-insert'

import '../shoelace'

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
   *
   * Made `private` to encourage this use of the `isWithin` method.
   */
  @property()
  private ancestors: string

  /**
   * The Stencila Schema node type of the parent node
   */
  protected parentNodeType: NodeType

  /**
   * An element property to be used if the Entity requires a tooltip
   */
  protected tooltipElement: Element | null = null

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

    this.parentNodeType = this.ancestors
      ? (this.ancestors.split('.').pop() as NodeType)
      : null

    this.context.nodeId = this.id

    const mode = getModeParam(window)
    if ((mode && mode === 'test-expand-all') || this.depth === 0) {
      // start with card open in default
      this.context.cardOpen = true
    }

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
      (this.ancestors && this.ancestors.includes(`${nodeType}.`))
    )
  }

  /**
   * Whether the node is within a user chat message
   */
  protected isWithinUserChatMessage() {
    return (
      this.isWithin('ChatMessage') &&
      this.closestGlobally('stencila-chat-message[message-role="User"]') !==
        null
    )
  }

  /**
   * Whether the node is within a model chat message
   */
  protected isWithinModelChatMessage() {
    return (
      this.isWithin('ChatMessage') &&
      this.closestGlobally('stencila-chat-message[message-role="Model"]') !==
        null
    )
  }

  /**
   * Whether the entity is the current [root], or has a parent [root] node
   *
   * Used to alter the rendering behavior of node cards for "freestanding"
   * nodes (e.g. those embedded in a Ghost page).
   */
  protected hasDocumentRootNode() {
    return this.hasAttribute('root') || this.closestGlobally('[root]') !== null
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

  override render() {
    return html`<slot></slot>`
  }

  protected renderCard() {
    return html``
  }

  /**
   * Renders a node card with a <stencila-ui-node-insert> component
   * so that the node can be inserted into a document
   */
  protected renderCardWithInsert() {
    const classes = apply([
      'absolute -left-[27px] top-0',
      'opacity-0 group-hover:opacity-100',
      'transition-opacity duration-300',
    ])

    const styles = css`
      box-shadow: 0px 0px 4px 0px rgba(0, 0, 0, 0.25);
    `

    const nodeTuple = [this.nodeName, this.id]

    return html`
      <div class="group relative">
        <div class="${classes} ${styles}">
          <stencila-ui-node-insert .selectedNodes=${[nodeTuple]}>
          </stencila-ui-node-insert>
        </div>
        ${this.renderCard()}
      </div>
    `
  }
}
