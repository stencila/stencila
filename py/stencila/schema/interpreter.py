import argparse
import ast
import base64
import datetime
import json
import logging
import sys
import traceback
import typing
from contextlib import redirect_stdout
from io import TextIOWrapper, BytesIO

import astor
from stencila.schema.types import Parameter, CodeChunk, Article, Entity, CodeExpression, ConstantSchema, EnumSchema, \
    BooleanSchema, NumberSchema, IntegerSchema, StringSchema, ArraySchema, TupleSchema, ImageObject, Datatable, \
    DatatableColumn, SchemaTypes, SoftwareSourceCode, Function, Variable, CodeError
from stencila.schema.util import from_json, to_json

try:
    import matplotlib.figure
    import matplotlib.artist

    MPLFigure = matplotlib.figure.Figure
    MPLArtist = matplotlib.artist.Artist
    mpl_available = True
except ImportError:
    class MPLFigure:
        pass


    class MLPArtist:
        pass


    mpl_available = False

try:
    from pandas import DataFrame
    import numpy

    pandas_available = True
except ImportError:
    class DataFrame:
        pass


    pandas_available = False

logger = logging.getLogger(__name__)
logger.addHandler(logging.NullHandler())

CHUNK_PREVIEW_LENGTH = 20


class CodeChunkParseResult(typing.NamedTuple):
    chunk_ast: typing.Optional[ast.Module] = None
    imports: typing.List[typing.Union[str, SoftwareSourceCode]] = []
    assigns: typing.List[typing.Union[Variable]] = []
    declares: typing.List[typing.Union[Function, Variable]] = []
    alters: typing.List[str] = []
    uses: typing.List[str] = []
    reads: typing.List[str] = []
    error: typing.Optional[CodeError] = None


class CodeChunkExecution(typing.NamedTuple):
    code_chunk: CodeChunk
    parse_result: CodeChunkParseResult


ExecutableCode = typing.Union[CodeChunkExecution, CodeExpression]


class CodeTimer:
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
        if not self.duration:
            raise RuntimeError('CodeTimer has not yet been run')

        return self.duration.total_seconds() * 1000


class StdoutBuffer(TextIOWrapper):
    def write(self, string: typing.Union[bytes, str]) -> int:
        if isinstance(string, str):
            return super(StdoutBuffer, self).write(string)
        else:
            return super(StdoutBuffer, self).buffer.write(string)


class DocumentCompilationResult:
    parameters: typing.List[Parameter] = []
    code: typing.List[ExecutableCode] = []
    assigns: typing.List[typing.Union[Function, Variable]] = []
    imports: typing.List[str] = []


def annotation_name_to_schema(name: str) -> typing.Optional[SchemaTypes]:
    if name == 'bool':
        return BooleanSchema()
    elif name == 'str':
        return StringSchema()
    elif name == 'int':
        return IntegerSchema()
    elif name == 'float':
        return NumberSchema()
    elif 'list' in name.lower():
        return ArraySchema()
    elif 'tuple' in name.lower():
        return TupleSchema()

    return None


def mode_is_read(mode: str) -> bool:
    return 'r' in mode or '+' in mode


def parse_open_filename(open_call: ast.Call) -> typing.Optional[str]:
    # if not hasattr(open_call, 'args') or len(open_call.args) == 0:
    #    return None
    filename = None

    if hasattr(open_call, 'args'):
        if len(open_call.args) >= 1:
            if not isinstance(open_call.args[0], ast.Str):
                return None
            filename = open_call.args[0].s

            if len(open_call.args) >= 2:
                if not isinstance(open_call.args[1], ast.Str):
                    return None

                if not mode_is_read(open_call.args[1].s):
                    return None

    if hasattr(open_call, 'keywords'):
        for kw in open_call.keywords:
            if not isinstance(kw.value, ast.Str):
                continue

            if kw.arg == 'file':
                filename = kw.value.s

            if kw.arg == 'mode':
                if not mode_is_read(kw.value.s):
                    return None

    return filename


