import argparse
import ast
import base64
import json
import logging
import sys
import typing
from contextlib import redirect_stdout
from io import TextIOWrapper, BytesIO

import astor
from stencila.schema.types import Parameter, CodeChunk, Article, Entity, CodeExpression, ConstantSchema, EnumSchema, \
    BooleanSchema, NumberSchema, IntegerSchema, StringSchema, ArraySchema, TupleSchema, ImageObject, Datatable, \
    DatatableColumn
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

ExecutableCode = typing.Union[CodeChunk, CodeExpression]


class StdoutBuffer(TextIOWrapper):
    def write(self, string: typing.Union[bytes, str]) -> int:
        if isinstance(string, str):
            return super(StdoutBuffer, self).write(string)
        else:
            return super(StdoutBuffer, self).buffer.write(string)


class DocumentParser:
    """Parse an executable document (`Article`) and cache references to its parameters and code nodes."""

    parameters: typing.List[Parameter] = []
    code: typing.List[ExecutableCode] = []

    def parse(self, source: Article) -> None:
        # todo: this traverses the article twice. Make it less hard coded, maybe pass through a lookup table that maps
        # a found type to its destination
        self.handle_item(source, Parameter, self.parameters, None)
        self.handle_item(source, (CodeChunk, CodeExpression), self.code, {'language': 'python'})

    def handle_item(self, item: typing.Any,
                    search_type: typing.Union[typing.Type[Entity], typing.Iterable[typing.Type[Entity]]],
                    destination: typing.List[Entity],
                    attr_match: typing.Optional[typing.Dict[str, typing.Any]]) -> None:
        if isinstance(item, dict):
            self.traverse_dict(item, search_type, destination, attr_match)
        elif isinstance(item, list):
            self.traverse_list(item, search_type, destination, attr_match)
        elif isinstance(item, Entity):
            if isinstance(item, search_type):
                can_add = True
                if attr_match:
                    for k, v in attr_match.items():
                        if getattr(item, k, None) != v:
                            can_add = False
                            break
                if can_add:
                    logger.debug('Adding {}'.format(type(item)))
                    destination.append(item)
            self.traverse_dict(item.__dict__, search_type, destination, attr_match)

    def traverse_dict(self, d: dict,
                      search_type: typing.Union[typing.Type[Entity], typing.Iterable[typing.Type[Entity]]],
                      destination: typing.List[Entity],
                      attr_match: typing.Optional[typing.Dict[str, typing.Any]]) -> None:
        for child in d.values():
            self.handle_item(child, search_type, destination, attr_match)

    def traverse_list(self, l: typing.List,
                      search_type: typing.Union[typing.Type[Entity], typing.Iterable[typing.Type[Entity]]],
                      destination: typing.List[Entity],
                      attr_match: typing.Optional[typing.Dict[str, typing.Any]]) -> None:
        for child in l:
            self.handle_item(child, search_type, destination, attr_match)


class Executor:
    """Execute a list of code blocks, maintaining its own `globals` scope for this execution run."""

    globals: typing.Optional[typing.Dict[str, typing.Any]]

    def execute_code_chunk(self, chunk: CodeChunk, _locals: typing.Dict[str, typing.Any]) -> None:
        cc_outputs = []

        tree = ast.parse(chunk.text, 'exec')

        for statement in tree.body:
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

            with redirect_stdout(s):
                result = run_function(code_to_run, self.globals, _locals)

            if capture_result and result is not None:
                cc_outputs.append(self.decode_output(result))

            s.seek(0)
            std_out_output = s.buffer.read()

            if std_out_output:
                cc_outputs.append(std_out_output.decode('utf8'))

        chunk.outputs = cc_outputs

    def execute_code_expression(self, expression: CodeExpression, _locals: typing.Dict[str, typing.Any]) -> None:
        expression.output = self.decode_output(eval(expression.text, self.globals, _locals))

    def execute(self, code: typing.List[ExecutableCode], parameter_values: typing.Dict[str, typing.Any]) -> None:
        self.globals = {}

        _locals = parameter_values.copy()

        for c in code:
            if isinstance(c, CodeChunk):
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

    doc_parser = DocumentParser()
    doc_parser.parse(article)

    e = Executor()

    pp = ParameterParser(doc_parser.parameters)
    pp.parse_cli_args(cli_args)

    e.execute(doc_parser.code, pp.parameter_values)

    if args.output_file == '-':
        sys.stdout.write(to_json(article))
    else:
        with open(args.output_file, 'w') as o:
            o.write(to_json(article))


if __name__ == '__main__':
    logging.basicConfig(stream=sys.stdout, level=logging.DEBUG)
    execute_document(sys.argv[1:])
