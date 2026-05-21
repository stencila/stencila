// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Action } from "./Action.js";

/**
 * An action that executes code, a prompt, a workflow, or another executable node.
 */
export class ExecuteAction extends Action {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "ExecuteAction";

  constructor(options?: Partial<ExecuteAction>) {
    super();
    this.type = "ExecuteAction";
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `ExecuteAction`
*/
export function executeAction(options?: Partial<ExecuteAction>): ExecuteAction {
  return new ExecuteAction(options);
}