def exception_to_code_error(e: Exception) -> CodeError:
    return CodeError(type(e).__name__, message=str(e), trace=traceback.format_exc())


def set_code_error(code: typing.Union[CodeChunk, CodeExpression], e: typing.Union[Exception, CodeError]) -> None:
    if code.errors is None:
        code.errors = []

    if isinstance(e, Exception):
        e = exception_to_code_error(e)

    code.errors.append(e)


def parse_code_chunk(chunk: CodeChunk) -> CodeChunkParseResult:
    try:
        chunk_ast = ast.parse(chunk.text)
    except Exception as e:
        return CodeChunkParseResult(None, error=exception_to_code_error(e))

    imports: typing.List[str] = []
    assigns: typing.Set[Variable] = set()
    declares: typing.Set[typing.Union[Function, Variable]] = set()
    alters: typing.Set[str] = set()
    uses: typing.Set[str] = set()
    reads: typing.Set[str] = set()
    seen_vars: typing.Set[str] = set()

    # If this is True, then there should be a call to 'open' somewhere in the code, which means the parser should
    # try to find it. This is a basic check so there might not be one (like if the code did , but if 'open(' is NOT in
    # the string then there definitely ISN'T one
    search_for_open = 'open(' in chunk.text

    for statement in chunk_ast.body:
        if isinstance(statement, ast.ImportFrom):
            if statement.module not in imports:
                imports.append(statement.module)
        elif isinstance(statement, ast.Import):
            for module_name in statement.names:
                if module_name.name not in imports:
                    imports.append(module_name.name)
        elif isinstance(statement, ast.FunctionDef):
            f = Function(statement.name)
            f.parameters = []

            for i, arg in enumerate(statement.args.args):
                p = Parameter(arg.arg)

                if arg.annotation:
                    p.schema = annotation_name_to_schema(arg.annotation.id)

                default_index = len(statement.args.defaults) - len(statement.args.args) + i
                # Only the last len(statement.args.defaults) can have defaults (since they must come after non-default
                # parameters)
                if default_index >= 0:
                    p.default = statement.args.defaults[default_index].value
                    p.required = False
                else:
                    p.required = True

                f.parameters.append(p)

            declares.append(f)
        elif isinstance(statement, (ast.Assign, ast.AnnAssign)):
            if hasattr(statement, 'targets'):
                targets = statement.targets
            elif hasattr(statement, 'target'):
                targets = [statement.target]
            else:
                raise TypeError('statement has no target or targets')

            for target in targets:
                is_alters = False
                if hasattr(target, 'id'):
                    # simple variable set/declaration
                    target_name = target.id
                elif hasattr(target, 'value'):
                    target_name = target.value.id
                    is_alters = True
                else:
                    raise ValueError("Don't know how to handle this")

                if target_name not in seen_vars:
                    if is_alters:
                        alters.add(target.value.id)
                        continue

                    v = Variable(target_name)

                    if hasattr(statement, 'annotation'):
                        # assignment with Type Annotation
                        v.schema = annotation_name_to_schema(statement.annotation.id)
                        declares.add(v)
                    else:
                        assigns.add(v)
                        seen_vars.add(target_name)
                        seen_vars.add(target_name)
        elif isinstance(statement, ast.Expr) and isinstance(statement.value, ast.Call):
            if hasattr(statement.value, 'args'):
                for arg in statement.value.args:
                    if isinstance(arg, ast.Name):
                        uses.add(arg.id)

    if search_for_open:
        for node in ast.walk(chunk_ast):
            if isinstance(node, ast.Call) and hasattr(node, 'func') and node.func.id == 'open':
                filename = parse_open_filename(node)

                if filename:
                    reads.add(filename)

    return CodeChunkParseResult(chunk_ast, imports, list(assigns), list(declares), list(alters), list(uses),
                                list(reads))


