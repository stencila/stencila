// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";

/**
 * A positional marker within inline content used to define the boundary of a cross-block range. Boundaries are referenced by their `id` from other nodes (e.g. `Comment.startLocation` and `Comment.endLocation`) to delimit regions that may span across multiple blocks.
 */
export class Boundary extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Boundary";

  constructor(options?: Partial<Boundary>) {
    super();
    this.type = "Boundary";
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `Boundary`
*/
export function boundary(options?: Partial<Boundary>): Boundary {
  return new Boundary(options);
}
