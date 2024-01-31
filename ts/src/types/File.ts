// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from "./Entity.js";

/**
 * A file on the file system.
 */
export class File extends Entity {
  type = "File";

  /**
   * The name of the file.
   */
  name: string;

  /**
   * The path (absolute or relative) of the file on the file system
   */
  path: string;

  /**
   * IANA media type (MIME type).
   */
  mediaType?: string;

  constructor(name: string, path: string, options?: Partial<File>) {
    super();
    if (options) Object.assign(this, options);
    this.name = name;
    this.path = path;
  }
}

/**
* Create a new `File`
*/
export function file(name: string, path: string, options?: Partial<File>): File {
  return new File(name, path, options);
}
