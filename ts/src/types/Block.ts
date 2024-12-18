// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { hydrate } from "../hydrate.js";

import { type Admonition } from "./Admonition.js";
import { type CallBlock } from "./CallBlock.js";
import { type ChatMessage } from "./ChatMessage.js";
import { type Claim } from "./Claim.js";
import { type CodeBlock } from "./CodeBlock.js";
import { type CodeChunk } from "./CodeChunk.js";
import { type DeleteBlock } from "./DeleteBlock.js";
import { type Figure } from "./Figure.js";
import { type ForBlock } from "./ForBlock.js";
import { type Form } from "./Form.js";
import { type Heading } from "./Heading.js";
import { type IfBlock } from "./IfBlock.js";
import { type IncludeBlock } from "./IncludeBlock.js";
import { type InsertBlock } from "./InsertBlock.js";
import { type InstructionBlock } from "./InstructionBlock.js";
import { type List } from "./List.js";
import { type MathBlock } from "./MathBlock.js";
import { type ModifyBlock } from "./ModifyBlock.js";
import { type Paragraph } from "./Paragraph.js";
import { type PromptBlock } from "./PromptBlock.js";
import { type QuoteBlock } from "./QuoteBlock.js";
import { type RawBlock } from "./RawBlock.js";
import { type ReplaceBlock } from "./ReplaceBlock.js";
import { type Section } from "./Section.js";
import { type StyledBlock } from "./StyledBlock.js";
import { type SuggestionBlock } from "./SuggestionBlock.js";
import { type Table } from "./Table.js";
import { type ThematicBreak } from "./ThematicBreak.js";
import { type Walkthrough } from "./Walkthrough.js";

/**
 * Union type in block content node types.
 */
export type Block =
  Admonition |
  CallBlock |
  ChatMessage |
  Claim |
  CodeBlock |
  CodeChunk |
  DeleteBlock |
  Figure |
  ForBlock |
  Form |
  Heading |
  IfBlock |
  IncludeBlock |
  InsertBlock |
  InstructionBlock |
  List |
  MathBlock |
  ModifyBlock |
  Paragraph |
  PromptBlock |
  QuoteBlock |
  RawBlock |
  ReplaceBlock |
  Section |
  StyledBlock |
  SuggestionBlock |
  Table |
  ThematicBreak |
  Walkthrough;

/**
 * Create a `Block` from an object
 */
export function block(other: Block): Block {
  switch(other.type) {
    case "Admonition":
    case "CallBlock":
    case "ChatMessage":
    case "Claim":
    case "CodeBlock":
    case "CodeChunk":
    case "DeleteBlock":
    case "Figure":
    case "ForBlock":
    case "Form":
    case "Heading":
    case "IfBlock":
    case "IncludeBlock":
    case "InsertBlock":
    case "InstructionBlock":
    case "List":
    case "MathBlock":
    case "ModifyBlock":
    case "Paragraph":
    case "PromptBlock":
    case "QuoteBlock":
    case "RawBlock":
    case "ReplaceBlock":
    case "Section":
    case "StyledBlock":
    case "SuggestionBlock":
    case "Table":
    case "ThematicBreak":
    case "Walkthrough":
      return hydrate(other) as Block
    default:
      // @ts-expect-error that this can never happen because this function may be used in weakly-typed JavaScript
      throw new Error(`Unexpected type for Block: ${other.type}`);
  }
}
