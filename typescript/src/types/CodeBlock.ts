// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Cord } from './Cord';

// A code block.
export class CodeBlock {
  type = "CodeBlock";

  // The identifier for this item
  id?: string;

  // The code.
  code: Cord;

  // The programming language of the code.
  programmingLanguage?: string;

  constructor(code: Cord, options?: CodeBlock) {
    if (options) Object.assign(this, options)
    this.code = code;
  }
}
