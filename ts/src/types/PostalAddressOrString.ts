// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { hydrate } from "../hydrate.js";

import { type PostalAddress } from "./PostalAddress.js";

/**
 * `PostalAddress` or `string`
 */
export type PostalAddressOrString =
  PostalAddress |
  string;

/**
 * Create a `PostalAddressOrString` from an object
 */
export function postalAddressOrString(other: PostalAddressOrString): PostalAddressOrString {
  if (other == null || typeof other !== "object" || Array.isArray(other) || typeof other.type === "undefined") {
    return other as PostalAddressOrString;
  }
  switch(other.type) {
    case "PostalAddress":
      return hydrate(other) as PostalAddressOrString
    default:
      throw new Error(`Unexpected type for PostalAddressOrString: ${other.type}`);
  }
}
