// Generated file; do not edit. See `../rust/schema-gen` crate.

import { CreativeWork } from './CreativeWork';

// A file on the filesystem
export class File extends CreativeWork {
  type = "File";

  // The path (absolute or relative) of the file on the filesystem
  path: string;

  constructor(name: string, path: string, options?: File) {
    super()
    if (options) Object.assign(this, options)
    this.name = name;
    this.path = path;
  }
}
