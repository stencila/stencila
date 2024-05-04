// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Author } from "./Author.js";
import { CompilationDigest } from "./CompilationDigest.js";
import { CompilationMessage } from "./CompilationMessage.js";
import { Cord } from "./Cord.js";
import { Entity } from "./Entity.js";
import { ProvenanceCount } from "./ProvenanceCount.js";

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
   * The authors of the code and content in the styled node.
   */
  authors?: Author[];

  /**
   * A summary of the provenance of the code and content in the styed node.
   */
  provenance?: ProvenanceCount[];

  /**
   * A digest of the `code` and `styleLanguage`.
   */
  compilationDigest?: CompilationDigest;

  /**
   * Messages generated while parsing and transpiling the style.
   */
  compilationMessages?: CompilationMessage[];

  /**
   * A Cascading Style Sheet (CSS) transpiled from the `code` property.
   */
  css?: string;

  /**
   * A space separated list of class names associated with the node.
   */
  classList?: string;

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
