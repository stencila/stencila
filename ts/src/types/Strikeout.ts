// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Inline } from "./Inline.js";
import { Mark } from "./Mark.js";

/**
 * Content that is marked as struck out.
 */
export class Strikeout extends Mark {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Strikeout";

  constructor(content: Inline[], options?: Partial<Strikeout>) {
    super(content);
    this.type = "Strikeout";
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
