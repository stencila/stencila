// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Author } from "./Author.js";
import { Cord } from "./Cord.js";
import { Executable } from "./Executable.js";
import { ProvenanceCount } from "./ProvenanceCount.js";

/**
 * Abstract base type for executable code nodes (e.g. `CodeChunk`).
 */
export class CodeExecutable extends Executable {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "CodeExecutable";

  /**
   * The code.
   */
  code: Cord;

  /**
   * The programming language of the code.
   */
  programmingLanguage?: string;

  /**
   * The authors of the executable code.
   */
  authors?: Author[];

  /**
   * A summary of the provenance of the code.
   */
  provenance?: ProvenanceCount[];

  constructor(code: Cord, options?: Partial<CodeExecutable>) {
    super();
    this.type = "CodeExecutable";
    if (options) Object.assign(this, options);
    this.code = code;
  }
}

/**
* Create a new `CodeExecutable`
*/
export function codeExecutable(code: Cord, options?: Partial<CodeExecutable>): CodeExecutable {
  return new CodeExecutable(code, options);
}
