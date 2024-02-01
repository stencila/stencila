// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { CodeExecutable } from "./CodeExecutable.js";
import { Cord } from "./Cord.js";
import { LabelType } from "./LabelType.js";
import { Node } from "./Node.js";

/**
 * A executable chunk of code.
 */
export class CodeChunk extends CodeExecutable {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "CodeChunk";

  /**
   * The type of the label for the chunk.
   */
  labelType?: LabelType;

  /**
   * A short label for the chunk.
   */
  label?: string;

  /**
   * A caption for the chunk.
   */
  caption?: Block[];

  /**
   * Outputs from executing the chunk.
   */
  outputs?: Node[];

  /**
   * Whether the code should be treated as side-effect free when executed.
   */
  executionPure?: boolean;

  constructor(code: Cord, options?: Partial<CodeChunk>) {
    super(code);
    this.type = "CodeChunk";
    if (options) Object.assign(this, options);
    this.code = code;
  }
}

/**
* Create a new `CodeChunk`
*/
export function codeChunk(code: Cord, options?: Partial<CodeChunk>): CodeChunk {
  return new CodeChunk(code, options);
}
