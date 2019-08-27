import typing

from stencila.schema import util
from stencila.schema.interpreter import DocumentCompiler, CodeChunkExecution
from stencila.schema.types import Article, Parameter, CodeExpression


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
