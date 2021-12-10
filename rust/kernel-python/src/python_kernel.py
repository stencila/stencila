#!/usr/bin/env python3

import json
import os
import resource
from sys import stdin, stdout, stderr

from python_codec import decode_value, encode_exception, encode_value

READY = u"\U0010ACDC\n"
RESULT = u"\U0010CB40\n"
TRANS = u"\U0010ABBA\n"
FORK = u"\U0010DE70\n"


# Monkey patch `print` to encode individual objects (if no options used)
def print(*objects, sep=" ", end="\n", file=stdout, flush=False):
    if sep != " " or end != "\n" or file != stdout or flush:
        return __builtins__.print(*objects, sep, end, file, flush)
    for object in objects:
        json = encode_value(object)
        stdout.write(json + RESULT)


context = {"print": print, "decode_value": decode_value}

stdout.write(READY)
stdout.flush()
stderr.write(READY)
stderr.flush()

for code in stdin:
    if code.endswith(FORK):
        pid = os.fork()
        if pid > 0:
            # Parent process so just go to the next line
            continue

        # Child process, so...

        # Separate code and paths of FIFO pipes to replace stdout and stderr
        code = code[:-len(FORK)]
        pos = code.rfind("|")
        (code, pipes) = code[:pos], code[(pos + 1):]
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

    lines = code.split("\\n")
    rest, last = lines[:-1], lines[-1]
    try:
        try:
            last = compile(last, "<code>", "eval")
        except:
            unescaped = code.replace("\\n", "\n")
            exec(compile(unescaped, "<code>", "exec"))
        else:
            if rest:
                joined = "\n".join(rest)
                exec(compile(joined, "<code>", "exec"))
            value = eval(last, globals(), context)
            if value is not None:
                json = encode_value(value)
                stdout.write(json + RESULT)
    except Exception as exc:
        json = encode_exception(exc)
        stderr.write(json + RESULT)

    stdout.write(TRANS)
    stdout.flush()
    stderr.write(TRANS)
    stderr.flush()
