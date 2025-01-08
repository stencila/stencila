// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { hydrate } from "../hydrate.js";

import { type Admonition } from "./Admonition.js";
import { type AudioObject } from "./AudioObject.js";
import { type CallBlock } from "./CallBlock.js";
import { type Chat } from "./Chat.js";
import { type ChatMessage } from "./ChatMessage.js";
import { type ChatMessageGroup } from "./ChatMessageGroup.js";
import { type Claim } from "./Claim.js";
import { type CodeBlock } from "./CodeBlock.js";
import { type CodeChunk } from "./CodeChunk.js";
import { type DeleteBlock } from "./DeleteBlock.js";
import { type Figure } from "./Figure.js";
import { type File } from "./File.js";
import { type ForBlock } from "./ForBlock.js";
import { type Form } from "./Form.js";
import { type Heading } from "./Heading.js";
import { type IfBlock } from "./IfBlock.js";
import { type ImageObject } from "./ImageObject.js";
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
import { type VideoObject } from "./VideoObject.js";
import { type Walkthrough } from "./Walkthrough.js";

/**
 * Union type in block content node types.
 */
export type Block =
  Admonition |
  AudioObject |
  CallBlock |
  Chat |
  ChatMessage |
  ChatMessageGroup |
  Claim |
  CodeBlock |
  CodeChunk |
  DeleteBlock |
  Figure |
  File |
  ForBlock |
  Form |
  Heading |
  IfBlock |
  ImageObject |
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
  VideoObject |
  Walkthrough;

/**
 * Create a `Block` from an object
 */
export function block(other: Block): Block {
  switch(other.type) {
    case "Admonition":
    case "AudioObject":
    case "CallBlock":
    case "Chat":
    case "ChatMessage":
    case "ChatMessageGroup":
    case "Claim":
    case "CodeBlock":
    case "CodeChunk":
    case "DeleteBlock":
    case "Figure":
    case "File":
    case "ForBlock":
    case "Form":
    case "Heading":
    case "IfBlock":
    case "ImageObject":
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
    case "VideoObject":
    case "Walkthrough":
      return hydrate(other) as Block
    default:
      // @ts-expect-error that this can never happen because this function may be used in weakly-typed JavaScript
      throw new Error(`Unexpected type for Block: ${other.type}`);
  }
}
