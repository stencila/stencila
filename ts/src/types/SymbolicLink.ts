// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";

/**
 * A symbolic link on a file system.
 */
export class SymbolicLink extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "SymbolicLink";

  /**
   * The name of the symbolic link.
   */
  name: string;

  /**
   * The path (absolute or relative) of the symbolic link on the file system.
   */
  path: string;

  /**
   * The raw target path stored by the symbolic link.
   */
  target: string;

  constructor(name: string, path: string, target: string, options?: Partial<SymbolicLink>) {
    super();
    this.type = "SymbolicLink";
    if (options) Object.assign(this, options);
    this.name = name;
    this.path = path;
    this.target = target;
  }
}

/**
* Create a new `SymbolicLink`
*/
export function symbolicLink(name: string, path: string, target: string, options?: Partial<SymbolicLink>): SymbolicLink {
  return new SymbolicLink(name, path, target, options);
}
