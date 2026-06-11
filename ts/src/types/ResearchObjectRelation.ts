// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";
import { ResearchObjectRelationKind } from "./ResearchObjectRelationKind.js";

/**
 * A relation from one research object to another.
 */
export class ResearchObjectRelation extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "ResearchObjectRelation";

  /**
   * The kind of relation.
   */
  kind: ResearchObjectRelationKind;

  /**
   * The target research object or external resource.
   */
  target: string;

  constructor(kind: ResearchObjectRelationKind, target: string, options?: Partial<ResearchObjectRelation>) {
    super();
    this.type = "ResearchObjectRelation";
    if (options) Object.assign(this, options);
    this.kind = kind;
    this.target = target;
  }
}

/**
* Create a new `ResearchObjectRelation`
*/
export function researchObjectRelation(kind: ResearchObjectRelationKind, target: string, options?: Partial<ResearchObjectRelation>): ResearchObjectRelation {
  return new ResearchObjectRelation(kind, target, options);
}
