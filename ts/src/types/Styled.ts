// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Author } from "./Author.js";
import { CompilationDigest } from "./CompilationDigest.js";
import { CompilationError } from "./CompilationError.js";
import { Cord } from "./Cord.js";
import { Entity } from "./Entity.js";

/**
 * An abstract base class for a document node that has styling applied to it and/or its content.
 */
export class Styled extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Styled";

  /**
   * The code of the equation in the `styleLanguage`.
   */
  code: Cord;

  /**
   * The language used for the style specification e.g. css, tw
   */
  styleLanguage?: string;

  /**
   * The authors of the styling code.
   */
  authors?: Author[];

  /**
   * A digest of the `code` and `styleLanguage`.
   */
  compilationDigest?: CompilationDigest;

  /**
   * Errors generated when parsing and transpiling the style.
   */
  compilationErrors?: CompilationError[];

  /**
   * A Cascading Style Sheet (CSS) transpiled from the `code` property.
   */
  css?: string;

  /**
   * A list of class names associated with the node.
   */
  classes?: string[];

  constructor(code: Cord, options?: Partial<Styled>) {
    super();
    this.type = "Styled";
    if (options) Object.assign(this, options);
    this.code = code;
  }
}

/**
* Create a new `Styled`
*/
export function styled(code: Cord, options?: Partial<Styled>): Styled {
  return new Styled(code, options);
}
