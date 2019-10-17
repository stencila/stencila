"""
This is for interpreting and executing an executable document in JSON format. It can be called from the command line:

python3 interpreter.py <infile> <outfile> [document parameters]

See README.md for more information.

Warning: `eval` and `exec` are used to run code in the document. Don't execute documents that you haven't verified
yourself.
"""

import argparse
import ast
import base64
import datetime
import json
import logging
import sys
import typing
from contextlib import redirect_stdout
from io import TextIOWrapper, BytesIO

import astor

from stencila.schema.code_parsing import CodeChunkExecution, set_code_error, CodeChunkParser
from stencila.schema.types import Parameter, CodeChunk, Article, Entity, CodeExpression, \
    ConstantSchema, EnumSchema, BooleanSchema, NumberSchema, IntegerSchema, StringSchema, \
    ArraySchema, TupleSchema, ImageObject, Datatable, DatatableColumn, Function
from stencila.schema.util import from_json, to_json

try:
    import matplotlib.figure
    import matplotlib.artist
    from matplotlib.cbook import silent_list

    # pylint: disable=C0103
    MPLFigure = matplotlib.figure.Figure
    # pylint: disable=C0103
    MPLArtist = matplotlib.artist.Artist
    MPL_AVAILABLE = True
except ImportError:
    # pylint: disable=R0903
    class MPLFigure:  # type: ignore
        """A fake MPLFigure to prevent ReferenceErrors later."""


    # pylint: disable=R0903
    class MLPArtist:  # type: ignore
        """A fake MLPArtist to prevent ReferenceErrors later."""


    # pylint: disable=R0903,C0103
    class silent_list:  # type: ignore
        """A fake silent_list to prevent ReferenceErrors later."""


    MPL_AVAILABLE = False

try:
    from pandas import DataFrame
    import numpy

    PANDAS_AVAILABLE = True
except ImportError:
    # pylint: disable=R0903
    class DataFrame:  # type: ignore
        """A fake DataFrame to prevent ReferenceErrors later."""


    PANDAS_AVAILABLE = False

LOGGER = logging.getLogger(__name__)
LOGGER.addHandler(logging.NullHandler())

CHUNK_PREVIEW_LENGTH = 20

ExecutableCode = typing.Union[CodeChunkExecution, CodeExpression]
StatementRuntime = typing.Tuple[bool, typing.Union[str], typing.Callable]


# Used to indicate that a particular output should not be added to outputs (c.f. a valid `None` value)
SKIP_OUTPUT_SEMAPHORE = object()


class CodeTimer:
    """Context handler for timing code, use inside a `with` statement."""
    _start_time: datetime.datetime
    duration: typing.Optional[datetime.timedelta] = None

    def __enter__(self):
        self.duration = None
        self._start_time = datetime.datetime.now()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.duration = datetime.datetime.now() - self._start_time

    @property
    def duration_ms(self) -> float:
        """Calculate the duration (time between `__enter__` and `__exit__` in microseconds."""
        if not self.duration:
            raise RuntimeError('CodeTimer has not yet been run')

        return self.duration.total_seconds() * 1000


class StdoutBuffer(TextIOWrapper):
    """Used for capturing output to stdout."""

    def write(self, string: typing.Union[bytes, str]) -> int:
        if isinstance(string, str):
            return super(StdoutBuffer, self).write(string)
        return super(StdoutBuffer, self).buffer.write(string)


class DocumentCompilationResult(typing.NamedTuple):
    """Stores references to Parameters and Code that is parsed from a Document."""

    parameters: typing.List[Parameter] = []
    code: typing.List[ExecutableCode] = []


