// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";
import { Inline } from "./Inline.js";

/**
 * A block containing inlines with no other semantics.
 */
export class InlinesBlock extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "InlinesBlock";

  /**
   * The contents of the block.
   */
  content: Inline[];

  constructor(content: Inline[], options?: Partial<InlinesBlock>) {
    super();
    this.type = "InlinesBlock";
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `InlinesBlock`
*/
export function inlinesBlock(content: Inline[], options?: Partial<InlinesBlock>): InlinesBlock {
  return new InlinesBlock(content, options);
}
