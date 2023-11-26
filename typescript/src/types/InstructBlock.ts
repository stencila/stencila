// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from "./Block.js";
import { Instruct } from "./Instruct.js";

/**
 * An instruction to edit some block content.
 */
export class InstructBlock extends Instruct {
  type = "InstructBlock";

  /**
   * The content to which the instruction applies.
   */
  content?: Block[];

  constructor(text: string, options?: Partial<InstructBlock>) {
    super(text);
    if (options) Object.assign(this, options);
    this.text = text;
  }
}

/**
* Create a new `InstructBlock`
*/
export function instructBlock(text: string, options?: Partial<InstructBlock>): InstructBlock {
  return new InstructBlock(text, options);
}
