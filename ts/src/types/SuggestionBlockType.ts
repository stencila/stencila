// Generated file; do not edit. See `../rust/schema-gen` crate.

import { hydrate } from "../hydrate.js";

import { type DeleteBlock } from "./DeleteBlock.js";
import { type InsertBlock } from "./InsertBlock.js";
import { type ModifyBlock } from "./ModifyBlock.js";
import { type ReplaceBlock } from "./ReplaceBlock.js";

/**
 * Union type for all types that are descended from `SuggestionBlock`
 */
export type SuggestionBlockType =
  DeleteBlock |
  InsertBlock |
  ModifyBlock |
  ReplaceBlock;

/**
 * Create a `SuggestionBlockType` from an object
 */
export function suggestionBlockType(other: SuggestionBlockType): SuggestionBlockType {
  switch(other.type) {
    case "DeleteBlock":
    case "InsertBlock":
    case "ModifyBlock":
    case "ReplaceBlock":
      return hydrate(other) as SuggestionBlockType
    default:
      throw new Error(`Unexpected type for SuggestionBlockType: ${other.type}`);
  }
}
