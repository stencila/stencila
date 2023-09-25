// Generated file; do not edit. See `../rust/schema-gen` crate.
            
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
    case "AudioObject": return AudioObject.from(other as AudioObject);
    case "Button": return Button.from(other as Button);
    case "Cite": return Cite.from(other as Cite);
    case "CiteGroup": return CiteGroup.from(other as CiteGroup);
    case "CodeExpression": return CodeExpression.from(other as CodeExpression);
    case "CodeFragment": return CodeFragment.from(other as CodeFragment);
    case "Date": return Date.from(other as Date);
    case "DateTime": return DateTime.from(other as DateTime);
    case "Delete": return Delete.from(other as Delete);
    case "Duration": return Duration.from(other as Duration);
    case "Emphasis": return Emphasis.from(other as Emphasis);
    case "ImageObject": return ImageObject.from(other as ImageObject);
    case "Insert": return Insert.from(other as Insert);
    case "Link": return Link.from(other as Link);
    case "MathFragment": return MathFragment.from(other as MathFragment);
    case "Note": return Note.from(other as Note);
    case "Parameter": return Parameter.from(other as Parameter);
    case "Quote": return Quote.from(other as Quote);
    case "Span": return Span.from(other as Span);
    case "Strikeout": return Strikeout.from(other as Strikeout);
    case "Strong": return Strong.from(other as Strong);
    case "Subscript": return Subscript.from(other as Subscript);
    case "Superscript": return Superscript.from(other as Superscript);
    case "Text": return Text.from(other as Text);
    case "Time": return Time.from(other as Time);
    case "Timestamp": return Timestamp.from(other as Timestamp);
    case "Underline": return Underline.from(other as Underline);
    case "VideoObject": return VideoObject.from(other as VideoObject);
    default: throw new Error(`Unexpected type for Inline: ${other.type}`);
  }
}
