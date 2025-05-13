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
import { type Excerpt } from "./Excerpt.js";
import { type Figure } from "./Figure.js";
import { type File } from "./File.js";
import { type ForBlock } from "./ForBlock.js";
import { type Form } from "./Form.js";
import { type Heading } from "./Heading.js";
import { type IfBlock } from "./IfBlock.js";
import { type ImageObject } from "./ImageObject.js";
import { type IncludeBlock } from "./IncludeBlock.js";
import { type InlinesBlock } from "./InlinesBlock.js";
import { type InstructionBlock } from "./InstructionBlock.js";
import { type Island } from "./Island.js";
import { type List } from "./List.js";
import { type MathBlock } from "./MathBlock.js";
import { type Paragraph } from "./Paragraph.js";
import { type PromptBlock } from "./PromptBlock.js";
import { type QuoteBlock } from "./QuoteBlock.js";
import { type RawBlock } from "./RawBlock.js";
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
  Excerpt |
  Figure |
  File |
  ForBlock |
  Form |
  Heading |
  IfBlock |
  ImageObject |
  IncludeBlock |
  InlinesBlock |
  InstructionBlock |
  Island |
  List |
  MathBlock |
  Paragraph |
  PromptBlock |
  QuoteBlock |
  RawBlock |
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
    case "Excerpt":
    case "Figure":
    case "File":
    case "ForBlock":
    case "Form":
    case "Heading":
    case "IfBlock":
    case "ImageObject":
    case "IncludeBlock":
    case "InlinesBlock":
    case "InstructionBlock":
    case "Island":
    case "List":
    case "MathBlock":
    case "Paragraph":
    case "PromptBlock":
    case "QuoteBlock":
    case "RawBlock":
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
