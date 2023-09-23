// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Cord } from './Cord';
import { Entity } from './Entity';

// Textual content
export class Text extends Entity {
  type = "Text";

  // The value of the text content
  value: Cord;

  constructor(value: Cord, options?: Text) {
    super()
    if (options) Object.assign(this, options)
    this.value = value;
  }
}
