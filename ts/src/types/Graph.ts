// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { CreativeWork } from "./CreativeWork.js";
import { GraphEdge } from "./GraphEdge.js";
import { GraphNode } from "./GraphNode.js";

/**
 * A directed graph relating Stencila nodes, used for provenance, reactivity, and similar relationship graphs.
 */
export class Graph extends CreativeWork {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Graph";

  /**
   * An absolute URI identifying the resource or scoped document node that the graph was generated for.
   */
  subject: string;

  /**
   * The nodes in the graph.
   */
  nodes: GraphNode[];

  /**
   * The directed edges in the graph.
   */
  edges: GraphEdge[];

  constructor(subject: string, nodes: GraphNode[], edges: GraphEdge[], options?: Partial<Graph>) {
    super();
    this.type = "Graph";
    if (options) Object.assign(this, options);
    this.subject = subject;
    this.nodes = nodes;
    this.edges = edges;
  }
}

/**
* Create a new `Graph`
*/
export function graph(subject: string, nodes: GraphNode[], edges: GraphEdge[], options?: Partial<Graph>): Graph {
  return new Graph(subject, nodes, edges, options);
}
