// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Cord } from './Cord';
import { ExecutionDigest } from './ExecutionDigest';
import { Inline } from './Inline';

// Styled inline content
export class Span {
  type = "Span";

  // The identifier for this item
  id?: string;

  // The code of the equation in the `styleLanguage`.
  code: Cord;

  // The language used for the style specification e.g. css, tailwind, classes.
  styleLanguage?: string;

  // A digest of the `code` and `styleLanguage`.
  compileDigest?: ExecutionDigest;

  // Errors that occurred when transpiling the `code`.
  errors?: string[];

  // A Cascading Style Sheet (CSS) transpiled from the `code` property.
  css?: string;

  // A list of class names associated with the node
  classes?: string[];

  // The content within the span
  content: Inline[];

  constructor(code: Cord, content: Inline[], options?: Span) {
    if (options) Object.assign(this, options)
    this.code = code;
    this.content = content;
  }
}
