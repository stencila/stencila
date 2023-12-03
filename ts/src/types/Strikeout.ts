// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from "./Inline.js";
import { Mark } from "./Mark.js";

/**
 * Content that is marked as struck out.
 */
export class Strikeout extends Mark {
  type = "Strikeout";

  constructor(content: Inline[], options?: Partial<Strikeout>) {
    super(content);
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `Strikeout`
*/
export function strikeout(content: Inline[], options?: Partial<Strikeout>): Strikeout {
  return new Strikeout(content, options);
}
