// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Cord } from "./Cord.js";
import { Inline } from "./Inline.js";
import { Styled } from "./Styled.js";

/**
 * Styled inline content
 */
export class Span extends Styled {
  type = "Span";

  /**
   * The content within the span
   */
  content: Inline[];

  constructor(code: Cord, content: Inline[], options?: Partial<Span>) {
    super(code);
    if (options) Object.assign(this, options);
    this.code = code;
    this.content = content;
  }

  /**
  * Create a `Span` from an object
  */
  static from(other: Span): Span {
    return new Span(other.code!, other.content!, other);
  }
}
