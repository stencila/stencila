// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Cord } from "./Cord.js";
import { Entity } from "./Entity.js";

/**
 * Textual content.
 */
export class Text extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Text";

  /**
   * The value of the text content
   */
  value: Cord;

  constructor(value: Cord, options?: Partial<Text>) {
    super();
    this.type = "Text";
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
