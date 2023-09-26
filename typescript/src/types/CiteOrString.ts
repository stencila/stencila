// Generated file; do not edit. See `../rust/schema-gen` crate.

import { hydrate } from "../hydrate.js";

import { Cite } from "./Cite.js";

/**
 * `Cite` or `string`
 */
export type CiteOrString =
  Cite |
  string;

/**
 * Create a `CiteOrString` from an object
 */
export function citeOrString(other: CiteOrString): CiteOrString {
  if (other == null || typeof other !== "object" || Array.isArray(other) || typeof other.type === "undefined") {
    return other as CiteOrString;
  }
  switch(other.type) {
    case "Cite":
      return hydrate(other) as CiteOrString
    default:
      throw new Error(`Unexpected type for CiteOrString: ${other.type}`);
  }
}
