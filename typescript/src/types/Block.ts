// Generated file; do not edit. See `../rust/schema-gen` crate.

import { hydrate } from "../hydrate.js";

import { type Admonition } from "./Admonition.js";
import { type Call } from "./Call.js";
import { type Claim } from "./Claim.js";
import { type CodeBlock } from "./CodeBlock.js";
import { type CodeChunk } from "./CodeChunk.js";
import { type Figure } from "./Figure.js";
import { type For } from "./For.js";
import { type Form } from "./Form.js";
import { type Heading } from "./Heading.js";
import { type If } from "./If.js";
import { type Include } from "./Include.js";
import { type List } from "./List.js";
import { type MathBlock } from "./MathBlock.js";
import { type Paragraph } from "./Paragraph.js";
import { type QuoteBlock } from "./QuoteBlock.js";
import { type Section } from "./Section.js";
import { type StyledBlock } from "./StyledBlock.js";
import { type Table } from "./Table.js";
import { type ThematicBreak } from "./ThematicBreak.js";

/**
 * Union type in block content node types.
 */
export type Block =
  Admonition |
  Call |
  Claim |
  CodeBlock |
  CodeChunk |
  Figure |
  For |
  Form |
  Heading |
  If |
  Include |
  List |
  MathBlock |
  Paragraph |
  QuoteBlock |
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
    case "Call":
    case "Claim":
    case "CodeBlock":
    case "CodeChunk":
    case "Figure":
    case "For":
    case "Form":
    case "Heading":
    case "If":
    case "Include":
    case "List":
    case "MathBlock":
    case "Paragraph":
    case "QuoteBlock":
    case "Section":
    case "StyledBlock":
    case "Table":
    case "ThematicBreak":
      return hydrate(other) as Block
    default:
      throw new Error(`Unexpected type for Block: ${other.type}`);
  }
}
