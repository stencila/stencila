// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from "./Inline.js";
import { Mark } from "./Mark.js";

/**
 * Inline text that is underlined.
 */
export class Underline extends Mark {
  type = "Underline";

  constructor(content: Inline[], options?: Partial<Underline>) {
    super(content);
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
