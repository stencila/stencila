// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { Entity } from "./Entity.js";
import { Reference } from "./Reference.js";

/**
 * An excerpt from a `CreativeWork`.
 */
export class Excerpt extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Excerpt";

  /**
   * A `Reference` to the `CreativeWork` that the excerpt was taken from.
   */
  source: Reference;

  /**
   * The excerpted content.
   */
  content: Block[];

  constructor(source: Reference, content: Block[], options?: Partial<Excerpt>) {
    super();
    this.type = "Excerpt";
    if (options) Object.assign(this, options);
    this.source = source;
    this.content = content;
  }
}

/**
* Create a new `Excerpt`
*/
export function excerpt(source: Reference, content: Block[], options?: Partial<Excerpt>): Excerpt {
  return new Excerpt(source, content, options);
}
