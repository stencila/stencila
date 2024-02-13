// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";
import { Integer } from "./Integer.js";

/**
 * A hint to the structure of an `String`.
 */
export class StringHint extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "StringHint";

  /**
   * The number of characters in the string.
   */
  chars: Integer;

  constructor(chars: Integer, options?: Partial<StringHint>) {
    super();
    this.type = "StringHint";
    if (options) Object.assign(this, options);
    this.chars = chars;
  }
}

/**
* Create a new `StringHint`
*/
export function stringHint(chars: Integer, options?: Partial<StringHint>): StringHint {
  return new StringHint(chars, options);
}
