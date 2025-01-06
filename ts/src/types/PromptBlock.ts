// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { Executable } from "./Executable.js";
import { InstructionType } from "./InstructionType.js";
import { RelativePosition } from "./RelativePosition.js";

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
   * The relative position of the node being edited, described etc.
   */
  relativePosition?: RelativePosition;

  /**
   * A user text query used to infer the `target` prompt
   */
  query?: string;

  /**
   * An identifier for the prompt to be rendered
   */
  target?: string;

  /**
   * The home directory of the prompt
   */
  directory?: string;

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
