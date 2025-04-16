// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { CitationIntent } from "./CitationIntent.js";
import { CitationMode } from "./CitationMode.js";
import { CompilationMessage } from "./CompilationMessage.js";
import { Entity } from "./Entity.js";
import { Inline } from "./Inline.js";
import { IntegerOrString } from "./IntegerOrString.js";
import { Reference } from "./Reference.js";

/**
 * A reference to a `CreativeWork` that is cited in another `CreativeWork`.
 */
export class Citation extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Citation";

  /**
   * The target of the citation (URL or reference ID).
   */
  target: string;

  /**
   * Messages generated while resolving the target if the citation.
   */
  compilationMessages?: CompilationMessage[];

  /**
   * The `Reference` being cited, resolved from the `target`.
   */
  cites?: Reference;

  /**
   * Determines how the citation is shown within the surrounding text.
   */
  citationMode?: CitationMode;

  /**
   * The type/s of the citation, both factually and rhetorically.
   */
  citationIntent?: CitationIntent[];

  /**
   * Optional structured content/text of this citation.
   */
  content?: Inline[];

  /**
   * The page on which the work starts; for example "135" or "xiii".
   */
  pageStart?: IntegerOrString;

  /**
   * The page on which the work ends; for example "138" or "xvi".
   */
  pageEnd?: IntegerOrString;

  /**
   * Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55".
   */
  pagination?: string;

  /**
   * Text to show before the citation.
   */
  citationPrefix?: string;

  /**
   * Text to show after the citation.
   */
  citationSuffix?: string;

  constructor(target: string, options?: Partial<Citation>) {
    super();
    this.type = "Citation";
    if (options) Object.assign(this, options);
    this.target = target;
  }
}

/**
* Create a new `Citation`
*/
export function citation(target: string, options?: Partial<Citation>): Citation {
  return new Citation(target, options);
}
