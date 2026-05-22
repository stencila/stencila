// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { hydrate } from "../hydrate.js";

import { type SoftwareApplication } from "./SoftwareApplication.js";
import { type SoftwareSourceCode } from "./SoftwareSourceCode.js";

/**
 * `SoftwareApplication` or `SoftwareSourceCode` or `string`
 */
export type SoftwareApplicationOrSoftwareSourceCodeOrString =
  SoftwareApplication |
  SoftwareSourceCode |
  string;

/**
 * Create a `SoftwareApplicationOrSoftwareSourceCodeOrString` from an object
 */
export function softwareApplicationOrSoftwareSourceCodeOrString(other: SoftwareApplicationOrSoftwareSourceCodeOrString): SoftwareApplicationOrSoftwareSourceCodeOrString {
  if (other == null || typeof other !== "object" || Array.isArray(other) || typeof other.type === "undefined") {
    return other as SoftwareApplicationOrSoftwareSourceCodeOrString;
  }
  switch(other.type) {
    case "SoftwareApplication":
    case "SoftwareSourceCode":
      return hydrate(other) as SoftwareApplicationOrSoftwareSourceCodeOrString
    default:
      // @ts-expect-error that this can never happen because this function may be used in weakly-typed JavaScript
      throw new Error(`Unexpected type for SoftwareApplicationOrSoftwareSourceCodeOrString: ${other.type}`);
  }
}
