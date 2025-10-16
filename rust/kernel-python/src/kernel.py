#!/usr/bin/env python3

# We are using `ruff` and `pyright` for linting and type checking.
# We disable these checks as the imports are not necessarily there.
# The unbound variables occur because of the optional imports.
#
# ruff: noqa: PLC0415 SIM105
# pyright: reportMissingImports=false
# pyright: reportPossiblyUnboundVariable = false

import io
import json
import logging
import os
import re
import resource
import sys
import traceback
import types
import warnings
from dataclasses import dataclass, field
from typing import Any, Callable, Literal, Optional, TypedDict, Union, get_type_hints

# Include separate theme.py (this gets transcluded in build so that there is a
# single kernel script)
from .theme import theme

# 3.9 does not have `type` or TypeAlias.
PrimitiveType = Union[str, int, float, bool, None]

# Types
#
# We are partially constrained by relying on 3.9 and not wanting to to import
# `typing_extensions`.
#
# `total = False` is used to allow for any of the fields to be missing.
# Not the best, but NotRequired is 3.11+.
#
# What we have here is useful, however, because this provides an overview of
# the excepted types that are returned via the API.


class ArrayHint(TypedDict):
    type: Literal["ArrayHint"]
    length: int
    itemTypes: list[str]
    nulls: Union[int, None]
    minimum: Union[PrimitiveType, None]
    maximum: Union[PrimitiveType, None]


class DatatableColumnHint(TypedDict):
    type: Literal["DatatableColumnHint"]
    name: str
    itemType: PrimitiveType


class DatatableHint(TypedDict):
    type: Literal["DatatableHint"]
    rows: int
    columns: list[DatatableColumnHint]


class Validator(TypedDict, total=False):
    type: Literal[
        "BooleanValidator",
        "IntegerValidator",
        "NumberValidator",
        "StringValidator",
        "TimestampValidator",
        "DurationValidator",
    ]
    minimum: int


class ArrayValidator(TypedDict):
    type: Literal["ArrayValidator"]
    itemsValidator: Union[Validator, None]


class DatatableColumn(TypedDict):
    type: Literal["DatatableColumn"]
    name: str
    values: list[PrimitiveType]
    validator: Union[ArrayValidator, None]


class Datatable(TypedDict):
    type: Literal["Datatable"]
    columns: list[DatatableColumn]


class ImageObject(TypedDict):
    type: Literal["ImageObject"]
    contentUrl: str
    mediaType: str


class SoftwareApplication(TypedDict):
    type: Literal["SoftwareApplication"]
    name: str
    url: str
    software_version: str
    operating_system: str


class SoftwareSourceCode(TypedDict):
    type: Literal["SoftwareSourceCode"]
    name: str
    version: str
    programming_language: str


class Variable(TypedDict):
    type: Literal["Variable"]
    name: str
    programming_language: Literal["Python"]
    native_type: str
    node_type: Optional[str]
    hint: Any
    native_hint: Optional[str]


STENCILA_LEVEL = Union[
    Literal["Exception"],
    Literal["Error"],
    Literal["Warning"],
    Literal["Info"],
    Literal["Debug"],
]


class CodeLocation(TypedDict):
    type: Literal["CodeLocation"]
    startLine: Optional[int]
    startColumn: Optional[int]
    endLine: Optional[int]
    endColumn: Optional[int]


class ExecutionMessage(TypedDict, total=False):
    type: Literal["ExecutionMessage"]
    level: STENCILA_LEVEL
    message: str
    errorType: str
    stackTrace: Optional[str]
    codeLocation: Optional[CodeLocation]


# During development, set DEV environment variable to True
DEV_MODE = os.getenv("DEV") == "true"

# Define constants based on development status
READY = "READY" if DEV_MODE else "\U0010acdc"
LINE = "|" if DEV_MODE else "\U0010abba"
EXEC = "EXEC" if DEV_MODE else "\U0010b522"
EVAL = "EVAL" if DEV_MODE else "\U001010cc"
FORK = "FORK" if DEV_MODE else "\U0010de70"
BOX = "BOX" if DEV_MODE else "\U0010b0c5"
THEME = "THEME" if DEV_MODE else "\U0010DEC0"
INFO = "INFO" if DEV_MODE else "\U0010ee15"
PKGS = "PKGS" if DEV_MODE else "\U0010bec4"
LIST = "LIST" if DEV_MODE else "\U0010c155"
GET = "GET" if DEV_MODE else "\U0010a51a"
SET = "SET" if DEV_MODE else "\U00107070"
REMOVE = "REMOVE" if DEV_MODE else "\U0010c41c"
END = "END" if DEV_MODE else "\U0010cb40"

