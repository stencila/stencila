import json
import logging
import sys
import typing
from io import BytesIO

from stencila.schema.code_parsing import simple_code_chunk_parse
from stencila.schema.interpreter import Interpreter
from stencila.schema.types import CodeChunk
from stencila.schema.util import to_json, from_dict


def _byte(b: typing.Any) -> bytes:
    return bytes((b,))


def encode(number: int) -> bytes:
    """Pack `number` into varint bytes"""
    buf = b''
    while True:
        towrite = number & 0x7f
        number >>= 7
        if number:
            buf += _byte(towrite | 0x80)
        else:
            buf += _byte(towrite)
            break
    return buf


def decode_stream(stream: typing.IO) -> int:
    """Read a varint from `stream`"""
    shift = 0
    result = 0
    while True:
        i = _read_one(stream)
        result |= (i & 0x7f) << shift
        shift += 7
        if not (i & 0x80):
            break

    return result


def decode_bytes(buf: bytes) -> int:
    """Read a varint from from `buf` bytes"""
    return decode_stream(BytesIO(buf))


def _read_one(stream: typing.IO) -> int:
    """Read a byte from the file (as an integer)
    raises EOFError if the stream ends while reading bytes.
    """
    if hasattr(stream, 'recv'):
        c = stream.recv(1)
    elif hasattr(stream, 'buffer'):
        c = stream.buffer.read(1)
    else:
        c = stream.read(1)
    if c == b'':
        raise EOFError("Unexpected EOF while reading bytes")
    return ord(c)


class InterpreterListener:
    input_stream: typing.IO
    output_stream: typing.IO
    interpreter: Interpreter

    def __init__(self, input_stream: typing.IO, output_stream: typing.IO, interpreter: Interpreter) -> None:
        self.input_stream = input_stream
        self.output_stream = output_stream
        self.interpreter = interpreter

    def read_message(self) -> typing.Iterable[bytes]:
        while True:
            message_len = decode_stream(self.input_stream)
            yield self.input_stream.read(message_len)

    def write_message(self, s: bytes) -> None:
        self.output_stream.write(encode(len(s)))
        self.output_stream.write(s)
        self.output_stream.flush()

    def run_interpreter(self) -> None:
        for message in self.read_message():
            request = json.loads(message.decode('utf8'))

            code = from_dict(request['params']['node'])

            sys.stderr.write(message.decode('utf8'))
            sys.stderr.flush()

            if isinstance(code, CodeChunk):
                code = simple_code_chunk_parse(code)

            self.interpreter.execute([code], {})

            response = {
                'jsonrpc': '2.0',
                'id': request['id'],
                'result': code
            }

            self.write_message(to_json(response).encode('utf8'))


def start_stdio_interpreter() -> None:
    il = InterpreterListener(sys.stdin.buffer, sys.stdout.buffer, Interpreter())
    il.run_interpreter()


if __name__ == '__main__':
    logging.basicConfig(stream=sys.stdout, level=logging.DEBUG)
    start_stdio_interpreter()
