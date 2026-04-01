// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { CodeStatic } from "./CodeStatic.js";
import { CompilationDigest } from "./CompilationDigest.js";
import { CompilationMessage } from "./CompilationMessage.js";
import { Cord } from "./Cord.js";

/**
 * A code block.
 */
export class CodeBlock extends CodeStatic {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "CodeBlock";

  /**
   * Whether the code block is a demo that should also be rendered.
   */
  isDemo?: boolean;

  /**
   * A digest of the `code` and `programmingLanguage`.
   */
  compilationDigest?: CompilationDigest;

  /**
   * Messages generated while compiling the demo content.
   */
  compilationMessages?: CompilationMessage[];

  /**
   * The content rendered from the code when `isDemo` is true.
   */
  content?: Block[];

  constructor(code: Cord, options?: Partial<CodeBlock>) {
    super(code);
    this.type = "CodeBlock";
    if (options) Object.assign(this, options);
    this.code = code;
  }
}

/**
* Create a new `CodeBlock`
*/
export function codeBlock(code: Cord, options?: Partial<CodeBlock>): CodeBlock {
  return new CodeBlock(code, options);
}
