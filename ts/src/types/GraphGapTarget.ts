// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";
import { GraphEdgeKind } from "./GraphEdgeKind.js";
import { GraphGapDirection } from "./GraphGapDirection.js";
import { GraphGapTargetKind } from "./GraphGapTargetKind.js";

/**
 * The structured scope of a graph limitation.
 */
export class GraphGapTarget extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "GraphGapTarget";

  /**
   * Whether the limitation qualifies an item, property or relationship.
   */
  kind: GraphGapTargetKind;

  /**
   * A property path, required for a Property target.
   */
  property?: string;

  /**
   * The edge kind, required for a Relationship target.
   */
  relationship?: GraphEdgeKind;

  /**
   * The edge direction relative to the containing item, required for a Relationship target.
   */
  direction?: GraphGapDirection;

  /**
   * An optional relationship role narrowing its scope.
   */
  role?: string;

  constructor(kind: GraphGapTargetKind, options?: Partial<GraphGapTarget>) {
    super();
    this.type = "GraphGapTarget";
    if (options) Object.assign(this, options);
    this.kind = kind;
  }
}

/**
* Create a new `GraphGapTarget`
*/
export function graphGapTarget(kind: GraphGapTargetKind, options?: Partial<GraphGapTarget>): GraphGapTarget {
  return new GraphGapTarget(kind, options);
}
