#!/usr/bin/env python3

# We are using `pyright` for type checking. We disable these as the imports are
# not necessarily there. And the unbound variables occur because of the
# optional imports.
#
# pyright: reportMissingImports=false
# pyright: reportPossiblyUnboundVariable = false

import io
import json
import logging
import os
import resource
import sys
import traceback
import types
import warnings
from dataclasses import dataclass, field
from typing import Any, Callable, Literal, Optional, TypedDict, Union, get_type_hints

# 3.9 does not have `type` or TypeAlias.
PrimitiveType = Union[str, int, float, bool, None]


# TODO: This is just a first pass at typing. It could be improved.
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


class SoftwareApplication(TypedDict):
    type: Literal["SoftwareApplication"]
    name: str
    software_version: str
    operating_system: str


# Used to track imports
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
    node_type: str
    hint: Any
    native_hint: Optional[str]


STENCILA_LEVEL = Union[
    Literal["Exception"],
    Literal["Error"],
    Literal["Warning"],
    Literal["Info"],
    Literal["Debug"],
]


class ExecutionMessage(TypedDict, total=False):
    type: Literal["ExecutionMessage"]
    level: STENCILA_LEVEL
    message: str
    errorType: str
    stackTrace: str


# During development, set DEV environment variable to True
DEV_MODE = os.getenv("DEV") == "true"

# Define constants based on development status
READY = "READY" if DEV_MODE else "\U0010acdc"
LINE = "|" if DEV_MODE else "\U0010abba"
EXEC = "EXEC" if DEV_MODE else "\U0010b522"
EVAL = "EVAL" if DEV_MODE else "\U001010cc"
FORK = "FORK" if DEV_MODE else "\U0010de70"
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
try:
    MAXFD = os.sysconf("SC_OPEN_MAX")
except Exception:
    try:
        MAXFD = resource.getrlimit(resource.RLIMIT_NOFILE)[1]
    except Exception:
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
    """A helper class for building up a native hint"""

    parts: list[str] = field(default_factory=list)

    def push_head(self, part: str) -> None:
        self.parts.append("")
        self.parts.append(part)

    def push_data(self, part: str) -> None:
        """Note that the string here might have newlines in it"""
        self.parts.append(part)

    def to_string(self) -> str:
        return "\n".join(self.parts)


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
            convert_type = str

        length = np.size(array)

        return {
            "type": "ArrayHint",
            "length": length,
            "itemTypes": [items_type],
            "nulls": np.count_nonzero(np.isnan(array)) if length else None,
            "minimum": convert_type(np.nanmin(array)) if length else None,
            "maximum": convert_type(np.nanmax(array)) if length else None,
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
        for column_name in df.columns:
            column = df[column_name]

            # We fudge the conversion here, and so break type hints
            hint: DatatableColumnHint = ndarray_to_hint(column)  # type: ignore
            hint["type"] = "DatatableColumnHint"
            hint["name"] = str(column_name)
            hint["itemType"] = hint["itemTypes"][0]  # type: ignore

            columns.append(hint)

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
                    # ndarray and columns are noth the same, but we just need the dtype.
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
        nh.push_head("The dtypes of the Dataframe are:")
        nh.push_data(repr(value.dtypes))
        nh.push_head("The first few rows of the Dataframe are:")
        nh.push_data(repr(value.head(3)))
        nh.push_head("`describe` returns:")
        nh.push_data(repr(value.describe()))

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

        return {
            "type": "ImageObject",
            "contentUrl": url,
        }

except ImportError:
    MATPLOTLIB_AVAILABLE = False


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

    if MATPLOTLIB_AVAILABLE and is_matplotlib(obj):
        return json.dumps(matplotlib_to_image_object())

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
def execute(lines: list[str]) -> None:
    # If the last line is compilable as an `eval`-able
    # expression, then return it as a value. Otherwise
    # just execute all the lines
    rest, last = lines[:-1], lines[-1]
    try:
        last = compile(last, "<code>", "eval")
    except:  # noqa: E722
        compiled = compile("\n".join(lines), "<code>", "exec")
        exec(compiled, CONTEXT)
    else:
        if rest:
            joined = "\n".join(rest)
            compiled = compile(joined, "<code>", "exec")
            exec(compiled, CONTEXT)
        value = eval(last, CONTEXT)
        if value is not None:
            sys.stdout.write(to_json(value))


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
        "name": "python",
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
        node_type, hint = determine_type_and_hint(value)
        native_hint = determine_native_hint(value)

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
        nh.push_head("The function is described by `get_types_hints` as:")
        nh.push_data(str(th))
    except Exception:
        pass

    doc = value.__doc__
    if doc:
        nh.push_head("The docstring of the function is:")
        nh.push_data(doc)

    return nh.to_string()


def determine_native_hint(value: Any) -> str:
    if isinstance(value, Callable):
        return get_native_callable_hint(value)
    if PANDAS_AVAILABLE and isinstance(value, pd.DataFrame):
        return get_native_pandas_hint(value)

    # Default (which works fine with many types)
    nh = NativeHint()
    nh.push_head("The `repr` of this value is:")
    nh.push_data(repr(value))
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


def main() -> None:
    # Signal that ready to receive tasks
    for stream in (sys.stdout, sys.stderr):
        stream.write(READY + "\n")

    # Handle tasks
    while True:
        task = input().strip()
        if not task:
            continue

        lines = task.split(LINE)

        try:
            task_type = lines[0]

            if task_type == EXEC:
                execute(lines[1:])
            elif task_type == EVAL:
                evaluate(lines[1])
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
            else:
                raise ValueError(f"Unrecognized task: {task_type}")

        except KeyboardInterrupt:
            pass

        except Exception as e:
            stack_trace = io.StringIO()
            traceback.print_exc(file=stack_trace)
            stack_trace = stack_trace.getvalue()

            # Remove the first three lines (the header and where we were in `kernel.py`)
            # and the last line which repeats the message
            stack_trace = "\n".join(stack_trace.split("\n")[3:-1])

            # Remove the "double" exception that can be caused by re-throwing
            # the exception
            position = stack_trace.find("During handling of the above exception")
            if position:
                stack_trace = stack_trace[:position].strip()

            em: ExecutionMessage = {
                "type": "ExecutionMessage",
                "level": "Exception",
                "message": str(e),
                "errorType": type(e).__name__,
                "stackTrace": stack_trace,
            }

            sys.stderr.write(to_json(em) + "\n")

        for stream in (sys.stdout, sys.stderr):
            stream.write(READY + "\n")


if __name__ == "__main__":
    main()