class DocumentCompiler:
    """Parse an executable document (`Article`) and cache references to its parameters and code nodes."""

    TARGET_LANGUAGE = 'python'

    function_depth: int = 0

    def compile(self, source: Article) -> DocumentCompilationResult:
        """
        Compile an `Article` by walking it and finding `Parameter`s and `CodeChunk` or `CodeExpression` nodes.

        These are set on a `DocumentCompilationResult` which can be passed to the `Interpreter`.
        """
        self.function_depth = 0
        dcr = DocumentCompilationResult()

        self.handle_item(source, dcr)
        return dcr

    def handle_item(self, item: typing.Any, compilation_result: DocumentCompilationResult) -> None:
        """
        Parse any kind of dict, list of Entity.

        Returns nothing but updates the passed in `DocumentCompilationResult`.
        """
        if isinstance(item, dict):
            self.traverse_dict(item, compilation_result)
        elif isinstance(item, list):
            self.traverse_list(item, compilation_result)
        elif isinstance(item, Entity):
            if isinstance(item, (CodeChunk, CodeExpression)):
                if item.programmingLanguage == self.TARGET_LANGUAGE:  # Only add Python code
                    self.handle_code(item, compilation_result)

            elif isinstance(item, Parameter) and self.function_depth == 0:
                compilation_result.parameters.append(item)
                LOGGER.debug('Adding %s', type(item))

            if isinstance(item, Function):
                # This prevents treating a `Parameter` found inside `Function` as a document parameter
                self.function_depth += 1

            self.traverse_dict(item.__dict__, compilation_result)

            if isinstance(item, Function):
                self.function_depth -= 1

    @staticmethod
    def set_code_imports(code: CodeChunk, imports: typing.List[str]) -> None:
        """
        Set a list of imports (strings) onto the `code` CodeChunk.

        `imports` will be combined with existing imports on the CodeChunk with duplicates removed, unless the existing
        imports has an empty string semaphore which indicates no new imports should be added.
        """
        if code.imports is None:
            code.imports = imports
            return

        if '' in code.imports:
            return

        for imp in imports:
            if imp not in code.imports:
                code.imports.append(imp)

    def handle_code(self, item: typing.Union[CodeChunk, CodeExpression],
                    compilation_result: DocumentCompilationResult) -> None:
        """Parse a CodeChunk or CodeExpression and add it to `compilation_result.code` list."""
        if isinstance(item, CodeChunk):
            parser = CodeChunkParser()
            cc_result = parser.parse(item)
            self.set_code_imports(item, cc_result.imports)
            item.declares = cc_result.declares
            item.assigns = cc_result.assigns
            item.alters = cc_result.alters
            item.uses = cc_result.uses
            item.reads = cc_result.reads

            if cc_result.error:
                set_code_error(item, cc_result.error)

            code_to_add = CodeChunkExecution(item, cc_result)
        else:
            try:
                ast.parse(item.text)
            except SyntaxError as exc:  # Probably will only be this
                set_code_error(item, exc)
            code_to_add = item
        compilation_result.code.append(code_to_add)
        LOGGER.debug('Adding %s', type(item))

    def traverse_dict(self, _dict: dict, compilation_result: DocumentCompilationResult) -> None:
        """Traverse into each item of a dict."""
        for child in _dict.values():
            self.handle_item(child, compilation_result)

    def traverse_list(self, _list: typing.List, compilation_result: DocumentCompilationResult) -> None:
        """Traverse into each element in a list."""
        for child in _list:
            self.handle_item(child, compilation_result)


