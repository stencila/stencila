// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Collection } from "./Collection.js";
import { FileOrDirectory } from "./FileOrDirectory.js";

/**
 * A directory on the file system.
 */
export class Directory extends Collection {
  type = "Directory";

  /**
   * The name of the item.
   */
  name: string;

  /**
   * The files and other directories that are within this directory
   */
  parts: FileOrDirectory[];

  /**
   * The path (absolute or relative) of the file on the filesystem
   */
  path: string;

  constructor(name: string, parts: FileOrDirectory[], path: string, options?: Partial<Directory>) {
    super(parts);
    if (options) Object.assign(this, options);
    this.name = name;
    this.parts = parts;
    this.path = path;
  }
}

/**
* Create a new `Directory`
*/
export function directory(name: string, parts: FileOrDirectory[], path: string, options?: Partial<Directory>): Directory {
  return new Directory(name, parts, path, options);
}
