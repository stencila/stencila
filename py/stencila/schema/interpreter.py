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
    assigns: typing.List[str] = []
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


def annotation_name_to_schema(name: typing.Optional[str]) -> typing.Optional[SchemaTypes]:
    if not name:
        return None

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


class CodeChunkParser:
    imports: typing.List[str]
    declares: typing.List[typing.Union[Variable, Function]]
    assigns: typing.List[str]
    alters: typing.List[str]
    uses: typing.List[str]
    reads: typing.List[str]

    seen_vars: typing.List[str]

    def reset(self) -> None:
        self.imports = []
        self.declares = []
        self.assigns = []
        self.alters = []
        self.uses = []
        self.reads = []

        self.seen_vars = []

    def add_variable(self, name: str, type_annotation: typing.Optional[str]) -> None:
        if name in self.seen_vars:
            return
        v = Variable(name)
        v.schema = annotation_name_to_schema(type_annotation)
        self.seen_vars.append(name)
        self.declares.append(v)

    def add_name(self, name: str, target: typing.List) -> None:
        if name not in self.seen_vars and name not in target:
            self.seen_vars.append(name)
            target.append(name)

    def parse(self, chunk: CodeChunk) -> CodeChunkParseResult:
        self.reset()

        try:
            chunk_ast = ast.parse(chunk.text)
        except Exception as e:
            return CodeChunkParseResult(None, error=exception_to_code_error(e))

        # If this is True, then there should be a call to 'open' somewhere in the code, which means the parser should
        # try to find it. This is a basic check so there might not be one (like if the code did , but if 'open(' is NOT in
        # the string then there definitely ISN'T one
        search_for_open = 'open(' in chunk.text

        for statement in chunk_ast.body:
            self.parse_statement(statement)

        if search_for_open:
            self.find_file_reads(chunk_ast)

        return CodeChunkParseResult(chunk_ast, self.imports, self.assigns, self.declares, self.alters, self.uses,
                                    self.reads)

    def parse_statement(self, statement: typing.Union[ast.stmt, typing.List[ast.stmt]]) -> None:
        if isinstance(statement, list):
            for sub_statement in statement:
                self.parse_statement(sub_statement)
        elif isinstance(statement, ast.ImportFrom):
            self.parse_import(statement)
        elif isinstance(statement, (ast.Assign, ast.AnnAssign)):
            self.parse_assigns(statement)
        elif isinstance(statement, ast.BinOp):
            self.parse_bin_op(statement)
        elif isinstance(statement, ast.Call):
            self.parse_call(statement)
        elif isinstance(statement, ast.FunctionDef):
            self.parse_function_def(statement)
        elif isinstance(statement, ast.Dict):
            self.parse_dict(statement)
        elif isinstance(statement, ast.List):
            self.parse_statement(statement.elts)
        elif isinstance(statement, ast.Name):
            self.add_name(statement.id, self.uses)
        elif isinstance(statement, ast.Expr):
            self.parse_statement(statement.value)
        elif isinstance(statement, ast.AugAssign):
            self.parse_aug_assign(statement)
        elif isinstance(statement, (ast.If, ast.While)):
            self.parse_if_while(statement)
        elif isinstance(statement, ast.Compare):
            self.parse_compare(statement)
        elif isinstance(statement, ast.For):
            self.parse_for(statement)
        elif isinstance(statement, ast.Try):
            self.parse_try(statement)
        elif isinstance(statement, ast.ExceptHandler):
            self.parse_except_handler(statement)
        elif isinstance(statement, (ast.ClassDef, ast.Num, ast.Str, ast.Pass)):
            pass
        else:
            raise TypeError('Unrecognized statement: {}'.format(statement))

    def parse_import(self, statement: ast.ImportFrom) -> None:
        if statement.module not in self.imports:
            self.imports.append(statement.module)

    def recurse_attribute(self, attr: ast.Attribute) -> str:
        if isinstance(attr.value, ast.Attribute):
            return self.recurse_attribute(attr.value)
        if isinstance(attr.value, ast.Name):
            return attr.value.id

        raise TypeError('Unable to determine name of attribute {}'.format(attr.value))

    def parse_assigns(self, statement: typing.Union[ast.Assign, ast.AnnAssign]) -> None:
        if hasattr(statement, 'targets'):
            targets = statement.targets
        elif hasattr(statement, 'target'):
            targets = [statement.target]
        else:
            raise TypeError('{} has no target or targets'.format(statement))

        for target in targets:
            if isinstance(target, ast.Attribute):
                self.add_name(self.recurse_attribute(target), self.alters)
                continue

            if isinstance(target, ast.Name):
                if isinstance(statement, ast.AnnAssign):
                    annotation_name = statement.annotation.id if isinstance(statement.annotation, ast.Name) else None
                    self.add_variable(target.id, annotation_name)
                else:
                    self.add_name(target.id, self.assigns)

        if getattr(statement, 'value', None) is not None:
            self.parse_statement(statement.value)

    def parse_bin_op(self, statement: ast.BinOp) -> None:
        self.parse_statement(statement.left)
        self.parse_statement(statement.right)

    def parse_call(self, statement: ast.Call) -> None:
        if hasattr(statement, 'args'):
            self.parse_statement(statement.args)

        if hasattr(statement, 'keywords'):
            for kw in statement.keywords:
                self.parse_statement(kw.value)

    def parse_function_def(self, statement: ast.FunctionDef) -> None:
        if statement.name in self.seen_vars:
            return

        return_ann = statement.returns.id if isinstance(statement.returns, ast.Name) else None

        f = Function(statement.name, returns=annotation_name_to_schema(return_ann), parameters=[])

        for i, arg in enumerate(statement.args.args):
            p = Parameter(arg.arg)

            if arg.annotation:
                p.schema = annotation_name_to_schema(arg.annotation.id)

            default_index = len(statement.args.defaults) - len(statement.args.args) + i
            # Only the last len(statement.args.defaults) can have defaults (since they must come after non-default
            # parameters)
            if default_index >= 0:
                default = statement.args.defaults[default_index]

                if isinstance(default, ast.Num):
                    p.default = default.n
                elif isinstance(default, ast.Str):
                    p.default = default.s
                elif isinstance(default, ast.NameConstant):
                    # default of None/True/False
                    p.default = default.value
                else:
                    self.parse_statement(default)
                p.required = False
            else:
                p.required = True

            f.parameters.append(p)

        if statement.args.vararg:
            f.parameters.append(Parameter(statement.args.vararg.arg, required=False, repeats=True))

        if statement.args.kwarg:
            f.parameters.append(Parameter(statement.args.kwarg.arg, required=False, extends=True))

        self.seen_vars.append(f.name)
        self.declares.append(f)

    def parse_dict(self, statement: ast.Dict) -> None:
        for key in statement.keys:
            if isinstance(key, ast.Name):
                self.add_name(key.id, self.uses)
            else:
                self.parse_statement(key)
        for value in statement.values:
            if isinstance(value, ast.Name):
                self.add_name(value.id, self.uses)
            else:
                self.parse_statement(value)

    def parse_aug_assign(self, statement: ast.AugAssign) -> None:
        if isinstance(statement.target, ast.Name):
            self.add_name(statement.target.id, self.alters)
        else:
            self.parse_statement(statement.target)

        self.parse_statement(statement.value)

    def parse_if_while(self, statement: typing.Union[ast.If, ast.While]) -> None:
        self.parse_statement(statement.test)
        self.parse_statement(statement.body)
        self.parse_statement(statement.orelse)

    def parse_compare(self, statement: ast.Compare) -> None:
        self.parse_statement(statement.left)
        self.parse_statement(statement.comparators)

    def parse_for(self, statement: ast.For) -> None:
        if isinstance(statement.target, ast.Name):
            self.add_name(statement.target.id, self.assigns)
        self.parse_statement(statement.iter)
        self.parse_statement(statement.body)

    def parse_try(self, statement: ast.Try) -> None:
        self.parse_statement(statement.handlers)
        self.parse_statement(statement.body)
        self.parse_statement(statement.finalbody)
        self.parse_statement(statement.orelse)

    def parse_except_handler(self, statement: ast.ExceptHandler) -> None:
        self.parse_statement(statement.body)

    def find_file_reads(self, chunk_ast: ast.Module) -> None:
        for node in ast.walk(chunk_ast):
            if isinstance(node, ast.Call) and hasattr(node, 'func') and node.func.id == 'open':
                filename = parse_open_filename(node)

                if filename and filename not in self.reads:
                    self.reads.append(filename)


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
                        parser = CodeChunkParser()
                        cc_result = parser.parse(item)
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
