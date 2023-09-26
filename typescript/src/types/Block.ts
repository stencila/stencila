// Generated file; do not edit. See `../rust/schema-gen` crate.

import { hydrate } from "../hydrate.js";

import { Call } from "./Call.js";
import { Claim } from "./Claim.js";
import { CodeBlock } from "./CodeBlock.js";
import { CodeChunk } from "./CodeChunk.js";
import { Division } from "./Division.js";
import { Figure } from "./Figure.js";
import { For } from "./For.js";
import { Form } from "./Form.js";
import { Heading } from "./Heading.js";
import { If } from "./If.js";
import { Include } from "./Include.js";
import { List } from "./List.js";
import { MathBlock } from "./MathBlock.js";
import { Paragraph } from "./Paragraph.js";
import { QuoteBlock } from "./QuoteBlock.js";
import { Table } from "./Table.js";
import { ThematicBreak } from "./ThematicBreak.js";

/**
 * Union type for block content node types.
 */
export type Block =
  Call |
  Claim |
  CodeBlock |
  CodeChunk |
  Division |
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
  Table |
  ThematicBreak;

/**
 * Create a `Block` from an object
 */
export function block(other: Block): Block {
  switch(other.type) {
    case "Call":
    case "Claim":
    case "CodeBlock":
    case "CodeChunk":
    case "Division":
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
    case "Table":
    case "ThematicBreak":
      return hydrate(other) as Block
    default:
      throw new Error(`Unexpected type for Block: ${other.type}`);
  }
}
