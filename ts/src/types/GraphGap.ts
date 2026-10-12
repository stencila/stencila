// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";
import { GraphGapState } from "./GraphGapState.js";
import { GraphGapTarget } from "./GraphGapTarget.js";
import { ThingVariantOrString } from "./ThingVariantOrString.js";

/**
 * A sourced limitation of an account, occurrence, relationship or evidentiary basis.
 */
export class GraphGap extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "GraphGap";

  /**
   * The distinct availability or coverage state.
   */
  state: GraphGapState;

  /**
   * The producer whose account this gap qualifies.
   */
  source: ThingVariantOrString;

  /**
   * The structured scope of the limitation.
   */
  target: GraphGapTarget;

  /**
   * The affected resource's durable identity, never a graph-local node id.
   */
  resource?: string;

  /**
   * A stable producer-specific reason code.
   */
  code?: string;

  /**
   * Human-readable scope of the limitation.
   */
  description: string;

  constructor(state: GraphGapState, source: ThingVariantOrString, target: GraphGapTarget, description: string, options?: Partial<GraphGap>) {
    super();
    this.type = "GraphGap";
    if (options) Object.assign(this, options);
    this.state = state;
    this.source = source;
    this.target = target;
    this.description = description;
  }
}

/**
* Create a new `GraphGap`
*/
export function graphGap(state: GraphGapState, source: ThingVariantOrString, target: GraphGapTarget, description: string, options?: Partial<GraphGap>): GraphGap {
  return new GraphGap(state, source, target, description, options);
}
