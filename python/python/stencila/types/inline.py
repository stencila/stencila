# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

AudioObject = ForwardRef("AudioObject")
Button = ForwardRef("Button")
Cite = ForwardRef("Cite")
CiteGroup = ForwardRef("CiteGroup")
CodeExpression = ForwardRef("CodeExpression")
CodeFragment = ForwardRef("CodeFragment")
Date = ForwardRef("Date")
DateTime = ForwardRef("DateTime")
Delete = ForwardRef("Delete")
Duration = ForwardRef("Duration")
Emphasis = ForwardRef("Emphasis")
ImageObject = ForwardRef("ImageObject")
Insert = ForwardRef("Insert")
Link = ForwardRef("Link")
MathFragment = ForwardRef("MathFragment")
MediaObject = ForwardRef("MediaObject")
Note = ForwardRef("Note")
Parameter = ForwardRef("Parameter")
Quote = ForwardRef("Quote")
Span = ForwardRef("Span")
Strikeout = ForwardRef("Strikeout")
Strong = ForwardRef("Strong")
Subscript = ForwardRef("Subscript")
Superscript = ForwardRef("Superscript")
Text = ForwardRef("Text")
Time = ForwardRef("Time")
Timestamp = ForwardRef("Timestamp")
Underline = ForwardRef("Underline")
UnsignedInteger = ForwardRef("UnsignedInteger")
VideoObject = ForwardRef("VideoObject")


Inline = Union[
    AudioObject,
    Button,
    Cite,
    CiteGroup,
    CodeExpression,
    CodeFragment,
    Date,
    DateTime,
    Delete,
    Duration,
    Emphasis,
    ImageObject,
    Insert,
    Link,
    MathFragment,
    MediaObject,
    Note,
    Parameter,
    Quote,
    Span,
    Strikeout,
    Strong,
    Subscript,
    Superscript,
    Text,
    Time,
    Timestamp,
    Underline,
    VideoObject,
    None,
    bool,
    int,
    UnsignedInteger,
    float,
]
"""
Union type for valid inline content.
"""
