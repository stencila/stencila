// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";
import { Node } from "./Node.js";

/**
 * A node in a graph.
 */
export class GraphNode extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "GraphNode";

  /**
   * The durable graph-local id used by graph edges to reference this graph node.
   */
  declare id: string;

  /**
   * The embedded Stencila node represented by this graph node.
   */
  node: Node;

  constructor(id: string, node: Node, options?: Partial<GraphNode>) {
    super();
    this.type = "GraphNode";
    if (options) Object.assign(this, options);
    this.id = id;
    this.node = node;
  }
}

/**
* Create a new `GraphNode`
*/
export function graphNode(id: string, node: Node, options?: Partial<GraphNode>): GraphNode {
  return new GraphNode(id, node, options);
}
