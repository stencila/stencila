import io
import json
import traceback


def decode_value(json_):
    return json.loads(json_)


def encode_value(value):
    return json.dumps(value)


def encode_exception(exc):
    code_error = {"type": "CodeError", "errorType": exc.__class__.__name__}

    if hasattr(exc, "message"):
        code_error["errorMessage"] = exc.message
    else:
        code_error["errorMessage"] = str(exc)

    stack_trace = io.StringIO()
    traceback.print_exc(file=stack_trace)
    code_error["stackTrace"] = stack_trace.getvalue()

    return json.dumps(code_error)
