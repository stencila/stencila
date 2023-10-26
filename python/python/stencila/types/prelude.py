import sys

if sys.version_info >= (3, 11):
    from enum import StrEnum
else:
    from strenum import StrEnum

from typing import Any, ForwardRef, Literal, List, Optional, Union

from dataclasses import dataclass, field

from dataclasses_json import DataClassJsonMixin
