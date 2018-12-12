"""
Module that defines the `Server` class
"""

from typing import Any, Dict, List, Optional, Type
import asyncio
import json
import signal
import traceback

from ..Processor import Processor
from .jsonRpc import Request, Response, Error
from .Encoder import Encoder
from .JsonEncoder import JsonEncoder
from .JsonGzipEncoder import JsonGzipEncoder
from .Logger import Logger

class Server(Logger):
    """
    Base class for all servers.
    """

    processor: Processor
    """
    The procecessor that this server dispatches requests to.
    """
    
    encoders: List[Encoder]
    """
    The encoders that this server is able to use for
    encoding/decoding messages.
    """

    def __init__(self, processor: Processor, encoders: Optional[List[Encoder]] = None):
        self.processor = processor
        
        if encoders is None:
            encoders = [JsonEncoder(), JsonGzipEncoder()]
        else:
            assert len(encoders) > 0
        self.encoders = encoders

    async def start(self) -> None:
        """
        Start this server.

        Starts listening for requests.
        """
        self.log(starting=True)
        await self.open()

    async def open(self) -> None:
        raise NotImplementedError()

    async def stop(self) -> None:
        """
        Stop this server.

        Stops listening for requests.
        """
        await self.close()
        self.log(stopped=True)

    async def close(self) -> None:
        raise NotImplementedError()

    def run(self) -> None:
        loop = asyncio.get_event_loop()

        async def run():
            await self.start()

            def stop():
                self.running = False
                asyncio.ensure_future(self.stop())
                loop.remove_signal_handler(signal.SIGINT)
                loop.remove_signal_handler(signal.SIGTERM)
            loop.add_signal_handler(signal.SIGINT, stop)
            loop.add_signal_handler(signal.SIGTERM, stop)

            self.running = True
            while self.running:
                await asyncio.sleep(1)

        loop.run_until_complete(run())
        loop.close()

    async def receive(self, message: bytes, encoding: Optional[str] = 'json') -> bytes:
        assert self.processor
        
        response = Response()

        try:
            request = self.decode(message, encoding)
        except Exception as exc:
            response.error = Error.parse_error(str(exc))
        else:
            response.id = request.id

            if not request.method:
                response.error = Error.invalid_request('missing "method" property')

            try:
                result: Any = None
                if request.method == 'hello':
                    result = await self.handle_hello(request)
                elif request.method == 'goodbye':
                    result = await self.handle_goodbye(request)
                elif request.method == 'import':
                    result = await self.processor.import_(
                        request.param(0, 'thing'),
                        request.param(1, 'format', False)
                    )
                elif request.method == 'export':
                    result = await self.processor.export(
                        request.param(0, 'thing'),
                        request.param(1, 'format', False)
                    )
                elif request.method == 'compile':
                    result = await self.processor.compile(
                        request.param(0, 'thing'),
                        request.param(1, 'format', False)
                    )
                elif request.method == 'build':
                    result = await self.processor.build(
                        request.param(0, 'thing'),
                        request.param(1, 'format', False)
                    )
                elif request.method == 'execute':
                    result = await self.processor.execute(
                        request.param(0, 'thing'),
                        request.param(1, 'format', False)
                    )
                else:
                    raise Error.method_not_found(request.method, { 'method': request.method })

                response = Response(id=request.id, result=result)
                self.log(request=request, response=response)
            except Exception as exc:
                #raise exc
                if isinstance(exc, Error):
                    error = exc
                else:
                    error = Error.application_error(str(exc), { 'trace': traceback.format_exc() })
                response.error = error

        return self.encode(response, encoding)

    def supports(self, encoding: str) -> bool:
        """
        Does this server support the given encoding?
        """
        for encoder in self.encoders:
            if encoder.name() == encoding:
                return True
        return False

    def decode(self, message: bytes, encoding: Optional[str] = 'json') -> Request:
        """
        Decode a request message

        :param message: The message
        :type message: bytes
        :param encoding: The encoding of the message
        :param encoding: str
        :raises RuntimeError: If the encoding is not supported by this server
        :return: The request
        :rtype: Request
        """
        for encoder in self.encoders:
            if encoder.name() == encoding:
                return encoder.decode(message, Request)
        raise RuntimeError(f'Unhandled encoding: {encoding}')

    def encode(self, response: Response, encoding: Optional[str] = 'json') -> bytes:
        """
        Encode a response message
        """
        for encoder in self.encoders:
            if encoder.name() == encoding:
                return encoder.encode(response)
        raise RuntimeError(f'Unhandled encoding: {encoding}')

    async def handle_hello(self, request):
        # Intercept the call to hello to get the declared list of encodings
        version = request.param(0, 'version')
        result = await self.processor.hello(version)

        # If possible support the client's preferred encoding
        # but if not then fall back to this server's default
        encodings = request.param(1, 'encodings', False)
        if encodings:
            encoding_to_use = None
            for encoding in encodings:
                if self.supports(encoding):
                    encoding_to_use = encoding
                    break
            if encoding_to_use is None:
                raise RuntimeError(f'Unable to support any of the client encodings')
        else:
            encoding_to_use = 'json'
        result['encoding'] = encoding_to_use

        return result

    async def handle_goodbye(self, request):
        return await self.processor.goodbye()
