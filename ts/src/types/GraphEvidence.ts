// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { CodeLocation } from "./CodeLocation.js";
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
   * The exact location in source text where the evidence was found.
   */
  codeLocation?: CodeLocation;

  /**
   * The evidence carrier or authority, when not sufficiently represented by the code location.
   */
  source?: ThingVariantOrString;

  /**
   * When this evidence was recorded.
   */
  recordedAt?: Timestamp;

  /**
   * Additional machine-readable details about the evidence.
   */
  details?: Object;

  /**
   * Optional human-readable explanation of the evidence.
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
