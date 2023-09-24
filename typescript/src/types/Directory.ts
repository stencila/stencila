// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Collection } from './Collection';
import { FileOrDirectory } from './FileOrDirectory';

// A directory on the filesystem
export class Directory extends Collection {
  type = "Directory";

  // The files and other directories that are within this directory
  parts: FileOrDirectory[];

  // The path (absolute or relative) of the file on the filesystem
  path: string;

  constructor(name: string, parts: FileOrDirectory[], path: string, options?: Directory) {
    super(parts)
    if (options) Object.assign(this, options)
    this.name = name;
    this.parts = parts;
    this.path = path;
  }

  static from(other: Directory): Directory {
    return new Directory(other.name!, other.parts!, other.path!, other)
  }
}
