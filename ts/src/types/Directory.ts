// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";
import { FileOrDirectory } from "./FileOrDirectory.js";

/**
 * A directory on the file system.
 */
export class Directory extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Directory";

  /**
   * The name of the directory.
   */
  name: string;

  /**
   * The path (absolute or relative) of the file on the file system.
   */
  path: string;

  /**
   * The files and other directories within this directory.
   */
  parts: FileOrDirectory[];

  constructor(name: string, path: string, parts: FileOrDirectory[], options?: Partial<Directory>) {
    super();
    this.type = "Directory";
    if (options) Object.assign(this, options);
    this.name = name;
    this.path = path;
    this.parts = parts;
  }
}

/**
* Create a new `Directory`
*/
export function directory(name: string, path: string, parts: FileOrDirectory[], options?: Partial<Directory>): Directory {
  return new Directory(name, path, parts, options);
}