class Interpreter:
    """Execute a list of code blocks, maintaining its own `globals` scope for this execution run."""

    globals: typing.Dict[str, typing.Any]
    locals: typing.Dict[str, typing.Any]

    def __init__(self) -> None:
        self.globals = {}
        self.locals = {}

    def execute_code_chunk(self, chunk_execution: CodeChunkExecution, _locals: typing.Dict[str, typing.Any]) -> None:
        """Execute a `CodeChunk` that has been parsed and stored in a `CodeChunkExecution`."""
        chunk, parse_result = chunk_execution

        if parse_result.chunk_ast is None:
            LOGGER.info('Not executing CodeChunk without AST: %s', chunk.text[:CHUNK_PREVIEW_LENGTH])
            return

        cc_outputs: typing.List[typing.Any] = []

        duration = 0.0

        for statement in parse_result.chunk_ast.body:
            duration, error_occurred = self.execute_statement(statement, chunk, _locals, cc_outputs, duration)

            if error_occurred:
                break  # stop executing the rest of the statements in the chunk after capturing the outputs

        chunk.duration = duration

        if MPL_AVAILABLE:
            # Because of the way matplotlib might progressively build an image, only keep the last MPL that was
            # generated for a specific code chunk
            mpl_output_indexes = [i for i, output in enumerate(cc_outputs) if self.value_is_mpl(output)]

            if mpl_output_indexes:
                for i in reversed(mpl_output_indexes[:-1]):
                    # remove all but the last mpl
                    cc_outputs.pop(i)

                new_last_index = mpl_output_indexes[-1] - (len(mpl_output_indexes) - 1)  # output will have shifted back

                cc_outputs[new_last_index] = self.decode_mpl()

        chunk.outputs = cc_outputs

    def execute_statement(self, statement: ast.stmt, chunk: CodeChunk, _locals: typing.Dict[str, typing.Any],
                          cc_outputs: typing.List[str], duration: float) -> typing.Tuple[float, bool]:
        """
        Execute a single AST statement.

        The statement will be executed with `eval` or `exec` depending on its type (see `parse_statement_runtime`).
        """
        error_occurred = False

        capture_result, code_to_run, run_function = self.parse_statement_runtime(statement)
        stdout = StdoutBuffer(BytesIO(), sys.stdout.encoding)
        result = None

        with redirect_stdout(stdout):
            try:
                with CodeTimer() as code_timer:
                    result = run_function(code_to_run, self.globals, _locals)
                duration += code_timer.duration_ms
            # pylint: disable=W0703  # we really don't know what Exception some exec'd code might raise.
            except Exception as exc:
                error_occurred = True
                set_code_error(chunk, exc)

        if capture_result and result is not None:
            self.add_output(cc_outputs, result)

        stdout.seek(0)
        std_out_output = stdout.buffer.read()
        if std_out_output:
            cc_outputs.append(std_out_output.decode('utf8'))

        # could save a variable by returning `duration` as `None` if an error occurs but I think that breaks readability
        return duration, error_occurred

    def add_output(self, cc_outputs: typing.List[typing.Any], result: typing.Any) -> None:
        """
        Add an output to cc_outputs.

        Should be inside `execute_statement` as it's only used there, but pylint complains about too many local
        variables.
        """
        decoded = self.decode_output(result)
        if decoded != SKIP_OUTPUT_SEMAPHORE:
            cc_outputs.append(decoded)

    @staticmethod
    def parse_statement_runtime(statement: ast.stmt) -> StatementRuntime:
        """
        Determine the information to execute a statement.

        Returns the function with which to execute a statement (`eval` or `exec`), whether its output should be captured
        or not, and the compiled code to execute.

        See the other comments below on how/why these functions are chosen.
        """

        if isinstance(statement, ast.Expr):
            # An expression is something that we want to capture the result of - this could be something like
            # `a + 3` or a function call (not an assignment). It must be executed with `eval`. Since `eval` can't
            # execute compiled code, `astor` is used to convert it back to source code.
            capture_result = True
            run_function = eval
            code_to_run = astor.to_source(statement)
        else:
            # We don't care about the result of this call (it could just be an assignment, update or even function
            # definition) so it can be executed with `exec`.
            capture_result = False
            run_function = exec
            mod = ast.Module()
            mod.body = [statement]
            code_to_run = compile(mod, '<ast>', 'exec')
        return capture_result, code_to_run, run_function

    def execute_code_expression(self, expression: CodeExpression, _locals: typing.Dict[str, typing.Any]) -> None:
        """eval `CodeExpression.text`, and get the result. Catch any exception the occurs."""
        try:
            # pylint: disable=W0123  # Disable warning that eval is being used.
            expression.output = self.decode_output(eval(expression.text, self.globals, _locals))
        # pylint: disable=W0703  # we really don't know what Exception some eval'd code might raise.
        except Exception as exc:
            set_code_error(expression, exc)

    def execute(self, code: typing.List[ExecutableCode], parameter_values: typing.Dict[str, typing.Any]) -> None:
        """
        For each piece of code (`CodeChunk` or `CodeExpression`) execute it.

        The `parameter_values` are used as the locals values for all executions or evals throughout the code in this
        `Article`.
        """
        _locals = self.locals
        _locals.update(parameter_values)
        #_locals = parameter_values.copy()

        for piece in code:
            if isinstance(piece, CodeChunkExecution):
                self.execute_code_chunk(piece, _locals)
            elif isinstance(piece, CodeExpression):
                self.execute_code_expression(piece, _locals)
            else:
                raise TypeError('Unknown Code node type found: {}'.format(piece))

    @staticmethod
    def value_is_mpl(value: typing.Any) -> bool:
        """Basic type checking to determine if a variable is a MatPlotLib figure."""
        if not MPL_AVAILABLE:
            return False

        return isinstance(value, (MPLFigure, MPLArtist)) or (isinstance(value, list) and
                                                             len(value) == 1 and
                                                             isinstance(value[0], MPLArtist))

    @staticmethod
    def decode_mpl() -> ImageObject:
        """
        Decode a matplotlib `MPLFigure` or `MPLArtist` into an `ImageObject`.

        The `matplotlib.pyplot.savefig` function just saves the current MPL figure that's in the context.
        """
        image = BytesIO()
        matplotlib.pyplot.savefig(image, format='png')
        src = 'data:image/png;base64,' + base64.encodebytes(image.getvalue()).decode()
        return ImageObject(src)

    @staticmethod
    def decode_dataframe(data_frame: DataFrame) -> Datatable:
        """Decode a pandas `DataFrame` into a `Datatable`"""
        columns = []

        for column_name in data_frame.columns:
            column = data_frame[column_name]
            values = column.tolist()
            if column.dtype in (numpy.bool_, numpy.bool8):
                schema = BooleanSchema()
                values = [bool(row) for row in values]
            elif column.dtype in (numpy.int8, numpy.int16, numpy.int32, numpy.int64):
                schema = IntegerSchema()
                values = [int(row) for row in values]
            elif column.dtype in (numpy.float16, numpy.float32, numpy.float64):
                schema = NumberSchema()
                values = [float(row) for row in values]
            elif column.dtype in (numpy.str_, numpy.unicode_,):
                schema = StringSchema()
            else:
                schema = None

            columns.append(
                DatatableColumn(column_name, values, schema=ArraySchema(items=schema))
            )

        return Datatable(columns)

    def decode_output(self, output: typing.Any) -> typing.Any:
        """
        Check if the output is convertible from a special data type to a Stencila type.

        If not, just return the original object."""

        if MPL_AVAILABLE:
            if isinstance(output, silent_list):
                return SKIP_OUTPUT_SEMAPHORE

        if isinstance(output, list):
            return [self.decode_output(item) for item in output]

        if isinstance(output, tuple):
            return tuple(self.decode_output(item) for item in output)

        if isinstance(output, DataFrame):
            return self.decode_dataframe(output)

        if PANDAS_AVAILABLE:
            if isinstance(output, numpy.ndarray):
                return output.tolist()

        return output


