// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";
import { Inline } from "./Inline.js";

/**
 * Abstract base class for nodes that mark some other inline content in some way (e.g. as being emphasised, or quoted).
 */
export class Mark extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Mark";

  /**
   * The content that is marked.
   */
  content: Inline[];

  constructor(content: Inline[], options?: Partial<Mark>) {
    super();
    this.type = "Mark";
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `Mark`
*/
export function mark(content: Inline[], options?: Partial<Mark>): Mark {
  return new Mark(content, options);
}
