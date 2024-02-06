// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";
import { StringOperation } from "./StringOperation.js";

/**
 * An set of operations to modify a string.
 */
export class StringPatch extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "StringPatch";

  /**
   * The operations to be applied to the string.
   */
  operations: StringOperation[];

  constructor(operations: StringOperation[], options?: Partial<StringPatch>) {
    super();
    this.type = "StringPatch";
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
