// Generated file; do not edit. See `../rust/schema-gen` crate.

import { hydrate } from "../hydrate.js";

import { type Cite } from "./Cite.js";
import { type Text } from "./Text.js";

/**
 * `Cite` or `Text`
 */
export type CiteOrText =
  Cite |
  Text;

/**
 * Create a `CiteOrText` from an object
 */
export function citeOrText(other: CiteOrText): CiteOrText {
  switch(other.type) {
    case "Cite":
    case "Text":
      return hydrate(other) as CiteOrText
    default:
      throw new Error(`Unexpected type for CiteOrText: ${other.type}`);
  }
}
