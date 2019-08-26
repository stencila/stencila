import typing

from stencila.schema.interpreter import CodeChunkParser, CodeChunkParseResult
from stencila.schema.types import Variable, IntegerSchema, CodeChunk, Function, Parameter, SchemaTypes, StringSchema, \
    BooleanSchema

ASSIGNMENT_CODE = """
# this code assigns variables
a = 5
b = 6
a = 7
c: int = 8
c = 9
c: int = 10
def test_func():
    d = 4
"""

FUNCTION_CODE = """
# this code defines functions with various types of arguments
def basic():
    return 1
    
    
def standard_args(a, b, c):
    return 2
    
    
def variable_args(d, e, *args, **kwargs):
    return 3
    
    
def default_args(f = 1, g = 'foo'):
    return 4

    
def annotated_types(h: int, j: str = 'bar') -> bool:
    return True


def named_constants(t = True, f = False, n = None):
    return False
    
def function_defaults(v = somefunc()):
    return 0
"""

USES_CODE = """
# this code uses a lot of variables in different ways
a + b
d + e + f
g - h
i - j - k
l / m
n / o / p
q * r
s * t * u
v or w
x and y
z | aa
bb ^ cc
dd & ee
ff.gg
hh[ii]
jj.kk.ll
"""

OPEN_CODE = """
# this code has calls to open(), both in the main level and in defined functions

f = open('read1')  # no mode, assumed to be read

with open('read2', 'r')  as f:  # open with ContextManager and mode
    a = f.read()
    
open('write', 'w')  # is a write, don't include

open('readwrite', 'r+')  # read and write, include file 

open('unknownmode', r)  # variable mode, don't know what it is, skip

open(file='kwread')  # kwargs testing
open('kwread2', mode='r')
open(file='kwread3', mode='r')
open(file='kwread4', mode='r+')

open('kwwrite', mode='w')
open(file='kwwrite2', mode='w')
open(file=1, mode='w')  # should never happen but needed for full code coverage

open(v)  # don't know the actual file name since it's a variable, don't include
open(v.y)
open(v['y'])

def open_func():
    open('readinfunc')  # should traverse into function defs to find opens

"""

IMPORT_CODE = """
from abc import deff
from foo.bar import baz
import rex
"""

COLLECTIONS_CODE = """
# this code assigns collections with variables, so they must be in the 'uses' result

[a, b, c, {d: e, f: g, h: somefunc(i)}]
[j, k, {l: m, n: o[p][q]}, anotherfunc(r)]
(s, t, {u: v}, [w, x, lastfunc(y[z])])
{'foo': 'bar'}  # shouldn't be in 'uses'
"""

SLICES_CODE = """
# this code checks for parsing of different ways of slicing array to 'uses'

a[b]
a[c:d]
a[d:e:f]
"""


def check_result_fields_empty(result: CodeChunkParseResult, non_empty_fields: typing.List[str]) -> None:
    for name, value in result._asdict().items():
        if name in non_empty_fields:
            continue
        if isinstance(value, list):
            assert len(value) == 0


def check_parameter(p: Parameter, name: str, required: bool, default: typing.Any,
                    schema: typing.Optional[typing.Type[SchemaTypes]]) -> None:
    assert p.name == name
    assert p.required == required
    assert p.default == default
    if schema is not None:
        assert isinstance(p.schema, schema)


def test_variable_parsing() -> None:
    """
    Test that assignments without annotations are extracted into `assigns` and assignments with are to `declares.`

    Also test that:
     - function definitions are recorded as declarations (basic test just to have a function body to parse, actual
        function parsing tests are in test_function_def_parsing)
     - variables that are reassigned are only recorded once
     - assignment/declarations in function definitions are not recorded
    """
    c = CodeChunk(ASSIGNMENT_CODE)
    ccp = CodeChunkParser()
    parse_result = ccp.parse(c)

    assert len(parse_result.declares) == 2
    assert type(parse_result.declares[0]) == Variable
    assert parse_result.declares[0].name == 'c'
    assert type(parse_result.declares[0].schema) == IntegerSchema

    assert type(parse_result.declares[1]) == Function  # The correctness of parsing the function is tested elsewhere

    assert parse_result.assigns == ['a', 'b']

    check_result_fields_empty(parse_result, ['declares', 'assigns'])


