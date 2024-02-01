// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { hydrate } from "../hydrate.js";

import { type PropertyValue } from "./PropertyValue.js";

/**
 * `PropertyValue` or `string`
 */
export type PropertyValueOrString =
  PropertyValue |
  string;

/**
 * Create a `PropertyValueOrString` from an object
 */
export function propertyValueOrString(other: PropertyValueOrString): PropertyValueOrString {
  if (other == null || typeof other !== "object" || Array.isArray(other) || typeof other.type === "undefined") {
    return other as PropertyValueOrString;
  }
  switch(other.type) {
    case "PropertyValue":
      return hydrate(other) as PropertyValueOrString
    default:
      // @ts-expect-error that this can never happen because this function may be used in weakly-typed JavaScript
      throw new Error(`Unexpected type for PropertyValueOrString: ${other.type}`);
  }
}