# Try to get the maximum number of file descriptors the process can have open
# SC_OPEN_MAX "The maximum number of files that a process can have open at any
# time" sysconf(3)
# RLIMIT_NOFILE "specifies a value one greater than the maximum file descriptor
# number that can be opened by this process." getrlimit(2)
if sys.platform != "win32":
    try:
        MAXFD = os.sysconf("SC_OPEN_MAX")
    except Exception:
        try:
            MAXFD = resource.getrlimit(resource.RLIMIT_NOFILE)[1]
        except Exception:
            MAXFD = 256
else:
    MAXFD = 256

# We try and intercept the logging and warnings to write to stderr
# 1. Install logging handler to write to stderr.
# 2. Install a formatter to write log records as Stencila's `ExecutionMessage` format.
# 3. Install a warnings handler to write warnings to stderr via logging.
LOGGING_TO_STENCILA: dict[str, STENCILA_LEVEL] = {
    "CRITICAL": "Exception",
    "ERROR": "Error",
    "WARNING": "Warning",
    "INFO": "Info",
    "DEBUG": "Debug",
}


class StencilaFormatter(logging.Formatter):
    def format(self, record: logging.LogRecord) -> str:
        """Convert log record to JSON format."""
        if hasattr(record, "warning_details"):
            error_type = record.warning_details["category"]  # type: ignore
        else:
            error_type = record.name

        em: ExecutionMessage = {
            "type": "ExecutionMessage",
            "level": LOGGING_TO_STENCILA.get(record.levelname, "Error"),
            "message": record.getMessage(),
            "errorType": error_type,
        }
        return json.dumps(em) + END


# Configure handler to write to stderr
handler = logging.StreamHandler(sys.stderr)
handler.setFormatter(StencilaFormatter())


# Get root logger and add handler
logger = logging.getLogger()
logger.addHandler(handler)


# We ignore much of the extra information that warnings provide for now.
def log_warning(message, category, filename, lineno, file=None, line=None) -> None:  # type: ignore  # noqa: ANN001
    warning_details = {
        "warning_details": {
            "category": str(category.__name__),  # pyright: ignore[reportAttributeAccessIssue]
            "filename": filename,
            "lineno": lineno,
            "line": line,
            "file": file,
        }
    }
    logger.warning(message, extra=warning_details)


warnings.showwarning = log_warning


@dataclass
class NativeHint:
    """A helper class for build up a Markdown native hint"""

    blocks: list[str] = field(default_factory=list)

    def push_para(self, content: str) -> None:
        self.blocks.append(content)

    def push_code(self, code: str) -> None:
        """Note that the string here might have newlines in it"""
        if len(code) > 1000:
            code = code[:1000] + "..."
        self.blocks.append("```\n" + code + "\n```")

    def to_string(self) -> str:
        return "\n\n".join(self.blocks)


