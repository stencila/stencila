#!/usr/bin/env python3

import json
import os
import sys
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


# Monkey patch `print` to encode individual objects (if no options used)
def print(*objects, sep=" ", end="\n", file=sys.stdout, flush=False):
    if sep != " " or end != "\n" or file != sys.stdout or flush:
        return __builtins__.print(*objects, sep, end, file, flush)
    for object in objects:
        sys.stdout.write(json.dumps(object) + END + "\n")


# Create the initial context with monkey patched print
context: Dict[str, Any] = {"print": print}


# Function to execute lines of code
def execute(lines: str):
    global context

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
            sys.stdout.write(json.dumps(value))


# Function to evaluate an expression
def evaluate(expression: str):
    global context

    value = eval(expression, context)
    sys.stdout.write(json.dumps(value))


# Function to list variables in the context
def list_variables():
    global context

    for name, value in context.items():
        if name == "print":
            continue

        native_type = type(value).__name__
        node_type, value_hint = determine_type_and_hint(value)

        variable = {
            "type": "Variable",
            "name": name,
            "programmingLanguage": "Python",
            "nativeType": native_type,
            "nodeType": node_type,
            "valueHint": value_hint,
        }

        sys.stdout.write(json.dumps(variable) + END + "\n")


# Function to determine node type and value hint
def determine_type_and_hint(value: Any):
    if value is None:
        return "Null", None
    elif isinstance(value, bool):
        return "Boolean", value
    elif isinstance(value, (int, float)):
        return "Number", value
    elif isinstance(value, str):
        return "String", len(value)
    elif isinstance(value, (list, tuple)):
        return "Array", len(value)
    elif isinstance(value, dict):
        return "Object", len(value)
    else:
        return "Object", None  # Fallback


# Function to get a variable
def get_variable(name: str):
    global context

    value = context.get(name)
    if value is not None:
        sys.stdout.write(json.dumps(value))


# Function to set a variable
def set_variable(name: str, value: str):
    global context

    context[name] = json.loads(value)


# Function to remove a variable
def remove_variable(name: str):
    global context

    context.pop(name, None)


# Function to fork the kernel instance
def fork(pipes: str):
    global context

    child_context = json.dumps(context)
    pid = os.fork()
    if pid == 0:
        # Child process
        sys.stdin.close()
        sys.stdout.close()
        sys.stderr.close()
        with open(pipes[0], "rb", 0) as stdin, open(pipes[1], "wb", 0) as stdout, open(
            pipes[2], "wb", 0
        ) as stderr:
            sys.stdin = stdin
            sys.stdout = stdout
            sys.stderr = stderr
            context = json.loads(pipes[3])
            sys.exit(main())
    else:
        # Parent process
        print(pid)


# Main function to handle tasks
def main():
    global context

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
            # Ignore KeyboardInterrupt
            pass
        except Exception as e:
            sys.stderr.write(
                json.dumps(
                    {
                        "type": "ExecutionError",
                        "errorType": type(e).__name__,
                        "errorMessage": str(e),
                    }
                )
                + "\n"
            )

        for stream in (sys.stdout, sys.stderr):
            stream.write(READY + "\n")


# If command-line arguments are provided, use them for IO streams and initial context
if len(sys.argv) > 1:
    with open(sys.argv[1], "r") as stdin_file, open(
        sys.argv[2], "a"
    ) as stdout_file, open(sys.argv[3], "a") as stderr_file:
        sys.stdin = stdin_file
        sys.stdout = stdout_file
        sys.stderr = stderr_file
        context = json.load(sys.stdin)

# Run the main function
if __name__ == "__main__":
    for stream in (sys.stdout, sys.stderr):
        stream.write(READY + "\n")
    main()
