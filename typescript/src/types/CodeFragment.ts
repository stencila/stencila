// Generated file. Do not edit; see `rust/schema-gen` crate.

import { String } from './String';

// Inline code.
export class CodeFragment {
  // The type of this item
  type = "CodeFragment";

  // The identifier for this item
  id?: String;

  // The code.
  code: String;

  // The programming language of the code.
  programmingLanguage?: String;

  // Media type, typically expressed using a MIME format, of the code.
  mediaType?: String;

  constructor(code: String, options?: CodeFragment) {
    if (options) Object.assign(this, options)
    this.code = code;
  }
}
