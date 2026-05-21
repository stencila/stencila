// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";
import { GraphEvidenceConfidence } from "./GraphEvidenceConfidence.js";
import { GraphEvidenceKind } from "./GraphEvidenceKind.js";
import { type Object } from "./Object.js";
import { ThingVariantOrString } from "./ThingVariantOrString.js";
import { Timestamp } from "./Timestamp.js";

/**
 * Evidence for a graph edge.
 */
export class GraphEvidence extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "GraphEvidence";

  /**
   * The kind of evidence.
   */
  kind: GraphEvidenceKind;

  /**
   * The confidence in the evidence.
   */
  confidence?: GraphEvidenceConfidence;

  /**
   * The source of the evidence.
   */
  source?: ThingVariantOrString;

  /**
   * When this evidence was recorded.
   */
  recordedAt?: Timestamp;

  /**
   * Additional structured details about the evidence for machine consumers.
   */
  details?: Object;

  /**
   * A human-readable description of the evidence.
   */
  description?: string;

  constructor(kind: GraphEvidenceKind, options?: Partial<GraphEvidence>) {
    super();
    this.type = "GraphEvidence";
    if (options) Object.assign(this, options);
    this.kind = kind;
  }
}

/**
* Create a new `GraphEvidence`
*/
export function graphEvidence(kind: GraphEvidenceKind, options?: Partial<GraphEvidence>): GraphEvidence {
  return new GraphEvidence(kind, options);
}
