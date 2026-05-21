// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";
import { GraphEdgeKind } from "./GraphEdgeKind.js";
import { GraphEvidence } from "./GraphEvidence.js";

/**
 * A directed edge in a graph.
 */
export class GraphEdge extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "GraphEdge";

  /**
   * The id of the upstream dependency graph node.
   */
  source: string;

  /**
   * The id of the downstream dependant graph node.
   */
  target: string;

  /**
   * The kind of dependency relationship represented by this edge.
   */
  kind: GraphEdgeKind;

  /**
   * Evidence supporting the edge.
   */
  evidence?: GraphEvidence[];

  constructor(source: string, target: string, kind: GraphEdgeKind, options?: Partial<GraphEdge>) {
    super();
    this.type = "GraphEdge";
    if (options) Object.assign(this, options);
    this.source = source;
    this.target = target;
    this.kind = kind;
  }
}

/**
* Create a new `GraphEdge`
*/
export function graphEdge(source: string, target: string, kind: GraphEdgeKind, options?: Partial<GraphEdge>): GraphEdge {
  return new GraphEdge(source, target, kind, options);
}
