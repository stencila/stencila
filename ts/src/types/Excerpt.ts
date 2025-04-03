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
   * A `Reference` to the `CreativeWork` that the excerpt was taken from.
   */
  nodePath: string;

  /**
   * The route to the node that was excerpted including the .
   */
  nodeAncestors: string;

  /**
   * The type of the node that was excerpted.
   */
  nodeType: string;

  /**
   * The excerpted content.
   */
  content: Block[];

  constructor(source: Reference, nodePath: string, nodeAncestors: string, nodeType: string, content: Block[], options?: Partial<Excerpt>) {
    super();
    this.type = "Excerpt";
    if (options) Object.assign(this, options);
    this.source = source;
    this.nodePath = nodePath;
    this.nodeAncestors = nodeAncestors;
    this.nodeType = nodeType;
    this.content = content;
  }
}

/**
* Create a new `Excerpt`
*/
export function excerpt(source: Reference, nodePath: string, nodeAncestors: string, nodeType: string, content: Block[], options?: Partial<Excerpt>): Excerpt {
  return new Excerpt(source, nodePath, nodeAncestors, nodeType, content, options);
}
