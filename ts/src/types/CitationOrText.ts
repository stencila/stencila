// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { hydrate } from "../hydrate.js";

import { type Citation } from "./Citation.js";
import { type Text } from "./Text.js";

/**
 * `Citation` or `Text`
 */
export type CitationOrText =
  Citation |
  Text;

/**
 * Create a `CitationOrText` from an object
 */
export function citationOrText(other: CitationOrText): CitationOrText {
  switch(other.type) {
    case "Citation":
    case "Text":
      return hydrate(other) as CitationOrText
    default:
      // @ts-expect-error that this can never happen because this function may be used in weakly-typed JavaScript
      throw new Error(`Unexpected type for CitationOrText: ${other.type}`);
  }
}
