// Generated file. Do not edit; see `rust/schema-gen` crate.\n\n
            
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
