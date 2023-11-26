// Generated file; do not edit. See `../rust/schema-gen` crate.

import { hydrate } from "../hydrate.js";

import { type Primitive } from "./Primitive.js";
import { type StringPatch } from "./StringPatch.js";

/**
 * `StringPatch` or `Primitive`
 */
export type StringPatchOrPrimitive =
  StringPatch |
  Primitive;

/**
 * Create a `StringPatchOrPrimitive` from an object
 */
export function stringPatchOrPrimitive(other: StringPatchOrPrimitive): StringPatchOrPrimitive {
  switch(other.type) {
    case "StringPatch":
    case "Primitive":
      return hydrate(other) as StringPatchOrPrimitive
    default:
      throw new Error(`Unexpected type for StringPatchOrPrimitive: ${other.type}`);
  }
}
