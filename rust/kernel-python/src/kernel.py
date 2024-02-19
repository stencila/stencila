#!/usr/bin/env python3

import io
import json
import os
import resource
import sys
import traceback

# During development, set DEV environment variable to True
dev = os.getenv("DEV") == "true"

# Define constants based on development status
READY = "READY" if dev else "\U0010ACDC"
LINE = "|" if dev else "\U0010ABBA"
EXEC = "EXEC" if dev else "\U0010B522"
EVAL = "EVAL" if dev else "\U001010CC"
FORK = "FORK" if dev else "\U0010DE70"
LIST = "LIST" if dev else "\U0010C155"
GET = "GET" if dev else "\U0010A51A"
SET = "SET" if dev else "\U00107070"
REMOVE = "REMOVE" if dev else "\U0010C41C"
END = "END" if dev else "\U0010CB40"

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

    def ndarray_to_hint(array):
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

    def ndarray_to_validator(value):
        if value.dtype in NUMPY_BOOL_TYPES:
            validator = {"type": "BooleanValidator"}
        elif value.dtype in NUMPY_INT_TYPES:
            validator = {"type": "IntegerValidator"}
        elif value.dtype in NUMPY_UINT_TYPES:
            validator = {"type": "IntegerValidator", "minimum": 0}
        elif value.dtype in NUMPY_FLOAT_TYPES:
            validator = {"type": "NumberValidator"}
        elif str(value.dtype) == "datetime64":
            validator = {"type": "TimestampValidator"}
        elif str(value.dtype) == "timedelta64":
            validator = {"type": "DurationValidator"}
        else:
            validator = None

        return {"type": "ArrayValidator", "itemsValidator": validator}

    def ndarray_to_array(array):
        return array.tolist()

except ImportError:
    NUMPY_AVAILABLE = False


# Custom serialization and hints for pandas
try:
    import pandas as pd

    PANDAS_AVAILABLE = True

    def dataframe_to_hint(df):
        columns = []
        for column_name in df.columns:
            column = df[column_name]

            hint = ndarray_to_hint(column)
            hint["type"] = "DatatableColumnHint"
            hint["name"] = str(column_name)
            hint["itemType"] = hint["itemTypes"][0]

            columns.append(hint)

        return {"type": "DatatableHint", "rows": len(df), "columns": columns}

    def dataframe_to_datatable(df):
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
                    "validator": ndarray_to_validator(column),
                }
            )

        return {"type": "Datatable", "columns": columns}

    def dataframe_from_datatable(dt):
        columns = dt.get("columns") or []
        data = {
            column.get("name") or "unnamed": column.get("values") or []
            for column in columns
        }

        return pd.DataFrame(data)

except ImportError:
    PANDAS_AVAILABLE = False

# Custom serialization for `matplotlib` plots
try:
    import matplotlib
    import matplotlib.pyplot

    MATPLOTLIB_AVAILABLE = True

    matplotlib.use("Agg")

    # Monkey patch pyplot.show to return itself to
    # indicate that an image should be returned as an output
    # rather than launching a display
    def show(*args, **kwargs):
        return matplotlib.pyplot.show

    matplotlib.pyplot.show = show

    def is_matplotlib(value):
        """Is the value a matplotlib value or return of a matplotlib call?"""
        from matplotlib.artist import Artist
        from matplotlib.figure import Figure

        if isinstance(value, (Artist, Figure)):
            return True

        # This is somewhat crude but allows for calls that return lists of
        # matplotlib types not just single objects e.g. `pyplot.plot()`
        rep = repr(value)
        return rep.startswith(("<matplotlib.", "[<matplotlib."))

    def matplotlib_to_image_object():
        """Convert the current matplotlib figure to a `ImageObject`"""
        import base64

        from matplotlib import pyplot

        image = io.BytesIO()
        pyplot.savefig(image, format="png")
        pyplot.close()

        url = "data:image/png;base64," + base64.encodebytes(image.getvalue()).decode()

        return {
            "type": "ImageObject",
            "contentUrl": url,
        }

