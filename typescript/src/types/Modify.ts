// Generated file; do not edit. See `../rust/schema-gen` crate.

import { ModifyOperation } from "./ModifyOperation.js";

/**
 * A suggestion to modify one or more nodes.
 */
export class Modify {
  /**
   * The operations to be applied to the nodes.
   */
  operations: ModifyOperation[];

  constructor(operations: ModifyOperation[], options?: Partial<Modify>) {
    if (options) Object.assign(this, options);
    this.operations = operations;
  }
}

/**
* Create a new `Modify`
*/
export function modify(operations: ModifyOperation[], options?: Partial<Modify>): Modify {
  return new Modify(operations, options);
}
