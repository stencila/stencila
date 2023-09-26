// Generated file; do not edit. See `../rust/schema-gen` crate.

import { hydrate } from "../hydrate.js";

import { AudioObject } from "./AudioObject.js";
import { Button } from "./Button.js";
import { Cite } from "./Cite.js";
import { CiteGroup } from "./CiteGroup.js";
import { CodeExpression } from "./CodeExpression.js";
import { CodeFragment } from "./CodeFragment.js";
import { Date } from "./Date.js";
import { DateTime } from "./DateTime.js";
import { Delete } from "./Delete.js";
import { Duration } from "./Duration.js";
import { Emphasis } from "./Emphasis.js";
import { ImageObject } from "./ImageObject.js";
import { Insert } from "./Insert.js";
import { Integer } from "./Integer.js";
import { Link } from "./Link.js";
import { MathFragment } from "./MathFragment.js";
import { Note } from "./Note.js";
import { Parameter } from "./Parameter.js";
import { Quote } from "./Quote.js";
import { Span } from "./Span.js";
import { Strikeout } from "./Strikeout.js";
import { Strong } from "./Strong.js";
import { Subscript } from "./Subscript.js";
import { Superscript } from "./Superscript.js";
import { Text } from "./Text.js";
import { Time } from "./Time.js";
import { Timestamp } from "./Timestamp.js";
import { Underline } from "./Underline.js";
import { VideoObject } from "./VideoObject.js";

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
  number |
  string;

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
