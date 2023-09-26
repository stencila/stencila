// Generated file; do not edit. See `../rust/schema-gen` crate.

import { hydrate } from "../hydrate.js";

import { ImageObject } from "./ImageObject.js";

/**
 * `ImageObject` or `string`
 */
export type ImageObjectOrString =
  ImageObject |
  string;

/**
 * Create a `ImageObjectOrString` from an object
 */
export function imageObjectOrString(other: ImageObjectOrString): ImageObjectOrString {
  if (other == null || typeof other !== "object" || Array.isArray(other) || typeof other.type === "undefined") {
    return other as ImageObjectOrString;
  }
  switch(other.type) {
    case "ImageObject":
      return hydrate(other) as ImageObjectOrString
    default:
      throw new Error(`Unexpected type for ImageObjectOrString: ${other.type}`);
  }
}
