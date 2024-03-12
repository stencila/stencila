// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";
import { SuggestionStatus } from "./SuggestionStatus.js";

/**
 * Abstract base type for nodes that indicate a suggested change to content.
 */
export class Suggestion extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Suggestion";

  /**
   * The status of the suggestion including whether it is proposed, accepted, or rejected.
   */
  suggestionStatus?: SuggestionStatus;

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
