// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Entity } from "./Entity.js";
import { UnsignedInteger } from "./UnsignedInteger.js";

/**
 * A file on the file system.
 */
export class File extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "File";

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

  /**
   * The encoding used for the context (e.g. base64, gz)
   */
  transferEncoding?: string;

  /**
   * The size of the content in bytes
   */
  size?: UnsignedInteger;

  /**
   * The content of the file.
   */
  content?: string;

  constructor(name: string, path: string, options?: Partial<File>) {
    super();
    this.type = "File";
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
