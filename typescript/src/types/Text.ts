// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Cord } from './Cord';

// Textual content
export class Text {
  type = "Text";

  // The identifier for this item
  id?: string;

  // The value of the text content
  value: Cord;

  constructor(value: Cord, options?: Text) {
    if (options) Object.assign(this, options)
    this.value = value;
  }
}
