// Generated file; do not edit. See `../rust/schema-gen` crate.
            
import { Call } from './Call'
import { Claim } from './Claim'
import { CodeBlock } from './CodeBlock'
import { CodeChunk } from './CodeChunk'
import { Division } from './Division'
import { Figure } from './Figure'
import { For } from './For'
import { Form } from './Form'
import { Heading } from './Heading'
import { If } from './If'
import { Include } from './Include'
import { List } from './List'
import { MathBlock } from './MathBlock'
import { Paragraph } from './Paragraph'
import { QuoteBlock } from './QuoteBlock'
import { Table } from './Table'
import { ThematicBreak } from './ThematicBreak'

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

export function block(other: Block): Block {
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
    default: throw new Error(`Unexpected type for Block: ${other.type}`)
  }
}
