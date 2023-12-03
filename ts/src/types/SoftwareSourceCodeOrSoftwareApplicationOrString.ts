// Generated file; do not edit. See `../rust/schema-gen` crate.

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
      throw new Error(`Unexpected type for SoftwareSourceCodeOrSoftwareApplicationOrString: ${other.type}`);
  }
}
