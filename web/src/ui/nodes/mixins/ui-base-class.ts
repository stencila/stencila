import { NodeType } from '@stencila/types'
import { LitElement } from 'lit'
import { property } from 'lit/decorators'

import { NodeId } from '../../../types'
import { nodeUi } from '../icons-and-colours'

/**
 * A Base class for UI elements. Provides access to ui theming.
 */
export class UIBaseClass extends LitElement {
  /**
   * The type of node that this card is for
   *
   * Used to determine the title, icon and colors of the card.
   */
  @property()
  type: NodeType

  /**
   * The `id` of the cards parent node element
   */
  @property({ attribute: 'node-id' })
  nodeId: NodeId

  /**
   * Internal copy of the ui attributes.
   */
  protected ui: ReturnType<typeof nodeUi> | undefined = undefined

  /**
   * Provide ui options based on the node type.
   */
  override connectedCallback() {
    super.connectedCallback()

    this.ui = nodeUi(this.type)
  }
}
