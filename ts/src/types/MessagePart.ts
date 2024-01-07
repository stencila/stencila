// Generated file; do not edit. See `../rust/schema-gen` crate.

import { hydrate } from "../hydrate.js";

import { type AudioObject } from "./AudioObject.js";
import { type ImageObject } from "./ImageObject.js";
import { type VideoObject } from "./VideoObject.js";

/**
 * A union type for a part of a message.
 */
export type MessagePart =
  string |
  ImageObject |
  AudioObject |
  VideoObject;

/**
 * Create a `MessagePart` from an object
 */
export function messagePart(other: MessagePart): MessagePart {
  if (other == null || typeof other !== "object" || Array.isArray(other) || typeof other.type === "undefined") {
    return other as MessagePart;
  }
  switch(other.type) {
    case "ImageObject":
    case "AudioObject":
    case "VideoObject":
      return hydrate(other) as MessagePart
    default:
      throw new Error(`Unexpected type for MessagePart: ${other.type}`);
  }
}
