// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { CreativeWorkType } from "./CreativeWorkType.js";
import { Entity } from "./Entity.js";

/**
 * An excerpt from a `CreativeWork`.
 */
export class Excerpt extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Excerpt";

  /**
   * The `CreativeWork` that the excerpt was taken from.
   */
  source: CreativeWorkType;

  /**
   * The excerpted content.
   */
  content: Block[];

  constructor(source: CreativeWorkType, content: Block[], options?: Partial<Excerpt>) {
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
export function excerpt(source: CreativeWorkType, content: Block[], options?: Partial<Excerpt>): Excerpt {
  return new Excerpt(source, content, options);
}
