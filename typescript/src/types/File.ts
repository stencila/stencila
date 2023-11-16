// Generated file; do not edit. See `../rust/schema-gen` crate.

import { CreativeWork } from "./CreativeWork.js";

/**
 * A file on the file system.
 */
export class File extends CreativeWork {
  type = "File";

  /**
   * The path (absolute or relative) of the file on the filesystem
   */
  path: string;

  constructor(path: string, options?: Partial<File>) {
    super();
    if (options) Object.assign(this, options);
    this.path = path;
  }
}

/**
* Create a new `File`
*/
export function file(path: string, options?: Partial<File>): File {
  return new File(path, options);
}
