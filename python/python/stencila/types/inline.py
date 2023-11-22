# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

AudioObject = ForwardRef("AudioObject")
Button = ForwardRef("Button")
Cite = ForwardRef("Cite")
CiteGroup = ForwardRef("CiteGroup")
CodeExpression = ForwardRef("CodeExpression")
CodeInline = ForwardRef("CodeInline")
Date = ForwardRef("Date")
DateTime = ForwardRef("DateTime")
DeleteInline = ForwardRef("DeleteInline")
Duration = ForwardRef("Duration")
Emphasis = ForwardRef("Emphasis")
ImageObject = ForwardRef("ImageObject")
InsertInline = ForwardRef("InsertInline")
Link = ForwardRef("Link")
MathInline = ForwardRef("MathInline")
MediaObject = ForwardRef("MediaObject")
Note = ForwardRef("Note")
Parameter = ForwardRef("Parameter")
QuoteInline = ForwardRef("QuoteInline")
Strikeout = ForwardRef("Strikeout")
Strong = ForwardRef("Strong")
StyledInline = ForwardRef("StyledInline")
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
    CodeInline,
    Date,
    DateTime,
    DeleteInline,
    Duration,
    Emphasis,
    ImageObject,
    InsertInline,
    Link,
    MathInline,
    MediaObject,
    Note,
    Parameter,
    QuoteInline,
    StyledInline,
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
