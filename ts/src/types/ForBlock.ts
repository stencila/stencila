// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { CodeExecutable } from "./CodeExecutable.js";
import { Cord } from "./Cord.js";

/**
 * Repeat a block content for each item in an array.
 */
export class ForBlock extends CodeExecutable {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "ForBlock";

  /**
   * The name to give to the variable representing each item in the iterated array
   */
  variable: string;

  /**
   * The content to repeat for each item
   */
  content: Block[];

  /**
   * The content to render if there are no items
   */
  otherwise?: Block[];

  /**
   * The content repeated for each iteration
   */
  iterations?: Block[];

  constructor(code: Cord, variable: string, content: Block[], options?: Partial<ForBlock>) {
    super(code);
    this.type = "ForBlock";
    if (options) Object.assign(this, options);
    this.code = code;
    this.variable = variable;
    this.content = content;
  }
}

/**
* Create a new `ForBlock`
*/
export function forBlock(code: Cord, variable: string, content: Block[], options?: Partial<ForBlock>): ForBlock {
  return new ForBlock(code, variable, content, options);
}
