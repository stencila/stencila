// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Inline } from "./Inline.js";
import { Mark } from "./Mark.js";

/**
 * Inline text that is underlined.
 */
export class Underline extends Mark {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Underline";

  constructor(content: Inline[], options?: Partial<Underline>) {
    super(content);
    this.type = "Underline";
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `Underline`
*/
export function underline(content: Inline[], options?: Partial<Underline>): Underline {
  return new Underline(content, options);
}
