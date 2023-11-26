// Generated file; do not edit. See `../rust/schema-gen` crate.

import { hydrate } from "../hydrate.js";

import { type Admonition } from "./Admonition.js";
import { type CallBlock } from "./CallBlock.js";
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
import { type InstructBlock } from "./InstructBlock.js";
import { type List } from "./List.js";
import { type MathBlock } from "./MathBlock.js";
import { type ModifyBlock } from "./ModifyBlock.js";
import { type Paragraph } from "./Paragraph.js";
import { type QuoteBlock } from "./QuoteBlock.js";
import { type ReplaceBlock } from "./ReplaceBlock.js";
import { type Section } from "./Section.js";
import { type StyledBlock } from "./StyledBlock.js";
import { type Table } from "./Table.js";
import { type ThematicBreak } from "./ThematicBreak.js";

/**
 * Union type in block content node types.
 */
export type Block =
  Admonition |
  CallBlock |
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
  InstructBlock |
  List |
  MathBlock |
  ModifyBlock |
  Paragraph |
  QuoteBlock |
  ReplaceBlock |
  Section |
  StyledBlock |
  Table |
  ThematicBreak;

/**
 * Create a `Block` from an object
 */
export function block(other: Block): Block {
  switch(other.type) {
    case "Admonition":
    case "CallBlock":
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
    case "InstructBlock":
    case "List":
    case "MathBlock":
    case "ModifyBlock":
    case "Paragraph":
    case "QuoteBlock":
    case "ReplaceBlock":
    case "Section":
    case "StyledBlock":
    case "Table":
    case "ThematicBreak":
      return hydrate(other) as Block
    default:
      throw new Error(`Unexpected type for Block: ${other.type}`);
  }
}
