// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Cord } from "./Cord.js";
import { Inline } from "./Inline.js";
import { Styled } from "./Styled.js";

/**
 * Styled inline content.
 */
export class StyledInline extends Styled {
  type = "StyledInline";

  /**
   * The content within the span.
   */
  content: Inline[];

  constructor(code: Cord, content: Inline[], options?: Partial<StyledInline>) {
    super(code);
    if (options) Object.assign(this, options);
    this.code = code;
    this.content = content;
  }
}

/**
* Create a new `StyledInline`
*/
export function styledInline(code: Cord, content: Inline[], options?: Partial<StyledInline>): StyledInline {
  return new StyledInline(code, content, options);
}
