// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Cord } from "./Cord.js";
import { Entity } from "./Entity.js";

/**
 * Textual content.
 */
export class Text extends Entity {
  type = "Text";

  /**
   * The value of the text content
   */
  value: Cord;

  constructor(value: Cord, options?: Partial<Text>) {
    super();
    if (options) Object.assign(this, options);
    this.value = value;
  }
}

/**
* Create a new `Text`
*/
export function text(value: Cord, options?: Partial<Text>): Text {
  return new Text(value, options);
}
