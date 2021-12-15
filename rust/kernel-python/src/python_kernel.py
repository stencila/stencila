#!/usr/bin/env python3

import json
import os
import resource
from sys import stdin, stdout, stderr

from python_codec import decode_value, encode_exception, encode_value

READY = u"\U0010ACDC\n"
RESULT = u"\U0010CB40\n"
TASK = u"\U0010ABBA\n"
FORK = u"\U0010DE70\n"


# Monkey patch `print` to encode individual objects (if no options used)
def print(*objects, sep=" ", end="\n", file=stdout, flush=False):
    if sep != " " or end != "\n" or file != stdout or flush:
        return __builtins__.print(*objects, sep, end, file, flush)
    for object in objects:
        json = encode_value(object)
        stdout.write(json + RESULT)


globals_dict = globals()
globals_dict.update({"print": print, "decode_value": decode_value})

locals_dict = {}

stdout.write(READY)
stdout.flush()
stderr.write(READY)
stderr.flush()

for task in stdin:
    if task.endswith(FORK):
        pid = os.fork()
        if pid > 0:
            # Parent process so just go to the next line
            continue

        # Child process, so...

        # Separate code and paths of FIFO pipes to replace stdout and stderr
        payload = task[: -len(FORK)]
        pos = payload.rfind("|")
        (code, pipes) = payload[:pos], payload[(pos + 1) :]
        (new_stdout, new_stderr) = pipes.split(";")

        # Close file descriptors so that we're not interfeering with
        # parent's file descriptors and so stdin, stdout and stderr get replaced below.
        # See https://gist.github.com/ionelmc/5038117 for a more sophisticated approach to this.
        os.closerange(0, 1024)

        # Set stdin to /dev/null to avoid getting more input
        # and to end loop on next iteration
        os.open("/dev/null", os.O_RDONLY)  # 0: stdin

        # Replace stdout and stderr with pipes
        os.open(new_stdout, os.O_WRONLY | os.O_TRUNC)  # 1: stdout
        os.open(new_stderr, os.O_WRONLY | os.O_TRUNC)  # 2: stderr
    else:
        code = task

    lines = code.split("\\n")
    rest, last = lines[:-1], lines[-1]
    try:
        try:
            last = compile(last, "<code>", "eval")
        except:
            unescaped = code.replace("\\n", "\n")
            compiled = compile(unescaped, "<code>", "exec")
            exec(compiled, globals_dict, locals_dict)
        else:
            if rest:
                joined = "\n".join(rest)
                compiled = compile(joined, "<code>", "exec")
                exec(compiled, globals_dict, locals_dict)
            value = eval(last, globals_dict, locals_dict)
            if value is not None:
                json = encode_value(value)
                stdout.write(json + RESULT)
    except Exception as exc:
        json = encode_exception(exc)
        stderr.write(json + RESULT)

    stdout.write(TASK)
    stdout.flush()
    stderr.write(TASK)
    stderr.flush()
