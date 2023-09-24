// Generated file; do not edit. See `../rust/schema-gen` crate.
            
import { Directory } from './Directory'
import { File } from './File'

// `File` or `Directory`
export type FileOrDirectory =
  File |
  Directory;

export function fileOrDirectory(other: FileOrDirectory): FileOrDirectory {
  switch(other.type) {
    case "File": return File.from(other as File);
    case "Directory": return Directory.from(other as Directory);
    default: throw new Error(`Unexpected type for FileOrDirectory: ${other.type}`)
  }
}
