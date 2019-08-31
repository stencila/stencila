import typing

from stencila.schema import util
from stencila.schema.interpreter import DocumentCompiler, CodeChunkExecution
from stencila.schema.types import Article, Parameter, CodeExpression, CodeChunk


def test_compile_article():
    chunk_1_text = 'a = 4'
    chunk_2_text = 'invalid python code'
    chunk_3_text = 'def somefunc(bad_param):\n    return bad_param'

    expr_1_text = 'a * parameter_one'
    expr_2_text = 'more invalid python code'

    article = util.from_dict({
        'type': 'Article',
        'title': 'Upcoming Temperatures',
        'authors': [],
        'content': [
            {
                'type': 'Parameter',
                'name': 'parameter_one',
                'schema': {
                    'type': 'IntegerSchema'
                }
            },
            {
                'type': 'Heading',
                'depth': 1,
                'content': [
                    'A Heading'
                ]
            },
            {
                'type': 'CodeChunk',
                'text': 'let a = \'I am JavaScript!\'',
                'language': 'notpython'
            },
            {
                'type': 'CodeChunk',
                'text': chunk_1_text,
                'language': 'python'
            },
            {
                'type': 'CodeChunk',
                'text': chunk_2_text,
                'language': 'python'
            },
            {
                'type': 'CodeChunk',
                'text': chunk_3_text,
                'language': 'python',
                'declares': [
                    {
                        'type': 'Function',
                        'name': 'somefunc',
                        'parameters': [
                            {
                                'type': 'Parameter',
                                'name': 'bad_param',
                                'required': True
                            }
                        ]
                    }
                ]
            },
            {
                'type': 'CodeExpression',
                'text': 'invalid code',
                'language': 'notpython'
            },
            {
                'type': 'CodeExpression',
                'text': expr_1_text,
                'language': 'python'
            },
            {
                'type': 'CodeExpression',
                'text': expr_2_text,
                'language': 'python'
            }
        ]
    })

    article = typing.cast(Article, article)

    dc = DocumentCompiler()
    dcr = dc.compile(article)
    assert len(dcr.parameters) == 1
    assert isinstance(dcr.parameters[0], Parameter)
    assert dcr.parameters[0].name == 'parameter_one'

    assert len(dcr.code) == 5

    for c in dcr.code:
        if isinstance(c, CodeChunkExecution):
            c = c[0]
        assert c.language == 'python'

    code_chunks = list(map(lambda c: c[0], filter(lambda ce: isinstance(ce, CodeChunkExecution), dcr.code)))
    code_exprs = list(filter(lambda ce: isinstance(ce, CodeExpression), dcr.code))

    assert code_chunks[0].text == chunk_1_text
    assert code_chunks[1].text == chunk_2_text
    assert code_chunks[2].text == chunk_3_text

    assert code_exprs[0].text == expr_1_text
    assert code_exprs[1].text == expr_2_text


def test_import_appending():
    """Found imports in a piece of code should be added to the list of imports the code chunk already specifies."""
    c = CodeChunk('import moda\nimport modb\nimport modc', imports=['modc', 'modd'], language='python')

    dc = DocumentCompiler()
    dc.compile(c)

    assert len(c.imports) == 4
    assert 'moda' in c.imports
    assert 'modb' in c.imports
    assert 'modc' in c.imports
    assert 'modd' in c.imports


def test_import_with_semaphore():
    """If a `CodeChunk`'s imports has an empty string element then no imports should be added to its list."""
    c = CodeChunk('import moda\nimport modb', imports=['modc', 'modd', ''])

    dc = DocumentCompiler()
    dc.compile(c)

    assert len(c.imports) == 3
    assert 'moda' not in c.imports
    assert 'modb' not in c.imports
    assert 'modc' in c.imports
    assert 'modd' in c.imports
    assert '' in c.imports
