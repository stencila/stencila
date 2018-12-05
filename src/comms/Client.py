from typing import cast, Any, Dict, List, Optional, Union
from asyncio import Future
import json
import sys

from .jsonRpc import Request, Response
from .Logger import Logger

class Client(Logger):

    futures: Dict[int, Future] = {}

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

    async def hello(self, version: str = "1.0", name: Optional[str] = None,
                    messages: List[Dict[str, Any]]=[{"contentType": "application/json"}]) -> None:
        await self.call("hello", version=version, name=name, messages=messages)

    async def goodbye(self) -> None:
        await self.call("goodbye")

    async def execute(self, thing):
        return await self.call("execute", thing=thing)

    async def call(self, method: str, **kwargs):
        request = Request(method=method)
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
        await self.write(self.encode(request))
        future: Future = Future()
        self.futures[request.id] = future
        return future
        

    def recieve(self, response: Response) -> None:
        """
        Receive a request from the server.
        
        Uses the `id` of the response to match it to the corresponding
        request and resolve it's promise.
        
        :param: response The JSON-RPC response as a string or Response instance
        """
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

    def encode(self, request: Request) -> str:
        return json.dumps(request.__dict__)

    def decode(self, message: str) -> Response:
        # Convert the message into a response
        # Currently this only deals with JSON messages but in the furture
        # should handle other message formats
        return Response(**json.loads(message))

    async def read(self, message: str) -> None:
        # Recieve a response message
        self.recieve(self.decode(message))

    async def write(self, message: str) -> None:
        raise NotImplementedError()
