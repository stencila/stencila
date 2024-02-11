// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { hydrate } from "../hydrate.js";

import { type ArrayHint } from "./ArrayHint.js";
import { type Function } from "./Function.js";
import { type Integer } from "./Integer.js";
import { type ObjectHint } from "./ObjectHint.js";
import { type StringHint } from "./StringHint.js";
import { type Unknown } from "./Unknown.js";

/**
 * Union type for hints of the value and/or structure of data.
 */
export type Hint =
  ArrayHint |
  Function |
  ObjectHint |
  StringHint |
  Unknown |
  boolean |
  Integer |
  number;

/**
 * Create a `Hint` from an object
 */
export function hint(other: Hint): Hint {
  if (other == null || typeof other !== "object" || Array.isArray(other) || typeof other.type === "undefined") {
    return other as Hint;
  }
  switch(other.type) {
    case "ArrayHint":
    case "Function":
    case "ObjectHint":
    case "StringHint":
    case "Unknown":
      return hydrate(other) as Hint
    default:
      // @ts-expect-error that this can never happen because this function may be used in weakly-typed JavaScript
      throw new Error(`Unexpected type for Hint: ${other.type}`);
  }
}
