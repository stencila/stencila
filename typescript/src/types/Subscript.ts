// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from "./Inline.js";
import { Mark } from "./Mark.js";

/**
 * Subscripted content.
 */
export class Subscript extends Mark {
  type = "Subscript";

  constructor(content: Inline[], options?: Partial<Subscript>) {
    super(content);
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `Subscript`
*/
export function subscript(content: Inline[], options?: Partial<Subscript>): Subscript {
  return new Subscript(content, options);
}
