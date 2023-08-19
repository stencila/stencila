# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .audio_object import AudioObject
from .button import Button
from .cite import Cite
from .cite_group import CiteGroup
from .code_expression import CodeExpression
from .code_fragment import CodeFragment
from .date import Date
from .date_time import DateTime
from .duration import Duration
from .emphasis import Emphasis
from .image_object import ImageObject
from .link import Link
from .math_fragment import MathFragment
from .note import Note
from .null import Null
from .parameter import Parameter
from .quote import Quote
from .span import Span
from .strikeout import Strikeout
from .strong import Strong
from .subscript import Subscript
from .superscript import Superscript
from .text import Text
from .time import Time
from .timestamp import Timestamp
from .underline import Underline
from .video_object import VideoObject


Inline = Union[
    AudioObject,
    Button,
    Cite,
    CiteGroup,
    CodeExpression,
    CodeFragment,
    Date,
    DateTime,
    Duration,
    Emphasis,
    ImageObject,
    Link,
    MathFragment,
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
    Null,
    bool,
    int,
    float,
    str,
]
"""
Union type for valid inline content.
"""