class DocumentCompiler:
    """Parse an executable document (`Article`) and cache references to its parameters and code nodes."""

    TARGET_LANGUAGE = 'python'

    function_depth: int = 0

    def compile(self, source: Article) -> DocumentCompilationResult:
        # todo: this traverses the article twice. Make it less hard coded, maybe pass through a lookup table that maps
        # a found type to its destination
        self.function_depth = 0
        dcr = DocumentCompilationResult()

        self.handle_item(source, dcr)
        return dcr

    def handle_item(self, item: typing.Any, compilation_result: DocumentCompilationResult) -> None:
        if isinstance(item, dict):
            self.traverse_dict(item, compilation_result)
        elif isinstance(item, list):
            self.traverse_list(item, compilation_result)
        elif isinstance(item, Entity):
            if isinstance(item, (CodeChunk, CodeExpression)):
                if item.language == self.TARGET_LANGUAGE:  # Only add Python code

                    if isinstance(item, CodeChunk):
                        cc_result = parse_code_chunk(item)
                        item.imports = cc_result.imports
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
                        except Exception as e:
                            set_code_error(item, e)
                        code_to_add = item

                    compilation_result.code.append(code_to_add)
                    logger.debug('Adding {}'.format(type(item)))

            elif isinstance(item, Parameter) and self.function_depth == 0:
                compilation_result.parameters.append(item)
                logger.debug('Adding {}'.format(type(item)))

            if isinstance(item, Function):
                self.function_depth += 1

            self.traverse_dict(item.__dict__, compilation_result)

            if isinstance(item, Function):
                self.function_depth -= 1

    def traverse_dict(self, d: dict, compilation_result: DocumentCompilationResult) -> None:
        for child in d.values():
            self.handle_item(child, compilation_result)

    def traverse_list(self, l: typing.List, compilation_result: DocumentCompilationResult) -> None:
        for child in l:
            self.handle_item(child, compilation_result)


