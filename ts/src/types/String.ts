// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { hydrate } from "../hydrate.js";

/**
 * `string`
 */
export type String =
  string;

/**
 * Create a `String` from an object
 */
export function string(other: String): String {
  if (other == null || typeof other !== "object" || Array.isArray(other) || typeof other.type === "undefined") {
    return other as String;
  }
  switch(other.type) {
    
      return hydrate(other) as String
    default:
      // @ts-expect-error that this can never happen because this function may be used in weakly-typed JavaScript
      throw new Error(`Unexpected type for String: ${other.type}`);
  }
}
