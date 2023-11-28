// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from "./Entity.js";
import { StringOperation } from "./StringOperation.js";

/**
 * An set of operations to modify a string.
 */
export class StringPatch extends Entity {
  type = "StringPatch";

  /**
   * The operations to be applied to the string.
   */
  operations: StringOperation[];

  constructor(operations: StringOperation[], options?: Partial<StringPatch>) {
    super();
    if (options) Object.assign(this, options);
    this.operations = operations;
  }
}

/**
* Create a new `StringPatch`
*/
export function stringPatch(operations: StringOperation[], options?: Partial<StringPatch>): StringPatch {
  return new StringPatch(operations, options);
}
