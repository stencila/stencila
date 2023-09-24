// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Cord } from './Cord';
import { Inline } from './Inline';
import { Styled } from './Styled';

// Styled inline content
export class Span extends Styled {
  type = "Span";

  // The content within the span
  content: Inline[];

  constructor(code: Cord, content: Inline[], options?: Span) {
    super(code)
    if (options) Object.assign(this, options)
    this.code = code;
    this.content = content;
  }

  static from(other: Span): Span {
    return new Span(other.code!, other.content!, other)
  }
}
