// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { Cord } from "./Cord.js";
import { StyledBlock } from "./StyledBlock.js";

/**
 * A separate page in a document
 */
export class Page extends StyledBlock {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Page";

  constructor(code: Cord, content: Block[], options?: Partial<Page>) {
    super(code, content);
    this.type = "Page";
    if (options) Object.assign(this, options);
    this.code = code;
    this.content = content;
  }
}

/**
* Create a new `Page`
*/
export function page(code: Cord, content: Block[], options?: Partial<Page>): Page {
  return new Page(code, content, options);
}
