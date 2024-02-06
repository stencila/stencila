// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";

/**
 * Represents additional information about a relationship or property.
 */
export class Role extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Role";

  constructor(options?: Partial<Role>) {
    super();
    this.type = "Role";
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `Role`
*/
export function role(options?: Partial<Role>): Role {
  return new Role(options);
}
