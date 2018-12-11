"""
Module that defines the `Server` class
"""

from typing import Any, Dict
import asyncio
import json
import signal
import traceback

from ..Processor import Processor
from .jsonRpc import Request, Response, Error
from .JsonEncoder import JSON_ENCODING, JsonEncoder
from .JsonGzipEncoder import JSON_GZIP_ENCODING, JsonGzipEncoder
from .Logger import Logger

class Server(Logger):
    """
    Base class for all servers.
    """

    processor: Processor
    """
    The procecessor that this server dispatches requests to.
    """

    def __init__(self, processor: Processor):
        self.processor = processor

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

    async def receive(self, message: bytes, format: Dict = JSON_ENCODING):
        assert self.processor
        
        response = Response()

        try:
            request = self.decode(message, format)
        except Exception as exc:
            response.error = Error.parse_error(str(exc))
            return response
        
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
        return self.encode(response, format)

    def supports(self, encoding: Dict) -> bool:
        """
        Does this server support the given encoding?

        This method may be overriden by a derived class to expand or restrict
        the supported encodings
        """
        return encoding == JSON_ENCODING or encoding == JSON_GZIP_ENCODING

    def decode(self, message: bytes, encoding: Dict = JSON_ENCODING) -> Request:
        """
        Decode a request message.

        Currently, this assumes that the message is a JSON string.
        In the future, alternative message encodings will be available.
        """

        if encoding == JSON_ENCODING:
            return JsonEncoder.decode(message, Request)
        elif encoding == JSON_GZIP_ENCODING:
            return JsonGzipEncoder.decode(message, Request)
        raise RuntimeError(f'Unhandled encoding: {encoding}')

    def encode(self, response: Response, encoding: Dict = JSON_ENCODING) -> bytes:
        """
        Encode a response message

        Currently, this simply encodes the response as a JSON string.
        """

        if encoding == JSON_ENCODING:
            return JsonEncoder.encode(response)
        elif encoding == JSON_GZIP_ENCODING:
            return JsonGzipEncoder.encode(response)
        raise RuntimeError(f'Unhandled encoding: {encoding}')

    async def handle_hello(self, request):
        # Intercept the call to hello to get the declared list of encodings
        version = request.param(0, 'version')
        result = await self.processor.hello(version)

        encoding_to_use = JSON_ENCODING
        encodings = request.param(1, 'encodings', False)
        if encodings:
            for encoding in encodings:
                if self.supports(encoding):
                    encoding_to_use = encoding
                    break
        result['encoding'] = encoding_to_use

        return result

    async def handle_goodbye(self, request):
        return await self.processor.goodbye()