# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *


class TimeUnit(StrEnum):
    """
    A unit in which time can be measured.
    """

    Year = "Year"
    Month = "Month"
    Week = "Week"
    Day = "Day"
    Hour = "Hour"
    Minute = "Minute"
    Second = "Second"
    Millisecond = "Millisecond"
    Microsecond = "Microsecond"
    Nanosecond = "Nanosecond"
    Picosecond = "Picosecond"
    Femtosecond = "Femtosecond"
    Attosecond = "Attosecond"