# Custom serialization and hints for numpy
try:
    import numpy as np

    NUMPY_AVAILABLE = True
    NUMPY_BOOL_TYPES = (np.bool_,)
    NUMPY_INT_TYPES = (np.byte, np.short, np.intc, np.int_, np.longlong)
    NUMPY_UINT_TYPES = (
        np.ubyte,
        np.ushort,
        np.uintc,
        np.uint,
        np.ulonglong,
    )
    NUMPY_FLOAT_TYPES = (np.half, np.single, np.double, np.longdouble)

    def ndarray_to_hint(array: np.ndarray) -> ArrayHint:
        if array.dtype in NUMPY_BOOL_TYPES:
            items_type = "Boolean"
            convert_type = bool
        elif array.dtype in NUMPY_INT_TYPES:
            items_type = "Integer"
            convert_type = int
        elif array.dtype in NUMPY_UINT_TYPES:
            items_type = "UnsignedInteger"
            convert_type = int
        elif array.dtype in NUMPY_FLOAT_TYPES:
            items_type = "Number"
            convert_type = float
        elif str(array.dtype).startswith("datetime64"):
            items_type = "Timestamp"
            convert_type = int
        elif str(array.dtype).startswith("timedelta64"):
            items_type = "Duration"
            convert_type = int
        else:
            items_type = "String"
            convert_type = None

        length = np.size(array)

        null_count = None
        if length > 0 and convert_type:
            try:
                null_count = np.count_nonzero(np.isnan(array))
            except Exception:
                pass

        return {
            "type": "ArrayHint",
            "length": length,
            "itemTypes": [items_type],
            "nulls": null_count,
            "minimum": (
                convert_type(np.nanmin(array)) if length and convert_type else None
            ),
            "maximum": (
                convert_type(np.nanmax(array)) if length and convert_type else None
            ),
        }

    def ndarray_to_validator(value: np.ndarray) -> ArrayValidator:
        # Help the type system out by predetermining the type.
        validator: Validator | None
        if value.dtype in NUMPY_BOOL_TYPES:
            validator = {"type": "BooleanValidator"}
        elif value.dtype in NUMPY_INT_TYPES:
            validator = {"type": "IntegerValidator"}
        elif value.dtype in NUMPY_UINT_TYPES:
            # validator = {"type": "IntegerValidator", "minimum": 0}
            validator = {"type": "IntegerValidator"}
        elif value.dtype in NUMPY_FLOAT_TYPES:
            validator = {"type": "NumberValidator"}
        elif str(value.dtype) == "datetime64":
            validator = {"type": "TimestampValidator"}
        elif str(value.dtype) == "timedelta64":
            validator = {"type": "DurationValidator"}
        elif value.dtype == np.object_:
            validator = {"type": "StringValidator"}
        else:
            validator = None

        return {"type": "ArrayValidator", "itemsValidator": validator}

    def ndarray_to_array(array: np.ndarray) -> list[PrimitiveType]:
        return array.tolist()

except ImportError:
    NUMPY_AVAILABLE = False


# Custom serialization and hints for pandas
try:
    import pandas as pd

    PANDAS_AVAILABLE = True

    def dataframe_to_hint(df: pd.DataFrame) -> DatatableHint:
        columns = []
        try:
            for column_name in df.columns:
                column = df[column_name]

                # We fudge the conversion here, and so break type hints
                hint: DatatableColumnHint = ndarray_to_hint(column)  # type: ignore
                hint["type"] = "DatatableColumnHint"
                hint["name"] = str(column_name)
                hint["itemType"] = hint["itemTypes"][0]  # type: ignore

                columns.append(hint)
        except Exception:
            pass

        return {"type": "DatatableHint", "rows": len(df), "columns": columns}

    def dataframe_to_datatable(df: pd.DataFrame) -> Datatable:
        columns = []
        for column_name in df.columns:
            column = df[column_name]

            values = column.tolist()
            if column.dtype in NUMPY_BOOL_TYPES:
                values = [bool(row) for row in values]
            elif column.dtype in NUMPY_INT_TYPES or column.dtype in NUMPY_UINT_TYPES:
                values = [int(row) for row in values]
            elif column.dtype in NUMPY_FLOAT_TYPES:
                values = [float(row) for row in values]

            columns.append(
                {
                    "type": "DatatableColumn",
                    "name": str(column_name),
                    "values": values,
                    # ndarray and columns are not the same, but we just need the dtype.
                    "validator": ndarray_to_validator(column),  # type: ignore
                }
            )

        return {"type": "Datatable", "columns": columns}

    def dataframe_from_datatable(dt: Datatable) -> pd.DataFrame:
        columns = dt.get("columns") or []
        data = {
            column.get("name") or "unnamed": column.get("values") or []
            for column in columns
        }

        return pd.DataFrame(data)

    def get_native_pandas_hint(value: pd.DataFrame) -> str:
        nh = NativeHint()
        nh.push_para("The `dtypes` of the `DataFrame` are:")
        nh.push_code(repr(value.dtypes))
        nh.push_para("The first few rows of the `DataFrame` are:")
        nh.push_code(repr(value.head(3)))
        nh.push_para("The `describe` method of the `DataFrame` returns:")
        nh.push_code(repr(value.describe()))

        return nh.to_string()

except ImportError:
    PANDAS_AVAILABLE = False

