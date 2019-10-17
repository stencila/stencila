"""An interpreter that listens and executes commands in a loop."""

import json
import logging
import sys
import typing
from socket import socket

from stencila.schema.code_parsing import simple_code_chunk_parse
from stencila.schema.interpreter import Interpreter
from stencila.schema.types import CodeChunk
from stencila.schema.util import to_json, from_dict

StreamType = typing.Union[typing.BinaryIO, socket]


def _byte(data: typing.Any) -> bytes:
    """Convert `data` to `bytes`."""
    return bytes((data,))


def encode_int(number: int) -> bytes:
    """Pack `number` into varint bytes"""
    buf = b''
    while True:
        to_write = number & 0x7f
        number >>= 7
        if number:
            buf += _byte(to_write | 0x80)
        else:
            buf += _byte(to_write)
            break
    return buf


def read_length_prefix(stream: StreamType) -> int:
    """Read a varint from `stream`"""
    shift = 0
    result = 0
    while True:
        i = _read_one(stream)
        result |= (i & 0x7f) << shift
        shift += 7
        if not i & 0x80:
            break

    return result


def get_stream_buffer(stream: typing.BinaryIO) -> typing.BinaryIO:
    """Get the buffer from a stream, if it exists."""
    buffer = getattr(stream, 'buffer', None)
    return buffer if buffer else stream


def io_read(stream: typing.BinaryIO, count: int) -> bytes:
    """Read `count` bytes from `stream` or its underlying buffer if it exists."""
    return get_stream_buffer(stream).read(count)


def io_write(stream: typing.BinaryIO, message: bytes) -> None:
    """Write to `stream` or its underlying buffer if it exists."""
    stream = get_stream_buffer(stream)
    stream.write(message)
    stream.flush()


def stream_read(stream: StreamType, count: int) -> bytes:
    """Abstract reading from stream to work with IO (buffered/unbuffered) and sockets."""
    if isinstance(stream, socket):
        return stream.recv(count)

    return io_read(stream, count)


def stream_write(stream: StreamType, message: bytes) -> None:
    """Abstract writing to stream to work with IO (buffered/unbuffered) and sockets."""
    if isinstance(stream, socket):
        stream.send(message)
    else:
        io_write(stream, message)


def _read_one(stream: StreamType) -> int:
    """Read a byte from the file (as an integer).

    Raises EOFError if the stream ends while reading bytes.
    """
    char = stream_read(stream, 1)
    if char == b'':
        raise EOFError('Unexpected EOF while reading bytes')
    return ord(char)


class InterpreterListener:
    """A looping interpreter listener that communicates use LPS over streams or sockets."""

    input_stream: StreamType
    output_stream: StreamType
    interpreter: Interpreter

    def __init__(self, input_stream: StreamType, output_stream: StreamType, interpreter: Interpreter) -> None:
        self.input_stream = input_stream
        self.output_stream = output_stream
        self.interpreter = interpreter

    def read_message(self) -> typing.Iterable[bytes]:
        """Read one LPS message message and yield it, then repeat."""
        while True:
            message_len = read_length_prefix(self.input_stream)
            yield stream_read(self.input_stream, message_len)

    def write_message(self, message: bytes) -> None:
        """Write a message to the output stream."""
        stream_write(self.output_stream, encode_int(len(message)))
        stream_write(self.output_stream, message)

    def run_interpreter(self) -> None:
        """
        Run the interpreter in a loop forever (since the `read_message` generator never finishes).

        Sends the response back in JSONRPC format.
        """
        for message in self.read_message():
            request = json.loads(message.decode('utf8'))

            code = from_dict(request['params']['node'])

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
    """Start the looping interpreter, reading from stdin and writing to stdout."""
    InterpreterListener(sys.stdin.buffer, sys.stdout.buffer, Interpreter()).run_interpreter()


if __name__ == '__main__':
    logging.basicConfig(stream=sys.stdout, level=logging.DEBUG)
    start_stdio_interpreter()
