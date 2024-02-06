// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Inline } from "./Inline.js";
import { Mark } from "./Mark.js";

/**
 * Emphasized content.
 */
export class Emphasis extends Mark {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Emphasis";

  constructor(content: Inline[], options?: Partial<Emphasis>) {
    super(content);
    this.type = "Emphasis";
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `Emphasis`
*/
export function emphasis(content: Inline[], options?: Partial<Emphasis>): Emphasis {
  return new Emphasis(content, options);
}
