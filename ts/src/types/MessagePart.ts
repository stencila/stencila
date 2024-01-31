// Generated file; do not edit. See `../rust/schema-gen` crate.

import { hydrate } from "../hydrate.js";

import { type AudioObject } from "./AudioObject.js";
import { type ImageObject } from "./ImageObject.js";
import { type Text } from "./Text.js";
import { type VideoObject } from "./VideoObject.js";

/**
 * A union type for a part of a message.
 */
export type MessagePart =
  Text |
  ImageObject |
  AudioObject |
  VideoObject;

/**
 * Create a `MessagePart` from an object
 */
export function messagePart(other: MessagePart): MessagePart {
  switch(other.type) {
    case "Text":
    case "ImageObject":
    case "AudioObject":
    case "VideoObject":
      return hydrate(other) as MessagePart
    default:
      throw new Error(`Unexpected type for MessagePart: ${other.type}`);
  }
}