# Custom serialization for `matplotlib` plots
try:
    import matplotlib
    import matplotlib.pyplot

    MATPLOTLIB_AVAILABLE = True

    # Activate the `Agg` backend to avoid using the display .
    # This also works on all platforms.
    matplotlib.use("Agg")

    # Monkey patch pyplot.show to return itself to
    # indicate that an image should be returned as an output
    # rather than launching a display
    def show(*args, **kwargs):  # noqa
        return matplotlib.pyplot.show

    matplotlib.pyplot.show = show

    def is_matplotlib(value: Any) -> bool:
        """Is the value a matplotlib value or return of a matplotlib call?"""
        from matplotlib.artist import Artist
        from matplotlib.figure import Figure

        if value == matplotlib.pyplot.show or isinstance(value, (Artist, Figure)):
            return True

        # This is somewhat crude but allows for calls that return lists of
        # matplotlib types not just single objects e.g. `pyplot.plot()`
        rep = repr(value)
        return rep.startswith(("<matplotlib.", "[<matplotlib."))

    def matplotlib_to_image_object() -> ImageObject:
        """Convert the current matplotlib figure to a `ImageObject`"""
        import base64

        from matplotlib import pyplot

        image = io.BytesIO()
        pyplot.savefig(image, format="png")
        pyplot.close()

        # Use `b64encode` instead of `encodebytes` to avoid newlines every 76 characters
        url = "data:image/png;base64," + base64.b64encode(image.getvalue()).decode()

        return {"type": "ImageObject", "contentUrl": url, "mediaType": "image/png"}

except ImportError:
    MATPLOTLIB_AVAILABLE = False

# Custom serialization for Plotly
try:
    import plotly.io as pio
    from plotly.io.base_renderers import PlotlyRenderer

    PLOTLY_AVAILABLE = True

    def is_plotly(value: Any) -> bool:
        """Is the value a Plotly MIME bundle"""
        return isinstance(value, dict) and "application/vnd.plotly.v1+json" in value

    class ImageObjectRenderer(PlotlyRenderer):
        """
        A custom renderer to output a Plotly figure as an `ImageObject`.

        This is needed for when a user writes `fig.show()`. That method
        returns `None` and plotly renderers are responsible for "rendering"
        them to stdout.
        """

        def to_mimebundle(self, fig_dict: Any) -> dict[str, Any]:
            bundle = super().to_mimebundle(fig_dict)

            mime_type = next(iter(bundle))
            content = bundle[mime_type]

            image = {
                "type": "ImageObject",
                "mediaType": mime_type,
                "contentUrl": json.dumps(content),
            }
            sys.stdout.write(json.dumps(image) + END + "\n")

            # Do not return the mime bundle here otherwise Plotly
            # will print to stdout as a dict repr
            return {}

    pio.renderers["image_object_renderer"] = ImageObjectRenderer()
    pio.renderers.default = "image_object_renderer"

except ImportError:
    PLOTLY_AVAILABLE = False

# Use MIME bundle for Altair plot serialization
try:
    import altair as alt

    ALTAIR_AVAILABLE = True

    alt.renderers.enable("mimetype")

    def is_altair(value: Any) -> bool:
        """Is the value a Vega-Altair MIME bundle"""
        return isinstance(value, dict) and "application/vnd.vegalite.v5+json" in value

except ImportError:
    ALTAIR_AVAILABLE = False

# Custom serialization for Folium maps
try:
    import folium

    FOLIUM_AVAILABLE = True

    def is_folium(value: Any) -> bool:
        """Is the value a Folium map object"""
        return isinstance(value, folium.Map)

except ImportError:
    FOLIUM_AVAILABLE = False


class MimeBundleJSONEncoder(json.JSONEncoder):
    """
    JSON encoder used on mime bundles to handle objects that may
    be embedded in the bundle but which are not natively JSON serializable
    """

    def default(self, o: Any) -> Any:
        if NUMPY_AVAILABLE:
            if isinstance(o, (np.integer, np.int_)):  # type: ignore
                return int(o)
            if isinstance(o, (np.floating, np.float_)):  # type: ignore
                return float(o)
            if isinstance(o, np.ndarray):
                return o.tolist()

        if PANDAS_AVAILABLE:
            if isinstance(o, pd.Timestamp):
                return o.isoformat()
            if isinstance(o, pd.Timedelta):
                return o.total_seconds()
            if isinstance(o, pd.DataFrame):
                return o.to_dict(orient="records")
            if isinstance(o, pd.Series):
                return o.to_dict()

        if isinstance(o, complex):
            return [o.real, o.imag]

        if hasattr(o, "__dict__"):
            return o.__dict__

        if hasattr(o, "__str__"):
            return str(o)

        return super().default(o)


