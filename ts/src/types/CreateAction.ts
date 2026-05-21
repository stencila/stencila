// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Action } from "./Action.js";

/**
 * An action that creates a result.
 */
export class CreateAction extends Action {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "CreateAction";

  constructor(options?: Partial<CreateAction>) {
    super();
    this.type = "CreateAction";
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `CreateAction`
*/
export function createAction(options?: Partial<CreateAction>): CreateAction {
  return new CreateAction(options);
}
