// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";
import { UnsignedInteger } from "./UnsignedInteger.js";

/**
 * The location within some source code.
 */
export class CodeLocation extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "CodeLocation";

  /**
   * The source of the code, a file path, label or URL.
   */
  source?: string;

  /**
   * The 0-based index if the first line on which the error occurred.
   */
  startLine?: UnsignedInteger;

  /**
   * The 0-based index if the first column on which the error occurred.
   */
  startColumn?: UnsignedInteger;

  /**
   * The 0-based index if the last line on which the error occurred.
   */
  endLine?: UnsignedInteger;

  /**
   * The 0-based index if the last column on which the error occurred.
   */
  endColumn?: UnsignedInteger;

  constructor(options?: Partial<CodeLocation>) {
    super();
    this.type = "CodeLocation";
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `CodeLocation`
*/
export function codeLocation(options?: Partial<CodeLocation>): CodeLocation {
  return new CodeLocation(options);
}
