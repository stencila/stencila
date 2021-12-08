#!/usr/bin/env python3

import json
from sys import stdin, stdout, stderr

from python_codec import decode_value, encode_exception, encode_value

res_sep = u"\U0010ABBA\n"
trans_sep = u"\U0010ACDC\n"


def print(*objects, sep=" ", end="\n", file=stdout, flush=False):
    if sep != " " or end != "\n" or file != stdout or flush:
        return __builtins__.print(*objects, sep, end, file, flush)
    for object in objects:
        json = encode_value(object)
        stdout.write(json + res_sep)


context = {"print": print, "decode_value": decode_value}

for code in stdin:
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
                stdout.write(json + res_sep)
    except Exception as exc:
        json = encode_exception(exc)
        stderr.write(json + res_sep)

    stdout.write(trans_sep)
    stdout.flush()
    stderr.write(trans_sep)
    stderr.flush()

    code = ""
