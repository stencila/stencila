from typing import Any, Dict, List, Optional, Type, Union
from asyncio import Future
import json

from .jsonRpc import Request, Response
from .Encoder import Encoder
from .JsonEncoder import JsonEncoder
from .Logger import Logger

class Client(Logger):

    encoders: List[Encoder]

    encoder: Encoder

    futures: Dict[int, Future] = {}

    def __init__(self, encoders: Optional[List[Encoder]] = None):
        if encoders is None:
            encoders = [JsonEncoder()]
        else:
            assert len(encoders) > 0
        self.encoders = encoders

        # The encoder used for the first, `hello` request
        # is always a JSON encoder, but may be replaced after that
        # request based on the response from the server
        self.encoder = JsonEncoder()

    async def start(self) -> None:
        """
        Start this client.

        Opens the connection to the server and makes
        a `hello` handshake request.
        """
        self.log(starting=True)
        await self.open()
        await self.hello()

    async def stop(self) -> None:
        """
        Stop this client.

        Make a `goodbye` request and closes the connection to
        the server.
        """
        await self.goodbye()
        await self.close()
        self.log(stopped=True)

    async def hello(self) -> None:
        result = await self.call(
            "hello",
            version="1.0",
            encodings=[encoder.name() for encoder in self.encoders]
        )
        # Get the encoding to use from the result
        encoding = result.get('encoding')
        if encoding:
            encoders = [encoder for encoder in self.encoders if encoder.name() == encoding]
            assert len(encoders) == 1
            self.encoder = encoders[0]

    async def goodbye(self) -> None:
        await self.call("goodbye")

    async def execute(self, thing):
        return await self.call("execute", thing=thing)

    async def call(self, method: str, **kwargs):
        request = Request(method=method, params=kwargs)
        future = await self.send(request)
        await future
        response = future.result()
        self.log(request=request, response=response)
        return response.result

    async def send(self, request: Request) -> Future:
        """
        Send a request to the server.

        This method must be overriden by derived client classes to
        send the request over the transport protocol used by that class.

        :param: request The JSON-RPC request to send
        """
        future: Future = Future()
        self.futures[request.id] = future
        await self.write(self.encode(request))
        return future

    def receive(self, response: Response) -> None:
        """
        Receive a request from the server.

        Uses the `id` of the response to match it to the corresponding
        request and resolve it's promise.

        :param: response The JSON-RPC response as a string or Response instance
        """
        if not response.id:
            raise RuntimeError(f'Response does not have an id: {response.__dict__}')
        future = self.futures.get(response.id)
        if not future:
            raise RuntimeError(f'No request found for response with id: {response.id}')
        future.set_result(response)
        del self.futures[response.id]

    async def open(self) -> None:
        """
        Open the connection to the server.

        Should be implemented in derived classes to
        open connections to a server before the `hello`
        request is made.
        """
        raise NotImplementedError()

    async def close(self) -> None:
        """
        Close the connection to the server.

        Should be implemented in derived classes to
        close connections to a server after the `goodbye`
        request is made.
        """
        raise NotImplementedError()

    def decode(self, message: bytes) -> Response:
        return self.encoder.decode(message, Response)

    def encode(self, request: Request) -> bytes:
        return self.encoder.encode(request)

    async def read(self, message: bytes) -> None:
        # Recieve a response message
        self.receive(self.decode(message))

    async def write(self, message: bytes) -> None:
        raise NotImplementedError()
