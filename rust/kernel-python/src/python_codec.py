import io
import json
import traceback

# fmt: off

try:
    from numpy import ndarray
except ImportError:
    class ndarray: pass

try:
    from pandas import DataFrame
except ImportError:
    class DataFrame: pass

try:
    import matplotlib
    import matplotlib.pyplot
    
    matplotlib.use('Agg')
    
    # Monkey patch pyplot.show to return itself to
    # indicates that an image should be returned as an output
    def show(*args, **kwargs):
        return matplotlib.pyplot.show
    matplotlib.pyplot.show = show
    
    MATPLOTLIB_IMPORTED = True
except ImportError:
    MATPLOTLIB_IMPORTED = False

# fmt: on


def decode_value(json_):
    """Decode JSON to a Python value"""
    return json.loads(json_)


def encode_value(value):
    """Decode a Python value to JSON"""
    converted = convert_value(value)
    return json.dumps(converted, default=lambda value: repr(value))


def convert_value(value):
    """Convert a value prior to encoding"""

    # Shortcut for primitive types
    if isinstance(value, (bool, int, float, str)):
        return value

    if isinstance(value, ndarray):
        return convert_ndarray(value)

    if isinstance(value, DataFrame):
        return convert_dataframe(value)

    if MATPLOTLIB_IMPORTED and is_matplotlib(value):
        return convert_matplotlib()

    return value


def convert_ndarray(array):
    """Convert a numpy `ndarray` to an `Array`"""
    return array.tolist()


def convert_dataframe(df):
    """Convert a Pandas `Dataframe` to a `Datatable`"""
    import numpy

    columns = []
    for column_name in df.columns:
        column = df[column_name]
        values = convert_ndarray(column)
        if column.dtype in (numpy.bool_, numpy.bool8):
            validator = dict(type="BooleanValidator")
            values = [bool(row) for row in values]
        elif column.dtype in (numpy.int8, numpy.int16, numpy.int32, numpy.int64):
            validator = dict(type="IntegerValidator")
            values = [int(row) for row in values]
        elif column.dtype in (numpy.float16, numpy.float32, numpy.float64):
            validator = dict(type="NumberValidator")
            values = [float(row) for row in values]
        elif column.dtype in (
            numpy.str_,
            numpy.unicode_,
        ):
            validator = dict(type="StringValidator")
        else:
            validator = None

        columns.append(
            dict(
                type="DatatableColumn",
                name=str(column_name),  # Ensure name is a string
                values=values,
                validator=dict(type="ArrayValidator", itemsValidator=validator),
            )
        )

    return dict(type="Datatable", columns=columns)


def is_matplotlib(value):
    """Is the value a matplotlib value or return of a matplotlib call?"""
    from matplotlib.artist import Artist
    from matplotlib.figure import Figure

    if (
        value == matplotlib.pyplot.show
        or isinstance(value, Artist)
        or isinstance(value, Figure)
    ):
        return True

    # This is somewhat crude but allows for calls that return lists of
    # matplotlib types not just single objects e.g. `pyplot.plot()`
    rep = repr(value)
    return rep.startswith("<matplotlib.") or rep.startswith("[<matplotlib.")


def convert_matplotlib():
    """Convert the current matplotlib figure to a `ImageObject`"""
    from matplotlib import pyplot
    import base64

    image = io.BytesIO()
    pyplot.savefig(image, format="png")
    pyplot.close()
    src = "data:image/png;base64," + base64.encodebytes(image.getvalue()).decode()
    return dict(type="ImageObject", contentUrl=src)


def encode_message(type, message, exception=None):
    """
    Encode a `CodeMessage`

    A stack trace is generated from the exception parameter and is used when capturing
    Python exceptions (to show where the error occurred) and for interrupts (to
    show where the code was interrupted).
    """
    # For now, until we have a `CodeMessage` type, gets encoded as a `CodeError`
    code_message = {"type": "CodeError", "errorType": type, "errorMessage": message}
    if exception and not isinstance(exception, KeyboardInterrupt):
        stack_trace = io.StringIO()
        traceback.print_exc(file=stack_trace)
        stack_trace = stack_trace.getvalue()
        # Remove the first three lines (the header and where we where in `python_kernel.py`)
        # and the last line which repeats the message
        stack_trace = "\n".join(stack_trace.split("\n")[3:-1])
        # Remove the "double" exception that can be caused by re-throwing the exception
        position = stack_trace.find("During handling of the above exception")
        if position:
            stack_trace = stack_trace[:position].strip()
        code_message["stackTrace"] = stack_trace
    return json.dumps(code_message)


def encode_exception(exc):
    """Encode an exception to a `CodeMessage`"""
    type = exc.__class__.__name__
    message = exc.message if hasattr(exc, "message") else str(exc)
    return encode_message(type, message, exc)
