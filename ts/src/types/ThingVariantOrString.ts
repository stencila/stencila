// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { hydrate } from "../hydrate.js";

import { type ThingVariant } from "./ThingVariant.js";

/**
 * `ThingVariant` or `string`
 */
export type ThingVariantOrString =
  ThingVariant |
  string;

/**
 * Create a `ThingVariantOrString` from an object
 */
export function thingVariantOrString(other: ThingVariantOrString): ThingVariantOrString {
  if (other == null || typeof other !== "object" || Array.isArray(other) || typeof other.type === "undefined") {
    return other as ThingVariantOrString;
  }
  switch(other.type) {
    case "ThingVariant":
      return hydrate(other) as ThingVariantOrString
    default:
      throw new Error(`Unexpected type for ThingVariantOrString: ${other.type}`);
  }
}
