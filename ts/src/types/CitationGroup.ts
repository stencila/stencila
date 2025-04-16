// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Citation } from "./Citation.js";
import { Entity } from "./Entity.js";

/**
 * A group of `Citation` nodes.
 */
export class CitationGroup extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "CitationGroup";

  /**
   * One or more `Citation`s to be referenced in the same surrounding text.
   */
  items: Citation[];

  constructor(items: Citation[], options?: Partial<CitationGroup>) {
    super();
    this.type = "CitationGroup";
    if (options) Object.assign(this, options);
    this.items = items;
  }
}

/**
* Create a new `CitationGroup`
*/
export function citationGroup(items: Citation[], options?: Partial<CitationGroup>): CitationGroup {
  return new CitationGroup(items, options);
}
