import { customElement } from 'lit/decorators'

import { withTwind } from '../../../twind'
import { UIBaseClass } from '../mixins/ui-base-class'

/**
 * A component for displaying/selecting then `executionRecursion` property of executable nodes
 */
@customElement('stencila-ui-node-execution-recursion')
@withTwind()
export class UINodeExecutionRecursion extends UIBaseClass {
  // Temporarily disabled pending refactoring to execution "bounds"
}
