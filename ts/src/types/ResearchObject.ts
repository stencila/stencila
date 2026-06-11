// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { CreativeWork } from "./CreativeWork.js";
import { type Object } from "./Object.js";
import { ResearchObjectRelation } from "./ResearchObjectRelation.js";

/**
 * An abstract base type for research objects represented as block content.
 */
export class ResearchObject extends CreativeWork {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "ResearchObject";

  /**
   * A short label for the research object.
   */
  label?: string;

  /**
   * Content of the research object.
   */
  content: Block[];

  /**
   * Relations from this research object to other research objects.
   */
  relations?: ResearchObjectRelation[];

  /**
   * Additional metadata for the research object.
   */
  extra?: Object;

  constructor(content: Block[], options?: Partial<ResearchObject>) {
    super();
    this.type = "ResearchObject";
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `ResearchObject`
*/
export function researchObject(content: Block[], options?: Partial<ResearchObject>): ResearchObject {
  return new ResearchObject(content, options);
}
