// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { hydrate } from "../hydrate.js";

import { type ContainerImage } from "./ContainerImage.js";

/**
 * `ContainerImage` or `string`
 */
export type ContainerImageOrString =
  ContainerImage |
  string;

/**
 * Create a `ContainerImageOrString` from an object
 */
export function containerImageOrString(other: ContainerImageOrString): ContainerImageOrString {
  if (other == null || typeof other !== "object" || Array.isArray(other) || typeof other.type === "undefined") {
    return other as ContainerImageOrString;
  }
  switch(other.type) {
    case "ContainerImage":
      return hydrate(other) as ContainerImageOrString
    default:
      throw new Error(`Unexpected type for ContainerImageOrString: ${other.type}`);
  }
}
