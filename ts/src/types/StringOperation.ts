// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";
import { UnsignedInteger } from "./UnsignedInteger.js";

/**
 * An operation that modifies a string.
 */
export class StringOperation extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "StringOperation";

  /**
   * The start position in the string of the operation.
   */
  startPosition: UnsignedInteger;

  /**
   * The end position in the string of the operation.
   */
  endPosition?: UnsignedInteger;

  /**
   * The string value to insert or use as the replacement.
   */
  value?: string;

  constructor(startPosition: UnsignedInteger, options?: Partial<StringOperation>) {
    super();
    this.type = "StringOperation";
    if (options) Object.assign(this, options);
    this.startPosition = startPosition;
  }
}

/**
* Create a new `StringOperation`
*/
export function stringOperation(startPosition: UnsignedInteger, options?: Partial<StringOperation>): StringOperation {
  return new StringOperation(startPosition, options);
}
