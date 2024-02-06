// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

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
      // @ts-expect-error that this can never happen because this function may be used in weakly-typed JavaScript
      throw new Error(`Unexpected type for CiteOrText: ${other.type}`);
  }
}
