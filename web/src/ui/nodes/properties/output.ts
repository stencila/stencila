import { NodeType } from '@stencila/types'
import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../../twind'
import { nodeUi } from '../icons-and-colours'

import './generic/collapsible'

/**
 * A component for displaying the `output` property of a `CodeExpression`
 *
 * Renders a single output `Node` from the executed expression. Not to be
 * confused with `<stencila-ui-node-outputs>` which renders multiple `Node`s.
 *
 * This simply renders the `output` with a coloured background. It is implement
 * here, as a UI component, for consistency with implementation for other node
 * properties. However, its use is quite different, sitting outside of the node
 * card, and in content.
 */
@customElement('stencila-ui-node-output')
@withTwind()
export class UiNodeOutput extends LitElement {
  /**
   * The type of node that this property is for
   *
   * Always a `CodeExpression` but here for consistency with other
   * components for node properties where this is necessary.
   */
  @property()
  type: NodeType

  override render() {
    const { colour, borderColour } = nodeUi(this.type)

    const classes = apply([
      'inline-block',
      `bg-[${colour}]`,
      `border border-[${borderColour}]`,
    ])

    return html`<span class=${classes}><slot></slot></span>`
  }
}