def mimebundle_to_image_object(bundle: Any) -> ImageObject:
    """Convert a MIME bundle to an `ImageObject`"""
    mime_type = next(iter(bundle))
    return {
        "type": "ImageObject",
        "mediaType": mime_type,
        "contentUrl": json.dumps(bundle[mime_type], cls=MimeBundleJSONEncoder),
    }


def folium_to_image_object(folium_map: Any) -> ImageObject:
    """Convert a Folium map to an ImageObject with HTML content"""
    html_content = folium_map._repr_html_()
    return {
        "type": "ImageObject",
        "mediaType": "text/html",
        "contentUrl": html_content,
    }


# Serialize a Python object as JSON
def to_json(obj: Any) -> str:
    if isinstance(obj, (bool, int, float, str)):
        return json.dumps(obj)

    if isinstance(obj, range):
        return json.dumps(list(obj))

    if NUMPY_AVAILABLE and isinstance(obj, np.ndarray):  # pyright: ignore
        return json.dumps(ndarray_to_array(obj))

    if PANDAS_AVAILABLE and isinstance(obj, pd.DataFrame):  # pyright: ignore
        return json.dumps(dataframe_to_datatable(obj))

    if FOLIUM_AVAILABLE and is_folium(obj):
        return json.dumps(folium_to_image_object(obj))

    if MATPLOTLIB_AVAILABLE and is_matplotlib(obj):
        return json.dumps(matplotlib_to_image_object())

    if hasattr(obj, "_repr_mimebundle_"):
        bundle = obj._repr_mimebundle_()

        # Altair returns a tuple of bundles, the second item empty, so
        # if a tuple or list, just take the first item
        if isinstance(bundle, (tuple, list)) and len(bundle):
            bundle = bundle[0]

        if (PLOTLY_AVAILABLE and is_plotly(bundle)) or (
            ALTAIR_AVAILABLE and is_altair(bundle)
        ):
            return json.dumps(mimebundle_to_image_object(bundle))

        return json.dumps(bundle, cls=MimeBundleJSONEncoder)

    try:
        return json.dumps(obj)
    except:  # noqa: E722
        return str(obj)  # Fall back to serializing as a JSON string


# Deserialize a Python object from JSON
def from_json(string: str) -> Any:
    try:
        obj = json.loads(string)
    except:  # noqa: E722
        return string  # Fall back to deserializing as a string

    if isinstance(obj, dict):
        typ = obj.get("type")

        if PANDAS_AVAILABLE and typ == "Datatable":
            return dataframe_from_datatable(obj)  # type: ignore

    return obj


# Monkey patch `print` to encode individual objects (if no options used)
def print(*objects, sep=" ", end="\n", file=sys.stdout, flush=False):  # noqa
    if sep != " " or end != "\n" or file != sys.stdout or flush:
        __builtins__.print(*objects, sep, end, file, flush)
    for obj in objects:
        sys.stdout.write(to_json(obj) + END + "\n")


# Create the initial context with monkey patched print
CONTEXT: dict[str, Any] = {"print": print}


# Execute lines of code
def execute(lines: list[str], file: str) -> None:
    value = None
    buffer = ""
    for index, line in enumerate(lines):
        # If the current line is empty, indented, or starts with
        # certain keywords, continue to accumulate to buffer.
        if (
            line.startswith(("elif", "else", "except", "finally", " ", "\t"))
            or len(line.strip()) == 0
        ):
            buffer += line + "\n"
            continue

        # If any code in buffer try to execute it
        if len(buffer.strip()) > 0:
            try:
                # First, try to compile and evaluate as an expression
                compiled = compile(buffer, file, "eval")
                value = eval(compiled, CONTEXT)
            except SyntaxError:
                # Not an expression, try to execute as statements
                try:
                    compiled = compile(buffer, file, "exec")
                    exec(compiled, CONTEXT)
                except Exception:
                    # Failed so just add this line to buffer and continue
                    buffer += line + "\n"
                    continue

        # If successfully executed, reset buffer to empty
        # lines (so line numbering works) plus this line
        buffer = ("\n" * index) + line + "\n"

    # If any buffer remains, evaluate or execute it.
    if len(buffer.strip()) > 0:
        try:
            # First, try to compile and evaluate as an expression
            compiled = compile(buffer, file, "eval")
            value = eval(compiled, CONTEXT)
        except SyntaxError:
            # Not an expression, try to execute as statements
            # If there is an exception it will be thrown from here (as intended)
            compiled = compile(buffer, file, "exec")
            exec(compiled, CONTEXT)

    # Output the last value (if any)
    if value is not None:
        sys.stdout.write(to_json(value) + END + "\n")


