// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Action } from "./Action.js";

/**
 * An action that converts a resource from one representation or format to another.
 */
export class ConvertAction extends Action {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "ConvertAction";

  constructor(options?: Partial<ConvertAction>) {
    super();
    this.type = "ConvertAction";
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `ConvertAction`
*/
export function convertAction(options?: Partial<ConvertAction>): ConvertAction {
  return new ConvertAction(options);
}
