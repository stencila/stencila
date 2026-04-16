// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";

/**
 * A positional boundary marker within inline content.
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
