#!/usr/bin/env python3

import json
import sys

res_sep = u"\U0010ABBA\n"
trans_sep = u"\U0010ACDC\n"

context = {}

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
                joined = '\n'.join(rest)
                exec(compile(joined, "<code>", "exec"))
            value = eval(last, globals(), context)
            if value is not None:
                out = json.dumps(value)
                sys.stdout.write(res_sep)
                sys.stdout.write(out)
    except Exception as exc:
        sys.stderr.write(str(exc))

    sys.stdout.write(trans_sep)
    sys.stdout.flush()
    sys.stderr.write(trans_sep)
    sys.stderr.flush()

    code = ""
