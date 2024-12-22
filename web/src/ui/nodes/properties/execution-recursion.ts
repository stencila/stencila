import { customElement } from 'lit/decorators'

import { withTwind } from '../../../twind'

import { UINodeExecutionMode } from './execution-mode'

/**
 * A component for displaying/selecting then `executionRecursion` property of executable nodes
 */
@customElement('stencila-ui-node-execution-recursion')
@withTwind()
export class UINodeExecutionRecursion extends UINodeExecutionMode {
  protected override propertyName: 'executionMode' | 'executionRecursion' =
    'executionRecursion'
}