def test_function_def_parsing():
    c = CodeChunk(FUNCTION_CODE)
    ccp = CodeChunkParser()
    parse_result = ccp.parse(c)

    basic, standard_args, variable_args, default_args, annotated_types, named_constants, function_defaults = \
        parse_result.declares

    for fn in parse_result.declares:
        assert isinstance(fn, Function)
        if fn != annotated_types:
            assert fn.returns is None

    assert basic.name == 'basic'
    assert len(basic.parameters) == 0

    assert standard_args.name == 'standard_args'
    assert len(standard_args.parameters) == 3
    check_parameter(standard_args.parameters[0], 'a', True, None, None)
    check_parameter(standard_args.parameters[1], 'b', True, None, None)
    check_parameter(standard_args.parameters[2], 'c', True, None, None)

    assert variable_args.name == 'variable_args'
    assert len(variable_args.parameters) == 4
    check_parameter(variable_args.parameters[0], 'd', True, None, None)
    check_parameter(variable_args.parameters[1], 'e', True, None, None)

    check_parameter(variable_args.parameters[2], 'args', False, None, None)
    assert variable_args.parameters[2].repeats is True
    assert not variable_args.parameters[2].extends

    check_parameter(variable_args.parameters[3], 'kwargs', False, None, None)
    assert not variable_args.parameters[3].repeats
    assert variable_args.parameters[3].extends is True

    assert default_args.name == 'default_args'
    assert len(default_args.parameters) == 2
    check_parameter(default_args.parameters[0], 'f', False, 1, None)
    check_parameter(default_args.parameters[1], 'g', False, 'foo', None)

    assert annotated_types.name == 'annotated_types'
    assert len(annotated_types.parameters) == 2
    assert isinstance(annotated_types.returns, BooleanSchema)
    check_parameter(annotated_types.parameters[0], 'h', True, None, IntegerSchema)
    check_parameter(annotated_types.parameters[1], 'j', False, 'bar', StringSchema)

    assert named_constants.name == 'named_constants'
    assert len(named_constants.parameters) == 3
    check_parameter(named_constants.parameters[0], 't', False, True, None)
    check_parameter(named_constants.parameters[1], 'f', False, False, None)
    check_parameter(named_constants.parameters[2], 'n', False, None, None)

    assert function_defaults.name == 'function_defaults'
    assert len(function_defaults.parameters) == 1
    check_parameter(function_defaults.parameters[0], 'v', False, None, None)

    check_result_fields_empty(parse_result, ['declares'])


def test_uses_parsing():
    c = CodeChunk(USES_CODE)
    ccp = CodeChunkParser()
    parse_result = ccp.parse(c)

    check_result_fields_empty(parse_result, ['uses'])

    uses = ['a', 'b', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v',
            'w', 'x', 'y', 'z', 'aa', 'bb', 'cc', 'dd', 'ee', 'ff', 'hh', 'ii', 'jj']

    assert sorted(uses) == sorted(parse_result.uses)


def test_parsing_error():
    c = CodeChunk('this is invalid python++ code')
    ccp = CodeChunkParser()
    parse_result = ccp.parse(c)

    assert parse_result.error.kind == 'SyntaxError'
    assert parse_result.error.message == 'invalid syntax (<unknown>, line 1)'


def test_reads_parsing():
    c = CodeChunk(OPEN_CODE)
    ccp = CodeChunkParser()
    parse_result = ccp.parse(c)

    filenames = ['read1', 'read2', 'readwrite', 'kwread', 'kwread2', 'kwread3', 'kwread4', 'readinfunc']

    assert sorted(filenames) == sorted(parse_result.reads)


def test_imports_parsing():
    c = CodeChunk(IMPORT_CODE)
    ccp = CodeChunkParser()
    parse_result = ccp.parse(c)

    assert ['abc', 'foo.bar', 'rex'] == sorted(parse_result.imports)

    check_result_fields_empty(parse_result, ['imports'])


def test_collections_parsing():
    c = CodeChunk(COLLECTIONS_CODE)
    ccp = CodeChunkParser()
    parse_result = ccp.parse(c)

    assert [chr(c) for c in range(ord('a'), ord('z') + 1)] == sorted(parse_result.uses)

    check_result_fields_empty(parse_result, ['uses'])


def test_slices_parsing():
    c = CodeChunk(SLICES_CODE)
    ccp = CodeChunkParser()
    parse_result = ccp.parse(c)

    assert [chr(c) for c in range(ord('a'), ord('f') + 1)] == sorted(parse_result.uses)

    check_result_fields_empty(parse_result, ['uses'])
