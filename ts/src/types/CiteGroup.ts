// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Cite } from "./Cite.js";
import { Entity } from "./Entity.js";

/**
 * A group of `Cite` nodes.
 */
export class CiteGroup extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "CiteGroup";

  /**
   * One or more `Cite`s to be referenced in the same surrounding text.
   */
  items: Cite[];

  constructor(items: Cite[], options?: Partial<CiteGroup>) {
    super();
    this.type = "CiteGroup";
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
