// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { hydrate } from "../hydrate.js";

import { type Annotation } from "./Annotation.js";
import { type AudioObject } from "./AudioObject.js";
import { type Button } from "./Button.js";
import { type Cite } from "./Cite.js";
import { type CiteGroup } from "./CiteGroup.js";
import { type CodeExpression } from "./CodeExpression.js";
import { type CodeInline } from "./CodeInline.js";
import { type Date } from "./Date.js";
import { type DateTime } from "./DateTime.js";
import { type Duration } from "./Duration.js";
import { type Emphasis } from "./Emphasis.js";
import { type ImageObject } from "./ImageObject.js";
import { type InstructionInline } from "./InstructionInline.js";
import { type Integer } from "./Integer.js";
import { type Link } from "./Link.js";
import { type MathInline } from "./MathInline.js";
import { type MediaObject } from "./MediaObject.js";
import { type Note } from "./Note.js";
import { type Parameter } from "./Parameter.js";
import { type QuoteInline } from "./QuoteInline.js";
import { type Strikeout } from "./Strikeout.js";
import { type Strong } from "./Strong.js";
import { type StyledInline } from "./StyledInline.js";
import { type Subscript } from "./Subscript.js";
import { type SuggestionInline } from "./SuggestionInline.js";
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
  Annotation |
  AudioObject |
  Button |
  Cite |
  CiteGroup |
  CodeExpression |
  CodeInline |
  Date |
  DateTime |
  Duration |
  Emphasis |
  ImageObject |
  InstructionInline |
  Link |
  MathInline |
  MediaObject |
  Note |
  Parameter |
  QuoteInline |
  StyledInline |
  Strikeout |
  Strong |
  Subscript |
  SuggestionInline |
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
    case "Annotation":
    case "AudioObject":
    case "Button":
    case "Cite":
    case "CiteGroup":
    case "CodeExpression":
    case "CodeInline":
    case "Date":
    case "DateTime":
    case "Duration":
    case "Emphasis":
    case "ImageObject":
    case "InstructionInline":
    case "Link":
    case "MathInline":
    case "MediaObject":
    case "Note":
    case "Parameter":
    case "QuoteInline":
    case "StyledInline":
    case "Strikeout":
    case "Strong":
    case "Subscript":
    case "SuggestionInline":
    case "Superscript":
    case "Text":
    case "Time":
    case "Timestamp":
    case "Underline":
    case "VideoObject":
      return hydrate(other) as Inline
    default:
      // @ts-expect-error that this can never happen because this function may be used in weakly-typed JavaScript
      throw new Error(`Unexpected type for Inline: ${other.type}`);
  }
}
