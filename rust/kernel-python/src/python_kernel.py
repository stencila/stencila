#!/usr/bin/env python3

import json
import sys

from python_codec import decode_value, encode_exception, encode_value

res_sep = u"\U0010ABBA\n"
trans_sep = u"\U0010ACDC\n"


def print(*objects, sep=" ", end="\n", file=sys.stdout, flush=False):
    if sep != " " or end != "\n" or file != sys.stdout or flush:
        return __builtins__.print(*objects, sep, end, file, flush)
    for object in objects:
        sys.stdout.write(encode_value(object) + res_sep)


context = {"print": print, "decode_value": decode_value}

for code in sys.stdin:
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
                out = json.dumps(value)
                sys.stdout.write(out + res_sep)
    except Exception as exc:
        sys.stderr.write(encode_exception(exc))

    sys.stdout.write(trans_sep)
    sys.stdout.flush()
    sys.stderr.write(trans_sep)
    sys.stderr.flush()

    code = ""
