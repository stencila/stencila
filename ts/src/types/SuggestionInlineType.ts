// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

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
      // @ts-expect-error that this can never happen because this function may be used in weakly-typed JavaScript
      throw new Error(`Unexpected type for SuggestionInlineType: ${other.type}`);
  }
}