class ParameterParser:
    """
    Parse parameters that the document requires, from the command line.

    The `ArgumentParser` will fail if any required parameters are not passed in.
    """
    parameters: typing.Dict[str, Parameter]
    parameter_values: typing.Dict[str, typing.Any]

    def __init__(self, parameters: typing.List[Parameter]) -> None:
        self.parameters = {parameter.name: parameter for parameter in parameters}
        self.parameter_values = {}

    def parse_cli_args(self, cli_args: typing.List[str]) -> None:
        """
        Parse a list of `cli_args` (strings) into the `parameter_values` dict.

        This will fill in defaults and convert types based on the Parameter's schema. If a required argument is missing
        then the `ArgumentParser` class will cause the process to exit.
        """
        if not self.parameters:
            LOGGER.debug('No parameters passed to parse_cli_args')
            return

        param_parser = argparse.ArgumentParser(description='Parse Parameters')

        for param in self.parameters.values():
            if not isinstance(param.schema, ConstantSchema):
                param_parser.add_argument('--' + param.name, dest=param.name, required=param.required)

        args, _ = param_parser.parse_known_args(cli_args)

        LOGGER.debug('Parsed command line args: %s', args)

        for param_name in self.parameters:
            cli_value = getattr(args, param_name, None)
            if not cli_value and self.parameters[param_name].default:
                self.parameter_values[param_name] = self.parameters[param_name].default
            else:
                self.parameter_values[param_name] = self.deserialize_parameter(self.parameters[param_name], cli_value)

    @staticmethod
    def deserialize_parameter(parameter: Parameter, value: typing.Any) -> typing.Any:
        """Convert a value (usually a string) into the type specified by the parameter's schema."""
        if isinstance(parameter.schema, ConstantSchema):
            # A ConstantSchema doesn't take a value it stores its own value.
            return parameter.schema.value

        if isinstance(parameter.schema, EnumSchema):
            if value not in parameter.schema.values:
                raise TypeError('{} not found in enum values for {}'.format(value, parameter.name))
            return value

        if isinstance(parameter.schema, BooleanSchema):
            return value.lower() in ('true', 'yes', '1', 't')

        conversion_function: typing.Optional[typing.Callable] = None

        if isinstance(parameter.schema, IntegerSchema):
            conversion_function = int
        elif isinstance(parameter.schema, NumberSchema):
            conversion_function = float  # If there are problems with inaccuracy consider using Decimal instead
        elif isinstance(parameter.schema, (ArraySchema, TupleSchema)):
            conversion_function = json.loads

        return conversion_function(value) if conversion_function else value


