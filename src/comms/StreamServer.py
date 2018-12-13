from typing import Any, Optional

from .StreamConnection import StreamConnection
from .jsonRpc import Request
from .Server import Server
from .StreamMixin import StreamMixin

class StreamServer(StreamMixin, Server):

    encoding: str
    """
    The encoding used for the connection.
    """

    def __init__(self, processor, connection: Optional[StreamConnection] = None):
        Server.__init__(self, processor)
        StreamMixin.__init__(self, connection)

        self.encoding = 'json'

    async def open(self) -> None:
        assert self.connection
        async def callback(message):
            await self.connection.write(await self.receive(message, self.encoding))
        self.connection.listen(callback)

    async def hello(self, request: Request, connection: Any = None) -> Any:
        """
        Override the base ``hello`` method to intercept a 
        message to change the encoding used.

        :param request: The incoming client request
        :type request: Request
        :return: The result of the call to ``hello``
        :rtype: Any
        """

        result = await Server.hello(self, request)
        encoding = result.get('encoding')
        if encoding:
            self.encoding = encoding
        return result
