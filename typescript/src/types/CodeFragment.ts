// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Cord } from './Cord';

// Inline code.
export class CodeFragment {
  type = "CodeFragment";

  // The identifier for this item
  id?: string;

  // The code.
  code: Cord;

  // The programming language of the code.
  programmingLanguage?: string;

  // Media type, typically expressed using a MIME format, of the code.
  mediaType?: string;

  constructor(code: Cord, options?: CodeFragment) {
    if (options) Object.assign(this, options)
    this.code = code;
  }
}
