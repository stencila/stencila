from typing import Any, List
import asyncio

from ..Processor import Processor
from .jsonRpc import Request
from .StreamConnection import StreamConnection
from .Server import Server

class StreamMultiServerConnection(StreamConnection):

    def __init__(self, reader, writer, encoding):
        StreamConnection.__init__(self, reader, writer)
        self.encoding = encoding

class StreamMultiServer(Server):

    connections: List[StreamMultiServerConnection]
    """
    List of client connections and the encodings to use
    for each.
    """

    def __init__(self, processor: Processor, encoders=None):
        Server.__init__(self, processor=processor, encoders=encoders)
        
        self.connections = []

    def on_client_connected(self, reader, writer):
        self.log(connection=True)
        index = len(self.connections)
        connection = StreamMultiServerConnection(reader, writer, 'json')
        async def callback(message):
            await connection.write(await self.receive(message, connection.encoding, connection))
        connection.listen(callback)
        self.connections.append(connection)

    async def hello(self, request: Request, connection: Any = None) -> Any:
        """
        Override the base ``hello`` method to intercept a 
        message to change the encoding used.

        :param request: The incoming client request
        :type request: Request
        :return: The result of the call to ``hello``
        :rtype: Any
        """

        result = await Server.hello(self, request, connection)
        encoding = result.get('encoding')
        if encoding:
            assert isinstance(connection, StreamMultiServerConnection)
            connection.encoding = encoding
        return result

    async def close(self) -> None:
        """
        Close all the client connections
        """
        for connection in self.connections:
            await connection.close()
