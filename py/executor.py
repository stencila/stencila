import argparse
import json
import logging
import re
import sys
import typing
from contextlib import redirect_stdout
from io import TextIOWrapper, BytesIO

from stencila.schema.types import Parameter, CodeChunk, Article, Entity, CodeExpression, ConstantSchema, EnumSchema, \
    BooleanSchema, NumberSchema, IntegerSchema, StringSchema, ArraySchema, TupleSchema
from stencila.schema.util import from_json, to_json

logger = logging.getLogger(__name__)
logger.addHandler(logging.NullHandler())

RESULT_CAPTURE_VAR = 'STENCILA_SCHEMA_EXEC_RESULT'
ASSIGNMENT_RE = re.compile(r'^[_a-z][a-z0-9]+\s*=')
IMPORT_RE = re.compile(r'^(import|from) ')


class StdoutBuffer(TextIOWrapper):
    def write(self, string: typing.Union[bytes, str]) -> int:
        if isinstance(string, str):
            return super(StdoutBuffer, self).write(string)
        else:
            return super(StdoutBuffer, self).buffer.write(string)


class Executor:
    parameters: typing.List[Parameter] = []
    code: typing.List[typing.Union[CodeChunk, CodeExpression]] = []
    globals: typing.Optional[typing.Dict[str, typing.Any]]

    def parse(self, source: Article) -> None:
        # todo: this traverses the article twice. Make it less hard coded, maybe pass through a lookup table that maps
        # a found type to its destination
        self.handle_item(source, Parameter, self.parameters, None)
        self.handle_item(source, (CodeChunk, CodeExpression), self.code, {'language': 'python'})

    def execute_code_chunk(self, chunk: CodeChunk, _locals: typing.Dict[str, typing.Any]) -> None:
        cc_outputs = []
        for statement in chunk.text.split('\n'):
            capture_result = False

            if IMPORT_RE.match(statement):
                capture_result = False
            elif not ASSIGNMENT_RE.match(statement):
                capture_result = True
                statement = '{} = {}'.format(RESULT_CAPTURE_VAR, statement)

            s = StdoutBuffer(BytesIO(), sys.stdout.encoding)

            with redirect_stdout(s):
                exec(statement, self.globals, _locals)

            if capture_result:
                result = _locals[RESULT_CAPTURE_VAR]
                del _locals[RESULT_CAPTURE_VAR]
                if result is not None:
                    cc_outputs.append(result)

            s.seek(0)
            output = s.buffer.read()

            if output:
                cc_outputs.append(output.decode('utf8'))

        chunk.outputs = cc_outputs

    def execute_code_expression(self, expression: CodeExpression, _locals: typing.Dict[str, typing.Any]) -> None:
        expression.output = eval(expression.text, self.globals, _locals)

    def execute(self, parameter_values: typing.Dict[str, typing.Any]) -> None:
        self.globals = {}

        _locals = parameter_values.copy()

        for c in self.code:
            if isinstance(c, CodeChunk):
                self.execute_code_chunk(c, _locals)
            elif isinstance(c, CodeExpression):
                self.execute_code_expression(c, _locals)
            else:
                raise TypeError('Unknown Code node type found: {}'.format(c))

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


class ParameterParser:
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
                param_parser.add_argument('--' + param.name, dest=param.name, required=param.default is None)

        args, _ = param_parser.parse_known_args(cli_args)

        logger.debug('Parsed command line args: {}'.format(args))

        for param_name in self.parameters:
            cli_value = getattr(args, param_name, None)
            if not cli_value and self.parameters[param_name].default:
                self.parameter_values[param_name] = self.parameters[param_name].default
            else:
                self.parameter_values[param_name] = self.deserialize_parameter(self.parameters[param_name], cli_value)

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

    e = Executor()
    e.parse(article)

    pp = ParameterParser(e.parameters)
    pp.parse_cli_args(cli_args)

    e.execute(pp.parameter_values)

    if args.output_file == '-':
        sys.stdout.write(to_json(article))
    else:
        with open(args.output_file, 'w') as o:
            o.write(to_json(article))


if __name__ == '__main__':
    logging.basicConfig(stream=sys.stdout, level=logging.DEBUG)
    execute_document(sys.argv[1:])
