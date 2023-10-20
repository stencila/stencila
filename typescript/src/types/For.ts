// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Array } from "./Array.js";
import { Block } from "./Block.js";
import { CodeExecutable } from "./CodeExecutable.js";
import { Cord } from "./Cord.js";

/**
 * Repeat a block content for each item in an array.
 */
export class For extends CodeExecutable {
  type = "For";

  /**
   * The name to give to the variable representing each item in the iterated array
   */
  symbol: string;

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
  iterations?: Array[];

  constructor(code: Cord, symbol: string, content: Block[], options?: Partial<For>) {
    super(code);
    if (options) Object.assign(this, options);
    this.code = code;
    this.symbol = symbol;
    this.content = content;
  }
}

/**
* Create a new `For`
*/
export function for_(code: Cord, symbol: string, content: Block[], options?: Partial<For>): For {
  return new For(code, symbol, content, options);
}
