// Generated file; do not edit. See `../rust/schema-gen` crate.

import { String } from './String';
import { TextValue } from './TextValue';

// Textual content
export class Text {
  // The type of this item
  type = "Text";

  // The identifier for this item
  id?: String;

  // The value of the text content
  value: TextValue;

  constructor(value: TextValue, options?: Text) {
    if (options) Object.assign(this, options)
    this.value = value;
  }
}
