// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from "./Entity.js";
import { Inline } from "./Inline.js";
import { Integer } from "./Integer.js";

/**
 * A heading.
 */
export class Heading extends Entity {
  type = "Heading";

  /**
   * The level of the heading.
   */
  level: Integer = 0;

  /**
   * Content of the heading.
   */
  content: Inline[];

  constructor(level: Integer, content: Inline[], options?: Partial<Heading>) {
    super();
    if (options) Object.assign(this, options);
    this.level = level;
    this.content = content;
  }
}

/**
* Create a new `Heading`
*/
export function heading(level: Integer, content: Inline[], options?: Partial<Heading>): Heading {
  return new Heading(level, content, options);
}