def read_input(input_file: str) -> Article:
    """Read an input file from a "path", decode it as JSON and return the result.

    If the path is "-" then read from stdin. If the decoded object is not an `Article` then raise a `TypeError`.
    """
    if input_file == '-':
        j = sys.stdin.read()
    else:
        with open(input_file) as i:
            j = i.read()
    article = from_json(j)
    if not isinstance(article, Article):
        raise TypeError('Decoded JSON was not an Article')
    return article


def write_output(output_file: str, article: Article) -> None:
    """
    Convert an article to JSON and write to a "path".

    If the path is "-" then output to stdout.
    """
    if output_file == '-':
        sys.stdout.write(to_json(article))
    else:
        with open(output_file, 'w') as file:
            file.write(to_json(article))


def execute_compilation(compilation_result: DocumentCompilationResult, parameter_flags: typing.List[str]) -> None:
    """Compile an `Article`, and interpret it with the given parameters (in a format that would be read from CLI)."""
    param_parser = ParameterParser(compilation_result.parameters)
    param_parser.parse_cli_args(parameter_flags)
    Interpreter().execute(compilation_result.code, param_parser.parameter_values)


def compile_article(article: Article) -> DocumentCompilationResult:
    """Compile an `Article`, without executing it afterward."""
    return DocumentCompiler().compile(article)


def execute_from_cli(cli_args: typing.List[str], compile_only=False) -> None:
    """Read arguments from the CLI then parse an `Article` in JSON format, execute it, and output it somewhere."""
    cli_parser = argparse.ArgumentParser()
    cli_parser.add_argument('input_file', help='File to read from or "-" to read from stdin', nargs='?', default='-')
    cli_parser.add_argument('output_file', help='File to write to or "-" to write to stdout', nargs='?', default='-')

    args, _ = cli_parser.parse_known_args(cli_args)

    input_file = args.input_file
    output_file = args.output_file

    article = read_input(input_file)

    compilation_result = compile_article(article)

    if not compile_only:
        execute_compilation(compilation_result, cli_args)

    write_output(output_file, article)


if __name__ == '__main__':
    logging.basicConfig(stream=sys.stdout, level=logging.DEBUG)
    execute_from_cli(sys.argv[1:])
