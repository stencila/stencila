import { NodeType } from '@stencila/types'
import { LitElement } from 'lit'
import { property } from 'lit/decorators'

import { NodeId } from '../../../types'
import { NodeTypeUI, nodeUi } from '../icons-and-colours'

/**
 * A base class for UI elements representing Stencila document nodes
 */
export class UIBaseClass extends LitElement {
  /**
   * The type of node that this UI element is representing
   *
   * Used for getting the UI settings for the node type (see below)
   * and when emitting command events.
   */
  @property()
  type: NodeType

  /**
   * The id of node that this UI element is representing
   *
   * Mainly used when emitting command events.
   */
  @property({ attribute: 'node-id' })
  nodeId: NodeId

  /**
   * The depth of node that this UI element is representing
   *
   * Mainly used to alter UI (e.g. node markers) based on
   * the depth of the node
   */
  @property({ type: Number })
  depth: number

  /**
   * Internal copy of the UI attributes for the node type.
   */
  protected ui: NodeTypeUI | undefined = undefined

  override connectedCallback() {
    super.connectedCallback()

    this.ui = nodeUi(this.type)
  }
}
