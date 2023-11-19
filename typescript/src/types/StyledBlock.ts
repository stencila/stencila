// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from "./Block.js";
import { Cord } from "./Cord.js";
import { Styled } from "./Styled.js";

/**
 * Styled block content.
 */
export class StyledBlock extends Styled {
  type = "StyledBlock";

  /**
   * The content within the styled block
   */
  content: Block[];

  constructor(code: Cord, content: Block[], options?: Partial<StyledBlock>) {
    super(code);
    if (options) Object.assign(this, options);
    this.code = code;
    this.content = content;
  }
}

/**
* Create a new `StyledBlock`
*/
export function styledBlock(code: Cord, content: Block[], options?: Partial<StyledBlock>): StyledBlock {
  return new StyledBlock(code, content, options);
}
