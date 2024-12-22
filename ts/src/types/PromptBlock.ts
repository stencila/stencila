// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { Executable } from "./Executable.js";

/**
 * A preview of how a prompt will be rendered at a location in the document
 */
export class PromptBlock extends Executable {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "PromptBlock";

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
