// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from "./Entity.js";

/**
 * Represents additional information about a relationship or property.
 */
export class Role extends Entity {
  type = "Role";

  constructor(options?: Partial<Role>) {
    super();
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `Role`
*/
export function role(options?: Partial<Role>): Role {
  return new Role(options);
}