class Interpreter:
    """Execute a list of code blocks, maintaining its own `globals` scope for this execution run."""

    globals: typing.Optional[typing.Dict[str, typing.Any]]

    def execute_code_chunk(self, chunk_execution: CodeChunkExecution, _locals: typing.Dict[str, typing.Any]) -> None:
        chunk, parse_result = chunk_execution

        if parse_result.chunk_ast is None:
            logger.info('Not executing CodeChunk without AST: {}'.format(chunk.text[:CHUNK_PREVIEW_LENGTH]))
            return

        cc_outputs = []

        for statement in parse_result.chunk_ast.body:
            capture_result = False

            if isinstance(statement, ast.Expr):
                capture_result = True
                run_function = eval
                code_to_run = astor.to_source(statement)
            else:
                run_function = exec
                m = ast.Module()
                m.body = [statement]
                code_to_run = compile(m, '<ast>', 'exec')

            s = StdoutBuffer(BytesIO(), sys.stdout.encoding)

            result = None

            with redirect_stdout(s):
                try:
                    with CodeTimer() as ct:
                        result = run_function(code_to_run, self.globals, _locals)
                    chunk.duration = ct.duration_ms
                except Exception as e:
                    set_code_error(chunk, e)

            if capture_result and result is not None:
                cc_outputs.append(self.decode_output(result))

            s.seek(0)
            std_out_output = s.buffer.read()

            if std_out_output:
                cc_outputs.append(std_out_output.decode('utf8'))

        chunk.outputs = cc_outputs

    def execute_code_expression(self, expression: CodeExpression, _locals: typing.Dict[str, typing.Any]) -> None:
        try:
            expression.output = self.decode_output(eval(expression.text, self.globals, _locals))
        except Exception as e:
            set_code_error(expression, e)

    def execute(self, code: typing.List[ExecutableCode], parameter_values: typing.Dict[str, typing.Any]) -> None:
        self.globals = {}

        _locals = parameter_values.copy()

        for c in code:
            if isinstance(c, CodeChunkExecution):
                self.execute_code_chunk(c, _locals)
            elif isinstance(c, CodeExpression):
                self.execute_code_expression(c, _locals)
            else:
                raise TypeError('Unknown Code node type found: {}'.format(c))

    @staticmethod
    def value_is_mpl(value: typing.Any) -> bool:
        if not mpl_available:
            return False

        return isinstance(value, (MPLFigure, MPLArtist)) or (
                isinstance(value, list) and len(value) == 1 and isinstance(value[0], MPLArtist))

    @staticmethod
    def decode_mpl() -> ImageObject:
        image = BytesIO()
        matplotlib.pyplot.savefig(image, format='png')
        src = 'data:image/png;base64,' + base64.encodebytes(image.getvalue()).decode()
        return ImageObject(src)

    @staticmethod
    def decode_dataframe(df: DataFrame) -> Datatable:
        columns = []

        for column_name in df.columns:
            column = df[column_name]
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
        if self.value_is_mpl(output):
            return self.decode_mpl()

        if isinstance(output, DataFrame):
            return self.decode_dataframe(output)

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
        if not self.parameters:
            logger.debug('No parameters passed to parse_cli_args')
            return

        param_parser = argparse.ArgumentParser(description='Parse Parameters')

        for param in self.parameters.values():
            if not isinstance(param.schema, ConstantSchema):
                param_parser.add_argument('--' + param.name, dest=param.name,
                                          required=self.get_parameter_default(param) is None)

        args, _ = param_parser.parse_known_args(cli_args)

        logger.debug('Parsed command line args: {}'.format(args))

        for param_name in self.parameters:
            cli_value = getattr(args, param_name, None)
            if not cli_value and self.parameters[param_name].default:
                self.parameter_values[param_name] = self.parameters[param_name].default
            else:
                self.parameter_values[param_name] = self.deserialize_parameter(self.parameters[param_name], cli_value)

    @staticmethod
    def get_parameter_default(parameter: Parameter) -> typing.Any:
        if isinstance(parameter.schema, ConstantSchema):
            return parameter.schema.value or parameter.default

        return parameter.default

    @staticmethod
    def deserialize_parameter(parameter: Parameter, value: typing.Any) -> typing.Any:
        # Lots of TODOs here, might not care as passing this off to encoda soon

        if isinstance(parameter.schema, ConstantSchema):
            return parameter.schema.value

        if isinstance(parameter.schema, EnumSchema):
            if value not in parameter.schema.values:
                raise TypeError('{} not found in enum values for {}'.format(value, parameter.name))
            return value

        if isinstance(parameter.schema, BooleanSchema):
            return value.lower() in ('true', 'yes', '1' 't')

        if isinstance(parameter.schema, IntegerSchema):
            return int(value)

        if isinstance(parameter.schema, NumberSchema):
            return float(value)  # TODO should be a decimal?

        if isinstance(parameter.schema, StringSchema):
            return value

        if isinstance(parameter.schema, ArraySchema):
            return json.loads(value)

        if isinstance(parameter.schema, TupleSchema):
            return json.loads(value)


def execute_document(cli_args: typing.List[str]):
    cli_parser = argparse.ArgumentParser()
    cli_parser.add_argument('input_file', help='File to read from or "-" to read from stdin', nargs='?', default='-')
    cli_parser.add_argument('output_file', help='File to write to or "-" to write to stdout', nargs='?', default='-')

    args, _ = cli_parser.parse_known_args(cli_args)

    if args.input_file == '-':
        j = sys.stdin.read()
    else:
        with open(args.input_file) as i:
            j = i.read()

    article = from_json(j)

    if not isinstance(article, Article):
        raise TypeError('Decoded JSON was not an Article')

    compiler = DocumentCompiler()
    dcr = compiler.compile(article)

    pp = ParameterParser(dcr.parameters)
    pp.parse_cli_args(cli_args)

    i = Interpreter()
    i.execute(dcr.code, pp.parameter_values)

    if args.output_file == '-':
        sys.stdout.write(to_json(article))
    else:
        with open(args.output_file, 'w') as o:
            o.write(to_json(article))


if __name__ == '__main__':
    logging.basicConfig(stream=sys.stdout, level=logging.DEBUG)
    execute_document(sys.argv[1:])
