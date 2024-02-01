// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Inline } from "./Inline.js";
import { Mark } from "./Mark.js";

/**
 * Subscripted content.
 */
export class Subscript extends Mark {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Subscript";

  constructor(content: Inline[], options?: Partial<Subscript>) {
    super(content);
    this.type = "Subscript";
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
