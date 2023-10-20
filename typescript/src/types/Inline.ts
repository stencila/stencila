// Generated file; do not edit. See `../rust/schema-gen` crate.

import { hydrate } from "../hydrate.js";

import { type AudioObject } from "./AudioObject.js";
import { type Button } from "./Button.js";
import { type Cite } from "./Cite.js";
import { type CiteGroup } from "./CiteGroup.js";
import { type CodeExpression } from "./CodeExpression.js";
import { type CodeFragment } from "./CodeFragment.js";
import { type Date } from "./Date.js";
import { type DateTime } from "./DateTime.js";
import { type Delete } from "./Delete.js";
import { type Duration } from "./Duration.js";
import { type Emphasis } from "./Emphasis.js";
import { type ImageObject } from "./ImageObject.js";
import { type Insert } from "./Insert.js";
import { type Integer } from "./Integer.js";
import { type Link } from "./Link.js";
import { type MathFragment } from "./MathFragment.js";
import { type MediaObject } from "./MediaObject.js";
import { type Note } from "./Note.js";
import { type Parameter } from "./Parameter.js";
import { type Quote } from "./Quote.js";
import { type Span } from "./Span.js";
import { type Strikeout } from "./Strikeout.js";
import { type Strong } from "./Strong.js";
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
  CodeFragment |
  Date |
  DateTime |
  Delete |
  Duration |
  Emphasis |
  ImageObject |
  Insert |
  Link |
  MathFragment |
  MediaObject |
  Note |
  Parameter |
  Quote |
  Span |
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
    case "CodeFragment":
    case "Date":
    case "DateTime":
    case "Delete":
    case "Duration":
    case "Emphasis":
    case "ImageObject":
    case "Insert":
    case "Link":
    case "MathFragment":
    case "MediaObject":
    case "Note":
    case "Parameter":
    case "Quote":
    case "Span":
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
