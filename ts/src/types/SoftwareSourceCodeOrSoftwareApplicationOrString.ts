// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { hydrate } from "../hydrate.js";

import { type SoftwareApplication } from "./SoftwareApplication.js";
import { type SoftwareSourceCode } from "./SoftwareSourceCode.js";

/**
 * `SoftwareSourceCode` or `SoftwareApplication` or `string`
 */
export type SoftwareSourceCodeOrSoftwareApplicationOrString =
  SoftwareSourceCode |
  SoftwareApplication |
  string;

/**
 * Create a `SoftwareSourceCodeOrSoftwareApplicationOrString` from an object
 */
export function softwareSourceCodeOrSoftwareApplicationOrString(other: SoftwareSourceCodeOrSoftwareApplicationOrString): SoftwareSourceCodeOrSoftwareApplicationOrString {
  if (other == null || typeof other !== "object" || Array.isArray(other) || typeof other.type === "undefined") {
    return other as SoftwareSourceCodeOrSoftwareApplicationOrString;
  }
  switch(other.type) {
    case "SoftwareSourceCode":
    case "SoftwareApplication":
      return hydrate(other) as SoftwareSourceCodeOrSoftwareApplicationOrString
    default:
      // @ts-expect-error that this can never happen because this function may be used in weakly-typed JavaScript
      throw new Error(`Unexpected type for SoftwareSourceCodeOrSoftwareApplicationOrString: ${other.type}`);
  }
}