# Evaluate an expression
def evaluate(expression: str) -> None:
    if expression:
        value = eval(expression, CONTEXT)
        sys.stdout.write(to_json(value))


# Get runtime information
def get_info() -> None:
    import os
    import platform
    import sys

    pv = sys.version_info
    os_name = os.name
    platform = platform.platform()

    sw_app: SoftwareApplication = {
        "type": "SoftwareApplication",
        "name": "Python",
        "url": sys.executable,
        "software_version": f"{pv.major}.{pv.minor}.{pv.micro}",
        "operating_system": f"{os_name} {platform}",
    }

    sys.stdout.write(json.dumps(sw_app) + END + "\n")


# Get a list of packages available
# Uses `importlib.metadata` rather than `pkg_resources`
# which is deprecated (and less performant)
def get_packages() -> None:
    from importlib.metadata import distributions

    for distribution in distributions():
        ssc: SoftwareSourceCode = {
            "type": "SoftwareSourceCode",
            "name": distribution.metadata["Name"],
            "programming_language": "python",
            "version": distribution.version,
        }

        sys.stdout.write(json.dumps(ssc) + END + "\n")


# List variables in the CONTEXT
def list_variables() -> None:
    for name, value in CONTEXT.items():
        if name in ("__builtins__", "print") or isinstance(value, types.ModuleType):
            continue

        native_type = type(value).__name__

        try:
            node_type, hint = determine_type_and_hint(value)
        except Exception:
            node_type, hint = (None, None)

        try:
            native_hint = determine_native_hint(value)
        except Exception:
            native_hint = None

        variable: Variable = {
            "type": "Variable",
            "name": name,
            "programming_language": "Python",
            "native_type": native_type,
            "node_type": node_type,
            "hint": hint,
            "native_hint": native_hint,
        }

        sys.stdout.write(json.dumps(variable) + END + "\n")


# Determine node type and value hint for a variable
# TODO: The return type of this needs some work. Look at th API expectations
def determine_type_and_hint(value: Any) -> tuple[str, Any]:
    if value is None:
        return "Null", None
    if isinstance(value, bool):
        return "Boolean", value
    if isinstance(value, int):
        return "Integer", value
    if isinstance(value, float):
        return "Number", value
    if isinstance(value, str):
        return "String", {"type": "StringHint", "chars": len(value)}
    if isinstance(value, (list, tuple)):
        return "Array", {"type": "ArrayHint", "length": len(value)}
    if NUMPY_AVAILABLE and isinstance(value, np.ndarray):  # pyright: ignore
        return "Array", ndarray_to_hint(value)
    if PANDAS_AVAILABLE and isinstance(value, pd.DataFrame):  # pyright: ignore
        return "Datatable", dataframe_to_hint(value)
    if isinstance(value, dict):
        typ = value.get("type")
        if typ:
            return (str(typ), None)

        length = len(value)
        keys = [str(key) for key in value]
        values = [determine_type_and_hint(value)[1] for value in value.values()]
        return (
            "Object",
            {
                "type": "ObjectHint",
                "length": length,
                "keys": keys,
                "values": values,
            },
        )

    return "Object", {"type": "Unknown"}


def get_native_callable_hint(value: Callable) -> str:
    nh = NativeHint()
    try:
        # get_type_hints is a bit unreliable, but we'll try it.
        th = get_type_hints(value)
        nh.push_para("The function is described by `get_types_hints` as:")
        nh.push_code(str(th))
    except Exception:
        pass

    doc = value.__doc__
    if doc:
        nh.push_para("The docstring of the function is:")
        nh.push_code(doc)

    return nh.to_string()


def get_native_dict_hint(value: dict) -> str:
    nh = NativeHint()
    nh.push_para("The keys of the `dict` are:")
    nh.push_code(", ".join(value.keys()))
    return nh.to_string()


