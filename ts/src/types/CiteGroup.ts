// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Cite } from "./Cite.js";
import { Entity } from "./Entity.js";

/**
 * A group of `Cite` nodes.
 */
export class CiteGroup extends Entity {
  type = "CiteGroup";

  /**
   * One or more `Cite`s to be referenced in the same surrounding text.
   */
  items: Cite[];

  constructor(items: Cite[], options?: Partial<CiteGroup>) {
    super();
    if (options) Object.assign(this, options);
    this.items = items;
  }
}

/**
* Create a new `CiteGroup`
*/
export function citeGroup(items: Cite[], options?: Partial<CiteGroup>): CiteGroup {
  return new CiteGroup(items, options);
}
