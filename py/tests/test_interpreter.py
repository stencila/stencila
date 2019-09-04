import unittest.mock

from stencila.schema.interpreter import Interpreter, DocumentCompiler, ParameterParser, execute_compilation, \
    compile_article, DocumentCompilationResult, SKIP_OUTPUT_SEMAPHORE
from stencila.schema.code_parsing import CodeChunkParseResult, CodeChunkExecution, CodeChunkParser
from stencila.schema.types import CodeExpression, CodeChunk, Article


def execute_code_chunk(text: str) -> CodeChunk:
    cc = CodeChunk(text)
    cce = CodeChunkExecution(
        cc, CodeChunkParser().parse(cc)
    )
    Interpreter().execute([cce], {})
    return cc


def test_execute_simple_code_expression():
    ce = CodeExpression('4 + 5')
    Interpreter().execute([ce], {})
    assert ce.output == 9


def test_execute_parameterised_code_expression():
    ce = CodeExpression('p1 + p2')
    Interpreter().execute([ce], {'p1': 1, 'p2': 10})
    assert ce.output == 11


def test_catch_code_expression_error():
    ce = CodeExpression('1 / 0')
    Interpreter().execute([ce], {})
    assert ce.output is None
    assert ce.errors[0].kind == 'ZeroDivisionError'
    assert ce.errors[0].message == 'division by zero'
    assert ce.errors[0].trace is not None


@unittest.mock.patch('stencila.schema.interpreter.LOGGER')
@unittest.mock.patch('stencila.schema.interpreter.exec')
@unittest.mock.patch('stencila.schema.interpreter.eval')
def test_execute_code_chunk_without_ast(mock_eval, mock_exec, mock_logger):
    """If parsing the code fails to generate an AST then the code should not attempt to be executed."""
    execute_code_chunk('invalid code')
    assert mock_logger.info.called
    assert mock_exec.called is False  # make sure nothing is executed
    assert mock_eval.called is False


def test_output_capture():
    """Output to STDOUT should be captured in the CodeChunk's outputs property."""
    cc = execute_code_chunk('print(\'Hello world!\')')
    assert cc.outputs == ['Hello world!\n']


def test_result_capture():
    """Variable assignment should not be captured as an output, return values from functions should (for example)."""
    cc = execute_code_chunk('a = 5\ndef add_five(b):\n    return b + 5\nadd_five(a)')
    assert cc.outputs == [10]


def test_duration():
    """
    CodeChunk execution duration should be captured. We don't want to slow down running tests so just check it's
    greater than 0.
    """
    cc = execute_code_chunk('for i in range(10):\n b = i + 1')
    assert cc.duration > 0


def test_code_chunk_exception_capture():
    """
    If an Exception occurs it should be recorded and code outputs up to that point added to outputs.  The rest of the
    code should not be run, although subsequent code blocks should.
    """
    cc1 = CodeChunk('a = 5\na + 2\nprint(\'Goodbye world!\')\nbadref += 1\nprint(\'After exception!\')')
    cc2 = CodeChunk('2 + 2\nprint(\'CodeChunk2\')')

    cce1 = CodeChunkExecution(
        cc1, CodeChunkParser().parse(cc1)
    )
    cce2 = CodeChunkExecution(
        cc2, CodeChunkParser().parse(cc2)
    )

    Interpreter().execute([cce1, cce2], {})
    assert cc1.outputs == [7, 'Goodbye world!\n']
    assert cc1.errors[0].kind == 'NameError'

    assert cc2.outputs == [4, 'CodeChunk2\n']


@unittest.mock.patch('stencila.schema.interpreter.DocumentCompiler')
def test_compile_article(mock_dc_class):
    article = unittest.mock.MagicMock(spec=Article)

    dcr = compile_article(article)

    mock_dc_class.return_value.compile.assert_called_with(article)
    assert mock_dc_class.return_value.compile.return_value == dcr


@unittest.mock.patch('stencila.schema.interpreter.ParameterParser')
@unittest.mock.patch('stencila.schema.interpreter.Interpreter')
def test_execute_compilation(mock_interpreter_class, mock_pp_class):
    compilation_result = unittest.mock.MagicMock(spec=DocumentCompilationResult)
    parameters = ['--flag', 'value']

    parameter_parser = mock_pp_class.return_value
    interpreter = mock_interpreter_class.return_value

    execute_compilation(compilation_result, parameters)

    mock_pp_class.assert_called_with(compilation_result.parameters)
    parameter_parser.parse_cli_args.assert_called_with(parameters)
    interpreter.execute.assert_called_with(compilation_result.code, parameter_parser.parameter_values)


def test_sempahore_skipping():
    """If decode_output returns a SKIP_OUTPUT_SEMAPHORE then it should not be added to the outputs array."""
    i = Interpreter()
    outputs = []

    i.add_output(outputs, 'abc123')

    decode_original = i.decode_output
    i.decode_output = unittest.mock.MagicMock(return_value=SKIP_OUTPUT_SEMAPHORE)
    i.add_output(outputs, 'skip')
    i.decode_output = decode_original

    i.add_output(outputs, [1, 2, 3])

    assert outputs == ['abc123', [1, 2, 3]]
