import unittest.mock
import sys

import pytest

from stencila.schema.interpreter import ParameterParser
from stencila.schema.types import Parameter, ConstantSchema, EnumSchema, BooleanSchema, IntegerSchema, NumberSchema, \
    StringSchema, ArraySchema, TupleSchema


def test_parameter_deserializer():
    pp = ParameterParser([])

    # Constant schemas should always return the value from the schema
    assert pp.deserialize_parameter(Parameter('p', schema=ConstantSchema(value='abc123')), 'some string') == 'abc123'
    assert pp.deserialize_parameter(Parameter('p', schema=ConstantSchema(value='abc123')), None) == 'abc123'

    assert pp.deserialize_parameter(Parameter('p', schema=EnumSchema(values=['a', 'b'])), 'a') == 'a'

    with pytest.raises(TypeError):
        # value must be in the enum
        pp.deserialize_parameter(Parameter('p', schema=EnumSchema(values=['a', 'b'])), 'c')

    for t in ['TRUE', 'true', 't', 'T', '1', 'YES', 'yes']:
        assert pp.deserialize_parameter(Parameter('p', schema=BooleanSchema()), t) is True

    for f in ['false', '0', 'no', '', 'anything else']:
        assert pp.deserialize_parameter(Parameter('p', schema=BooleanSchema()), f) is False

    assert pp.deserialize_parameter(Parameter('p', schema=IntegerSchema()), '1000') == 1000

    # Not pi. But close.
    assert pp.deserialize_parameter(Parameter('p', schema=NumberSchema()), '3.1418') == 3.1418

    assert pp.deserialize_parameter(Parameter('p', schema=StringSchema()), '3.1418') == '3.1418'

    assert pp.deserialize_parameter(Parameter('p', schema=ArraySchema()), '[5, 6, 7]') == [5, 6, 7]

    assert pp.deserialize_parameter(Parameter('p', schema=TupleSchema()), '[1, true, "Up"]') == [1, True, 'Up']

    # default to String
    assert pp.deserialize_parameter(Parameter('p'), '321bca') == '321bca'


def test_parse_with_empty_parameters():
    pp = ParameterParser([])

    # should run without issue
    pp.parse_cli_args(['--something', 'value'])


def test_exit_on_missing_parameters():
    """ArgumentParser calls sys.exit(2) if required parameters are missing. Mock that call and verify it occurs."""
    pp = ParameterParser([
        Parameter('foo', required=True),
        Parameter('bar', required=True)
    ])

    old_exit = sys.exit
    sys.exit = unittest.mock.MagicMock()

    pp.parse_cli_args(['--foo', 'fooval'])

    sys.exit.assert_called_with(2)

    sys.exit = old_exit


def test_parameter_cli_parsing():
    pp = ParameterParser([
        Parameter('foo', required=True),
        Parameter('bar', required=True),
        Parameter('baz', required=False),
        Parameter('rex', required=False, default='rex_default'),
        Parameter('quz', required=False, schema=IntegerSchema()),
    ])

    pp.parse_cli_args(['--foo', 'fooval', '--bar', 'barval', '--quz', '1024', '--invalid', 'not a real param'])

    assert len(pp.parameter_values) == 5

    assert pp.parameter_values['foo'] == 'fooval'
    assert pp.parameter_values['bar'] == 'barval'
    assert pp.parameter_values['baz'] is None
    assert pp.parameter_values['rex'] == 'rex_default'
    assert pp.parameter_values['quz'] == 1024
