// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { Executable } from "./Executable.js";
import { InstructionType } from "./InstructionType.js";

/**
 * A preview of how a prompt will be rendered at a location in the document
 */
export class PromptBlock extends Executable {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "PromptBlock";

  /**
   * The type of instruction the  being used for
   */
  instructionType?: InstructionType;

  /**
   * The type of nodes the prompt is being used for
   */
  nodeTypes?: string[];

  /**
   * A text hint used to infer the `target` prompt
   */
  hint?: string;

  /**
   * An identifier for the prompt to be rendered
   */
  target?: string;

  /**
   * The executed content of the prompt
   */
  content?: Block[];

  constructor(options?: Partial<PromptBlock>) {
    super();
    this.type = "PromptBlock";
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `PromptBlock`
*/
export function promptBlock(options?: Partial<PromptBlock>): PromptBlock {
  return new PromptBlock(options);
}
