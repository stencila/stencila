from stencila.schema.interpreter import parse_code_chunk
from stencila.schema.types import Variable, IntegerSchema, CodeChunk


def test_variable_parsing():
    """Test that variables without annotations are extracted into `assigns` and variables with are to `declares.`"""
    c = CodeChunk("no_ann = 5\nwith_ann: int = 10")

    parse_result = parse_code_chunk(c)

    assert len(parse_result.declares) == 1
    assert type(parse_result.declares[0]) == Variable
    assert parse_result.declares[0].name == 'with_ann'
    assert type(parse_result.declares[0].schema) == IntegerSchema

    assert len(parse_result.assigns) == 1
    assert type(parse_result.assigns[0]) == Variable
    assert parse_result.assigns[0].name == 'no_ann'
    assert parse_result.assigns[0].schema is None


def test_variable_reassignment():
    """
    If a variable is declared and set and then set to another value later, it should only be in the `declares` array.
    """
    c = CodeChunk("with_ann: int = 10\nwith_ann = 5")

    parse_result = parse_code_chunk(c)

    assert len(parse_result.declares) == 1
    assert type(parse_result.declares[0]) == Variable
    assert parse_result.declares[0].name == 'with_ann'
    assert type(parse_result.declares[0].schema) == IntegerSchema

    assert len(parse_result.assigns) == 0
