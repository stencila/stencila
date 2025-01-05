// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Author } from "./Author.js";
import { Duration } from "./Duration.js";
import { Entity } from "./Entity.js";
import { ProvenanceCount } from "./ProvenanceCount.js";
import { SuggestionStatus } from "./SuggestionStatus.js";
import { Timestamp } from "./Timestamp.js";

/**
 * Abstract base type for nodes that indicate a suggested change to content.
 */
export class Suggestion extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Suggestion";

  /**
   * The status of the suggestion including whether it is the original, or is accepted, or rejected.
   */
  suggestionStatus?: SuggestionStatus;

  /**
   * The authors of the suggestion
   */
  authors?: Author[];

  /**
   * A summary of the provenance of the content within the suggestion.
   */
  provenance?: ProvenanceCount[];

  /**
   * Time taken to generate the suggestion.
   */
  executionDuration?: Duration;

  /**
   * The timestamp when the generation ended.
   */
  executionEnded?: Timestamp;

  /**
   * Feedback on the suggestion
   */
  feedback?: string;

  constructor(options?: Partial<Suggestion>) {
    super();
    this.type = "Suggestion";
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `Suggestion`
*/
export function suggestion(options?: Partial<Suggestion>): Suggestion {
  return new Suggestion(options);
}
