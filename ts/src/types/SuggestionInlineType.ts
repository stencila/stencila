// Generated file; do not edit. See `../rust/schema-gen` crate.

import { hydrate } from "../hydrate.js";

import { type DeleteInline } from "./DeleteInline.js";
import { type InsertInline } from "./InsertInline.js";
import { type ModifyInline } from "./ModifyInline.js";
import { type ReplaceInline } from "./ReplaceInline.js";

/**
 * Union type for all types that are descended from `SuggestionInline`
 */
export type SuggestionInlineType =
  DeleteInline |
  InsertInline |
  ModifyInline |
  ReplaceInline;

/**
 * Create a `SuggestionInlineType` from an object
 */
export function suggestionInlineType(other: SuggestionInlineType): SuggestionInlineType {
  switch(other.type) {
    case "DeleteInline":
    case "InsertInline":
    case "ModifyInline":
    case "ReplaceInline":
      return hydrate(other) as SuggestionInlineType
    default:
      throw new Error(`Unexpected type for SuggestionInlineType: ${other.type}`);
  }
}
