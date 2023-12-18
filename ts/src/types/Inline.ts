// Generated file; do not edit. See `../rust/schema-gen` crate.

import { hydrate } from "../hydrate.js";

import { type AudioObject } from "./AudioObject.js";
import { type Button } from "./Button.js";
import { type Cite } from "./Cite.js";
import { type CiteGroup } from "./CiteGroup.js";
import { type CodeExpression } from "./CodeExpression.js";
import { type CodeInline } from "./CodeInline.js";
import { type Date } from "./Date.js";
import { type DateTime } from "./DateTime.js";
import { type DeleteInline } from "./DeleteInline.js";
import { type Duration } from "./Duration.js";
import { type Emphasis } from "./Emphasis.js";
import { type ImageObject } from "./ImageObject.js";
import { type InsertInline } from "./InsertInline.js";
import { type InstructionInline } from "./InstructionInline.js";
import { type Integer } from "./Integer.js";
import { type Link } from "./Link.js";
import { type MathInline } from "./MathInline.js";
import { type MediaObject } from "./MediaObject.js";
import { type ModifyInline } from "./ModifyInline.js";
import { type Note } from "./Note.js";
import { type Parameter } from "./Parameter.js";
import { type QuoteInline } from "./QuoteInline.js";
import { type ReplaceInline } from "./ReplaceInline.js";
import { type Strikeout } from "./Strikeout.js";
import { type Strong } from "./Strong.js";
import { type StyledInline } from "./StyledInline.js";
import { type Subscript } from "./Subscript.js";
import { type Superscript } from "./Superscript.js";
import { type Text } from "./Text.js";
import { type Time } from "./Time.js";
import { type Timestamp } from "./Timestamp.js";
import { type Underline } from "./Underline.js";
import { type UnsignedInteger } from "./UnsignedInteger.js";
import { type VideoObject } from "./VideoObject.js";

/**
 * Union type for valid inline content.
 */
export type Inline =
  AudioObject |
  Button |
  Cite |
  CiteGroup |
  CodeExpression |
  CodeInline |
  Date |
  DateTime |
  DeleteInline |
  Duration |
  Emphasis |
  ImageObject |
  InsertInline |
  InstructionInline |
  Link |
  MathInline |
  MediaObject |
  ModifyInline |
  Note |
  Parameter |
  QuoteInline |
  ReplaceInline |
  StyledInline |
  Strikeout |
  Strong |
  Subscript |
  Superscript |
  Text |
  Time |
  Timestamp |
  Underline |
  VideoObject |
  null |
  boolean |
  Integer |
  UnsignedInteger |
  number;

/**
 * Create a `Inline` from an object
 */
export function inline(other: Inline): Inline {
  if (other == null || typeof other !== "object" || Array.isArray(other) || typeof other.type === "undefined") {
    return other as Inline;
  }
  switch(other.type) {
    case "AudioObject":
    case "Button":
    case "Cite":
    case "CiteGroup":
    case "CodeExpression":
    case "CodeInline":
    case "Date":
    case "DateTime":
    case "DeleteInline":
    case "Duration":
    case "Emphasis":
    case "ImageObject":
    case "InsertInline":
    case "InstructionInline":
    case "Link":
    case "MathInline":
    case "MediaObject":
    case "ModifyInline":
    case "Note":
    case "Parameter":
    case "QuoteInline":
    case "ReplaceInline":
    case "StyledInline":
    case "Strikeout":
    case "Strong":
    case "Subscript":
    case "Superscript":
    case "Text":
    case "Time":
    case "Timestamp":
    case "Underline":
    case "VideoObject":
      return hydrate(other) as Inline
    default:
      throw new Error(`Unexpected type for Inline: ${other.type}`);
  }
}
