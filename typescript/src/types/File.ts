// Generated file; do not edit. See `../rust/schema-gen` crate.

import { CreativeWork } from "./CreativeWork.js";

// A file on the filesystem
export class File extends CreativeWork {
  type = "File";

  // The name of the item.
  name: string;

  // The path (absolute or relative) of the file on the filesystem
  path: string;

  constructor(name: string, path: string, options?: File) {
    super();
    if (options) Object.assign(this, options);
    this.name = name;
    this.path = path;
  }

  static from(other: File): File {
    return new File(other.name!, other.path!, other);
  }
}
