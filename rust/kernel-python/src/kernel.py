#!/usr/bin/env python3

import json
import os
import resource
import sys
import traceback
from io import StringIO
from typing import Any, Dict

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
# SC_OPEN_MAX "The maximum number of files that a process can have open at any time" sysconf(3)
# RLIMIT_NOFILE "specifies a value one greater than the maximum file descriptor number that can be opened by this process." getrlimit(2)
try:
    MAXFD = os.sysconf("SC_OPEN_MAX")
except Exception:
    try:
        MAXFD = resource.getrlimit(resource.RLIMIT_NOFILE)[1]
    except Exception:
        MAXFD = 256

# Custom serialization/deserialization to Stencila JSON for ndarrays
try:
    import numpy as np
    from numpy import ndarray

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

    def ndarray_to_array_hint(array):
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
            "types": [items_type],
            "nulls": np.count_nonzero(np.isnan(array)) if length else None,
            "minimum": convert_type(np.nanmin(array)) if length else None,
            "maximum": convert_type(np.nanmax(array)) if length else None,
        }

except ImportError:

    class ndarray:
        pass


# Serialize a Python object as JSON (falling back to a string)
def to_json(object):
    if isinstance(object, (bool, int, float, str)):
        return json.dumps(object)

    if isinstance(object, ndarray):
        return json.dumps(object.tolist())

    try:
        return json.dumps(object)
    except:
        return str(object)


# Deserialize a Python object from JSON (falling back to a string)
def from_json(string):
    return json.loads(string)


# Monkey patch `print` to encode individual objects (if no options used)
def print(*objects, sep=" ", end="\n", file=sys.stdout, flush=False):
    if sep != " " or end != "\n" or file != sys.stdout or flush:
        return __builtins__.print(*objects, sep, end, file, flush)
    for object in objects:
        sys.stdout.write(to_json(object) + END + "\n")


# Create the initial context with monkey patched print
context: Dict[str, Any] = {"print": print}


# Execute lines of code
def execute(lines: str):
    # If the last line is compilable as an `eval`-able
    # expression, then return it as a value. Otherwise
    # just execute all the lines
    rest, last = lines[:-1], lines[-1]
    try:
        last = compile(last, "<code>", "eval")
    except:
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
def evaluate(expression: str):
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
def determine_type_and_hint(value: Any):
    if value is None:
        return "Null", None
    elif isinstance(value, bool):
        return "Boolean", value
    elif isinstance(value, int):
        return "Integer", value
    elif isinstance(value, float):
        return "Number", value
    elif isinstance(value, str):
        return "String", {"type": "StringHint", "chars": len(value)}
    elif isinstance(value, (list, tuple)):
        return "Array", {"type": "ArrayHint", "length": len(value)}
    elif isinstance(value, ndarray):
        return "Array", ndarray_to_array_hint(value)
    elif isinstance(value, dict):
        typ = value.get("type")
        if typ:
            return (str(typ), None)
        else:
            length = len(value)
            keys = [str(key) for key in value.keys()]
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

    else:
        return "Object", {"type": "Unknown"}


# Get a variable
def get_variable(name: str):
    value = context.get(name)
    if value is not None:
        sys.stdout.write(to_json(value))


# Set a variable
def set_variable(name: str, value: str):
    context[name] = from_json(value)


# Remove a variable
def remove_variable(name: str):
    context.pop(name, None)


# Fork the kernel instance
def fork(pipes: str):
    global context

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
        stack_trace = StringIO()
        traceback.print_exc(file=stack_trace)
        stack_trace = stack_trace.getvalue()

        # Remove the first three lines (the header and where we were in `kernel.py`)
        # and the last line which repeats the message
        stack_trace = "\n".join(stack_trace.split("\n")[3:-1])

        # Remove the "double" exception that can be caused by re-throwing the exception
        position = stack_trace.find("During handling of the above exception")
        if position:
            stack_trace = stack_trace[:position].strip()

        sys.stderr.write(
            to_json(
                {
                    "type": "ExecutionError",
                    "errorType": type(e).__name__,
                    "errorMessage": str(e),
                    "stackTrace": stack_trace,
                }
            )
            + "\n"
        )

    for stream in (sys.stdout, sys.stderr):
        stream.write(READY + "\n")
