// Generated file; do not edit. See `../rust/schema-gen` crate.
            
import { AudioObject } from './AudioObject'
import { Button } from './Button'
import { Cite } from './Cite'
import { CiteGroup } from './CiteGroup'
import { CodeExpression } from './CodeExpression'
import { CodeFragment } from './CodeFragment'
import { Date } from './Date'
import { DateTime } from './DateTime'
import { Duration } from './Duration'
import { Emphasis } from './Emphasis'
import { ImageObject } from './ImageObject'
import { Integer } from './Integer'
import { Link } from './Link'
import { MathFragment } from './MathFragment'
import { Note } from './Note'
import { Parameter } from './Parameter'
import { Quote } from './Quote'
import { Span } from './Span'
import { Strikeout } from './Strikeout'
import { Strong } from './Strong'
import { Subscript } from './Subscript'
import { Superscript } from './Superscript'
import { Text } from './Text'
import { Time } from './Time'
import { Timestamp } from './Timestamp'
import { Underline } from './Underline'
import { VideoObject } from './VideoObject'

// Union type for valid inline content.
export type Inline =
  AudioObject |
  Button |
  Cite |
  CiteGroup |
  CodeExpression |
  CodeFragment |
  Date |
  DateTime |
  Duration |
  Emphasis |
  ImageObject |
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
