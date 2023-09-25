// Generated file; do not edit. See `../rust/schema-gen` crate.
            
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

// Union type for block content node types.
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

export function blockFrom(other: Block): Block {
  switch(other.type) {
    case "Call": return Call.from(other as Call);
    case "Claim": return Claim.from(other as Claim);
    case "CodeBlock": return CodeBlock.from(other as CodeBlock);
    case "CodeChunk": return CodeChunk.from(other as CodeChunk);
    case "Division": return Division.from(other as Division);
    case "Figure": return Figure.from(other as Figure);
    case "For": return For.from(other as For);
    case "Form": return Form.from(other as Form);
    case "Heading": return Heading.from(other as Heading);
    case "If": return If.from(other as If);
    case "Include": return Include.from(other as Include);
    case "List": return List.from(other as List);
    case "MathBlock": return MathBlock.from(other as MathBlock);
    case "Paragraph": return Paragraph.from(other as Paragraph);
    case "QuoteBlock": return QuoteBlock.from(other as QuoteBlock);
    case "Table": return Table.from(other as Table);
    case "ThematicBreak": return ThematicBreak.from(other as ThematicBreak);
    default: throw new Error(`Unexpected type for Block: ${other.type}`);
  }
}
