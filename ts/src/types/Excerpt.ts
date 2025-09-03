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
   * The path to the node that was excepted.
   */
  nodePath: string;

  /**
   * The types of the ancestor nodes and the node that was excerpted.
   */
  nodeTypes: string;

  /**
   * The excerpted content.
   */
  content: Block[];

  constructor(source: Reference, nodePath: string, nodeTypes: string, content: Block[], options?: Partial<Excerpt>) {
    super();
    this.type = "Excerpt";
    if (options) Object.assign(this, options);
    this.source = source;
    this.nodePath = nodePath;
    this.nodeTypes = nodeTypes;
    this.content = content;
  }
}

/**
* Create a new `Excerpt`
*/
export function excerpt(source: Reference, nodePath: string, nodeTypes: string, content: Block[], options?: Partial<Excerpt>): Excerpt {
  return new Excerpt(source, nodePath, nodeTypes, content, options);
}
