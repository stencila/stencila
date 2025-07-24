// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { hydrate } from "../hydrate.js";

import { type CreativeWorkType } from "./CreativeWorkType.js";

/**
 * `CreativeWorkType` or `string`
 */
export type CreativeWorkTypeOrString =
  CreativeWorkType |
  string;

/**
 * Create a `CreativeWorkTypeOrString` from an object
 */
export function creativeWorkTypeOrString(other: CreativeWorkTypeOrString): CreativeWorkTypeOrString {
  if (other == null || typeof other !== "object" || Array.isArray(other) || typeof other.type === "undefined") {
    return other as CreativeWorkTypeOrString;
  }
  switch(other.type) {
    case "CreativeWorkType":
      return hydrate(other) as CreativeWorkTypeOrString
    default:
      throw new Error(`Unexpected type for CreativeWorkTypeOrString: ${other.type}`);
  }
}