except ImportError:
    MATPLOTLIB_AVAILABLE = False


# Serialize a Python object as JSON (falling back to a string)
def to_json(obj):
    if isinstance(obj, (bool, int, float, str)):
        return json.dumps(obj)

    if NUMPY_AVAILABLE and isinstance(obj, np.ndarray):
        return json.dumps(ndarray_to_array(obj))

    if PANDAS_AVAILABLE and isinstance(obj, pd.DataFrame):
        return json.dumps(dataframe_to_datatable(obj))

    if MATPLOTLIB_AVAILABLE and is_matplotlib(obj):
        return json.dumps(matplotlib_to_image_object())

    try:
        return json.dumps(obj)
    except:  # noqa: E722
        return str(obj)


# Deserialize a Python object from JSON (falling back to a string)
def from_json(string):
    obj = json.loads(string)

    if isinstance(obj, dict):
        typ = obj.get("type")

        if PANDAS_AVAILABLE and typ == "Datatable":
            return dataframe_from_datatable(obj)

    return obj


# Monkey patch `print` to encode individual objects (if no options used)
def print(*objects, sep=" ", end="\n", file=sys.stdout, flush=False):  # noqa: A001
    if sep != " " or end != "\n" or file != sys.stdout or flush:
        __builtins__.print(*objects, sep, end, file, flush)
    for obj in objects:
        sys.stdout.write(to_json(obj) + END + "\n")


# Create the initial context with monkey patched print
context = {"print": print}


# Execute lines of code
def execute(lines):
    # If the last line is compilable as an `eval`-able
    # expression, then return it as a value. Otherwise
    # just execute all the lines
    rest, last = lines[:-1], lines[-1]
    try:
        last = compile(last, "<code>", "eval")
    except:  # noqa: E722
        compiled = compile("\n".join(lines), "<code>", "exec")
        exec(compiled, context)
    else:
        if rest:
            joined = "\n".join(rest)
            compiled = compile(joined, "<code>", "exec")
            exec(compiled, context)
        value = eval(last, context)
        if value is not None:
            sys.stdout.write(to_json(value))


# Evaluate an expression
def evaluate(expression):
    if expression:
        value = eval(expression, context)
        sys.stdout.write(to_json(value))


# List variables in the context
def list_variables():
    for name, value in context.items():
        if name == "print":
            continue

        native_type = type(value).__name__
        node_type, hint = determine_type_and_hint(value)

        variable = {
            "type": "Variable",
            "name": name,
            "programmingLanguage": "Python",
            "nativeType": native_type,
            "nodeType": node_type,
            "hint": hint,
        }

        sys.stdout.write(json.dumps(variable) + END + "\n")


# Determine node type and value hint for a variable
def determine_type_and_hint(value):
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
    if NUMPY_AVAILABLE and isinstance(value, np.ndarray):
        return "Array", ndarray_to_hint(value)
    if PANDAS_AVAILABLE and isinstance(value, pd.DataFrame):
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


# Get a variable
def get_variable(name):
    value = context.get(name)
    if value is not None:
        sys.stdout.write(to_json(value))


# Set a variable
def set_variable(name, value):
    context[name] = from_json(value)


# Remove a variable
def remove_variable(name):
    context.pop(name, None)


# Fork the kernel instance
def fork(pipes):
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


def main():
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

            sys.stderr.write(
                to_json(
                    {
                        "type": "ExecutionMessage",
                        "level": "Error",
                        "message": str(e),
                        "errorType": type(e).__name__,
                        "stackTrace": stack_trace,
                    }
                )
                + "\n"
            )

        for stream in (sys.stdout, sys.stderr):
            stream.write(READY + "\n")


if __name__ == "__main__":
    main()
