// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from './Block';
import { Cord } from './Cord';
import { ExecutionDigest } from './ExecutionDigest';

// Styled block content
export class Division {
  type = "Division";

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

  // The content within the division
  content: Block[];

  constructor(code: Cord, content: Block[], options?: Division) {
    if (options) Object.assign(this, options)
    this.code = code;
    this.content = content;
  }
}
