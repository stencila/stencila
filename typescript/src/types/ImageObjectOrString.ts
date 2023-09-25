// Generated file; do not edit. See `../rust/schema-gen` crate.
            
import { ImageObject } from "./ImageObject.js";

// `ImageObject` or `string`
export type ImageObjectOrString =
  ImageObject |
  string;

export function imageObjectOrStringFrom(other: ImageObjectOrString): ImageObjectOrString {
  if (other == null || typeof other !== "object" || Array.isArray(other) || typeof other.type === "undefined") {
    return other as ImageObjectOrString;
  }
  switch(other.type) {
    case "ImageObject": return ImageObject.from(other as ImageObject);
    default: throw new Error(`Unexpected type for ImageObjectOrString: ${other.type}`);
  }
}