def determine_native_hint(value: Any) -> str:
    if isinstance(value, Callable):
        return get_native_callable_hint(value)
    if PANDAS_AVAILABLE and isinstance(value, pd.DataFrame):
        return get_native_pandas_hint(value)
    if isinstance(value, dict):
        return get_native_dict_hint(value)

    # Default (which works fine with many types)
    nh = NativeHint()
    nh.push_para("The `repr` of the variable is:")
    nh.push_code(repr(value))
    return nh.to_string()


# Get a variable
def get_variable(name: str) -> None:
    value = CONTEXT.get(name)
    if value is not None:
        sys.stdout.write(to_json(value))


# Set a variable
def set_variable(name: str, value: str) -> None:
    CONTEXT[name] = from_json(value)


# Remove a variable
def remove_variable(name: str) -> None:
    CONTEXT.pop(name, None)


# Fork the kernel instance
def fork(pipes: list[str]) -> None:
    # Fork is not available on Windows and this function should not
    # have been called but this avoids linting errors
    if sys.platform == "win32":
        raise OSError("fork() not supported in Windows")

    pid = os.fork()
    if pid == 0:
        # Close all file descriptors so that we're not interfering with
        # parent's file descriptors and so stdin, stdout and stderr get replaced below
        # using the right file descriptor indices (0, 1, 2).
        os.closerange(0, MAXFD)
        os.open(pipes[0], os.O_RDONLY)
        os.open(pipes[1], os.O_WRONLY | os.O_TRUNC)
        os.open(pipes[2], os.O_WRONLY | os.O_TRUNC)
    else:
        # Parent process: return pid of the fork
        sys.stdout.write(str(pid))


def box() -> None:
    """
    Restrict the capabilities of the kernel

    - erases potentially secret environment variables
    - restricts filesystem writes
    - restricts process management
    - restricts network access

    Functions in `pathlib` and `shutil` mostly rely on functions in `io` and `os`
    (which are patched) so are not patched directly here.
    """

    import builtins
    import os
    import socket

    # Erase environment variables
    for key in os.environ:
        if "SECRET" in key.upper() or "KEY" in key.upper() or "TOKEN" in key.upper():
            del os.environ[key]

    # Patch builtins.open to deny write access (which is also io.open in Python 3)
    builtins_open = builtins.open

    def readonly_open(file, mode="r", *args, **kwargs):  # noqa
        # Prevent any mode that implies writing
        if any(m in mode for m in ("w", "a", "+", "x")):
            readonly_permission_error()
        return builtins_open(file, mode, *args, **kwargs)

    builtins.open = readonly_open

    # Patch os.open to deny write access
    os_open = os.open

    def readonly_os_open(file, flags, *args, **kwargs):  # noqa
        if flags & os.O_WRONLY or flags & os.O_RDWR:
            readonly_permission_error()
        return os_open(file, flags, *args, **kwargs)

    os.open = readonly_os_open

    # Restrict filesystem writes
    def readonly_permission_error(*args, **kwargs):  # noqa
        raise PermissionError(
            "Write access to filesystem is restricted in boxed kernel"
        )

    # Patch functions that create or delete directories/files
    os.makedirs = readonly_permission_error
    os.mkdir = readonly_permission_error
    os.remove = readonly_permission_error
    os.removedirs = readonly_permission_error
    os.rmdir = readonly_permission_error
    os.unlink = readonly_permission_error

    # Patch functions that modify files (rename, link, truncate, etc.)
    os.rename = readonly_permission_error
    os.replace = readonly_permission_error
    os.link = readonly_permission_error
    os.symlink = readonly_permission_error
    os.truncate = readonly_permission_error

    # Patch functions that change file permissions and ownership
    os.chmod = readonly_permission_error
    os.chown = readonly_permission_error
    os.utime = readonly_permission_error

    # Restrict process management
    def process_permission_error(*args, **kwargs):  # noqa
        raise PermissionError("Process management is restricted in boxed kernel")

    # Creating processes
    os.execl = process_permission_error
    os.execle = process_permission_error
    os.execlp = process_permission_error
    os.execlpe = process_permission_error
    os.execv = process_permission_error
    os.execve = process_permission_error
    os.execvp = process_permission_error
    os.execvpe = process_permission_error
    os.fork = process_permission_error
    os.forkpty = process_permission_error
    os.popen = process_permission_error
    os.posix_spawn = process_permission_error
    os.posix_spawnp = process_permission_error
    os.spawnl = process_permission_error
    os.spawnle = process_permission_error
    os.spawnlp = process_permission_error
    os.spawnlpe = process_permission_error
    os.spawnv = process_permission_error
    os.spawnve = process_permission_error
    os.spawnvp = process_permission_error
    os.spawnvpe = process_permission_error
    os.system = process_permission_error
    os.waitid = process_permission_error
    os.waitpid = process_permission_error
    os.waitid = process_permission_error
    os.wait3 = process_permission_error
    os.wait4 = process_permission_error

    # Killing / changing priority
    os.abort = process_permission_error
    os._exit = process_permission_error
    os.kill = process_permission_error
    os.killpg = process_permission_error
    os.nice = process_permission_error
    os.setpriority = process_permission_error

    # User/group ids and management
    os.setegid = process_permission_error
    os.seteuid = process_permission_error
    os.setgid = process_permission_error
    os.setgroups = process_permission_error
    os.setpgid = process_permission_error
    os.setpgrp = process_permission_error
    os.setsid = process_permission_error

    # Restrict network access
    def network_permission_error(*args, **kwargs):  # noqa
        raise PermissionError("Network access is restricted in boxed kernel")

    socket.socket = network_permission_error
    socket.create_connection = network_permission_error


