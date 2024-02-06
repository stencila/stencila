// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Inline } from "./Inline.js";
import { Mark } from "./Mark.js";

/**
 * Superscripted content.
 */
export class Superscript extends Mark {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Superscript";

  constructor(content: Inline[], options?: Partial<Superscript>) {
    super(content);
    this.type = "Superscript";
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `Superscript`
*/
export function superscript(content: Inline[], options?: Partial<Superscript>): Superscript {
  return new Superscript(content, options);
}
