// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";
import { Inline } from "./Inline.js";

/**
 * A sentence, usually within a `Paragraph`.
 */
export class Sentence extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Sentence";

  /**
   * The content of the sentence.
   */
  content: Inline[];

  constructor(content: Inline[], options?: Partial<Sentence>) {
    super();
    this.type = "Sentence";
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `Sentence`
*/
export function sentence(content: Inline[], options?: Partial<Sentence>): Sentence {
  return new Sentence(content, options);
}
