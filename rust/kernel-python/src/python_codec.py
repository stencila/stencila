import io
import json
import traceback

# fmt: off

try:
    import numpy
    from numpy import ndarray
    NUMPY_BOOL_TYPES = (numpy.bool_,)
    NUMPY_INT_TYPES = (numpy.byte, numpy.short, numpy.intc, numpy.int_, numpy.longlong)
    NUMPY_UINT_TYPES = (numpy.ubyte, numpy.ushort, numpy.uintc, numpy.uint, numpy.ulonglong)
    NUMPY_FLOAT_TYPES = (numpy.half, numpy.single, numpy.double, numpy.longdouble)
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
    """Encode a Python value to JSON"""
    node = value_to_node(value)
    return json.dumps(node, default=lambda value: repr(value))


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


def value_to_node(value):
    """Convert a Python value prior to a Stencila `Node`"""

    if isinstance(value, (bool, int, float, str)):
        return value

    if isinstance(value, ndarray):
        return ndarray_to_array(value)

    if isinstance(value, DataFrame):
        return dataframe_to_datatable(value)

    if MATPLOTLIB_IMPORTED and is_matplotlib(value):
        return matplotlib_to_image_object()

    return value


def derive_nodes(what, path, context):
    """Derive Stencila nodes from a Python value"""

    if what == "parameter":
        return [value_as_parameter(path, context)]
    else:
        raise Exception("Do not know how to derive '%s' from a Python value" % what)


def value_as_parameter(name, context):
    """Derive a Stencila `Parameter` from a Python value"""

    value = eval(name, context)
    return dict(type="Parameter", name=name, validator=value_as_validator(value))


def value_as_validator(value):
    """Derive a Stencila `Validator` from a Python value"""

    if isinstance(value, bool):
        return dict(type="BooleanValidator")

    if isinstance(value, int):
        return dict(type="IntegerValidator")

    if isinstance(value, float):
        return dict(type="NumberValidator")

    if isinstance(value, str):
        return dict(type="StringValidator")

    if isinstance(value, list):
        return dict(type="ArrayValidator")

    if isinstance(value, dict):
        return dict(type="ObjectValidator")

    if isinstance(value, dict):
        return dict(type="ObjectValidator")

    import enum

    if isinstance(value, type) and value.__base__ is enum.Enum:
        return dict(type="EnumValidator", values=list(value.__members__.keys()))

    if isinstance(value, enum.Enum):
        return dict(
            type="EnumValidator", values=list(value.__class__.__members__.keys())
        )

    import numpy

    if isinstance(value, numpy.ndarray):
        return ndarray_as_validator(value)

    raise Exception(
        "Do not know how to derive a validator from a value of type '%s'" % type(value)
    )


def ndarray_as_validator(value):
    """Derive a Stencila `Validator` from a numpy ndarray"""
    import numpy

    if value.dtype in NUMPY_BOOL_TYPES:
        validator = dict(type="BooleanValidator")
    elif value.dtype in NUMPY_INT_TYPES:
        validator = dict(type="IntegerValidator")
    elif value.dtype in NUMPY_UINT_TYPES:
        validator = dict(type="IntegerValidator", minimum=0)
    elif value.dtype in NUMPY_FLOAT_TYPES:
        validator = dict(type="NumberValidator")
    elif str(value.dtype) == "datetime64":
        validator = dict(type="TimestampValidator")
    elif str(value.dtype) == "timedelta64":
        validator = dict(type="DurationValidator")
    else:
        validator = None

    return dict(type="ArrayValidator", itemsValidator=validator)


def ndarray_to_array(array):
    """Convert a numpy `ndarray` to a Stencila `Array`"""
    return array.tolist()


def dataframe_to_datatable(df):
    """Convert a Pandas `Dataframe` to a Stencila `Datatable`"""
    import numpy

    columns = []
    for column_name in df.columns:
        column = df[column_name]
        values = ndarray_to_array(column)
        if column.dtype in NUMPY_BOOL_TYPES:
            values = [bool(row) for row in values]
        elif column.dtype in NUMPY_INT_TYPES or column.dtype in NUMPY_UINT_TYPES:
            values = [int(row) for row in values]
        elif column.dtype in NUMPY_FLOAT_TYPES:
            values = [float(row) for row in values]

        columns.append(
            dict(
                type="DatatableColumn",
                name=str(column_name),  # Ensure name is a string
                values=values,
                validator=ndarray_as_validator(column),
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


def matplotlib_to_image_object():
    """Convert the current matplotlib figure to a `ImageObject`"""
    from matplotlib import pyplot
    import base64

    image = io.BytesIO()
    pyplot.savefig(image, format="png")
    pyplot.close()
    src = "data:image/png;base64," + base64.encodebytes(image.getvalue()).decode()
    return dict(type="ImageObject", contentUrl=src)