def main() -> None:
    # Signal that ready to receive tasks
    for stream in (sys.stdout, sys.stderr):
        stream.write(READY + "\n")

    # Handle tasks
    code_id = 0
    code_label = ""
    while True:
        task = input().strip()
        if not task:
            continue

        lines = task.split(LINE)

        try:
            task_type = lines[0]

            if task_type == EXEC:
                code_id += 1
                code_label = f"Code chunk #{code_id}"
                execute(lines[1:], code_label)
            elif task_type == EVAL:
                # Note: if multiple lines provided then joined with space
                evaluate(" ".join(lines[1:]))
            elif task_type == INFO:
                get_info()
            elif task_type == PKGS:
                get_packages()
            elif task_type == LIST:
                list_variables()
            elif task_type == GET:
                get_variable(lines[1])
            elif task_type == SET:
                set_variable(lines[1], lines[2])
            elif task_type == REMOVE:
                remove_variable(lines[1])
            elif task_type == FORK:
                fork(lines[1:])
            elif task_type == BOX:
                box()
            elif task_type == THEME:
                theme(lines[1], lines[2] if len(lines) > 2 else "{}")
            else:
                raise ValueError(f"Unrecognized task: {task_type}")

        except KeyboardInterrupt:
            pass

        except Exception as e:
            # Extract the traceback information
            # Create a textual repr of traceback similar to the standard and
            # gets the deepest location in the stack within the current code
            tb = traceback.extract_tb(sys.exc_info()[2])
            stack_trace = ""
            code_location = None
            for frame in tb:
                if frame.filename == __file__:
                    continue

                file = frame.filename
                location = code_label if file == code_label else f'File "{file}"'

                stack_trace += f"{location}, line {frame.lineno}, in {frame.name}\n"
                if file == code_label:
                    code_location = {
                        "type": "CodeLocation",
                        "startLine": (
                            frame.lineno - 1 if frame.lineno is not None else None
                        ),
                        # Some line and column numbers only available in Python >=3.11
                        "endLine": (
                            frame.end_lineno - 1  # type: ignore
                            if getattr(frame, "end_lineno", None) is not None
                            else None
                        ),
                        "startColumn": getattr(frame, "colno", None),
                        "endColumn": getattr(frame, "end_colno", None),
                    }

            if len(stack_trace) == 0:
                stack_trace = None

            if code_location is None:
                # Resort to parsing print of stack trace to get
                # location (for syntax errors)
                strio = io.StringIO()
                traceback.print_exc(file=strio)
                matches = re.findall(r"line (\d+)", strio.getvalue())
                if matches:
                    line = int(matches[-1]) - 1
                    code_location = {
                        "type": "CodeLocation",
                        "startLine": int(line),
                        "endLine": int(line),
                    }

            em: ExecutionMessage = {
                "type": "ExecutionMessage",
                "level": "Exception",
                "message": str(e),
                "errorType": type(e).__name__,
                "stackTrace": stack_trace,
                "codeLocation": code_location,  # type: ignore
            }
            sys.stderr.write(to_json(em) + "\n")

        for stream in (sys.stdout, sys.stderr):
            stream.write(READY + "\n")


if __name__ == "__main__":
    main()
