from typing import List
import asyncio
import re

from .Client import Client
from .StreamConnection import StreamConnection
from .StreamClient import StreamClient

TCP_URL_REGEX = re.compile(r'^tcp://([^:/]+)(?:\:(\d+))?')

class TcpClient(StreamClient):

    def __init__(self, url: str = 'tcp://127.0.0.1', encoders=None):
        StreamClient.__init__(self, url=url, encoders=encoders)
        match = TCP_URL_REGEX.match(url)
        if match:
            self._host = match.group(1)
            self._port = int(match.group(2)) if match.group(2) else 2000
        else:
            raise RuntimeError(f'Invalid URL for TCP: {url}')

    @staticmethod
    def connectable(url: str) -> bool:
        return url[:6] == 'tcp://'

    @staticmethod
    async def discover() -> List[Client]:
        """
        Discover `TcpServers`.

        Currently this is a naive implementation which scan a limited number
        of ports on localhost. It is nonetheless, useful for testings.

        Future implementations, may use a service discovery approach e.g mDNS, Consul
        
        :raises exc: Any unhandled exception when attepting to scan a port
        :return: List of ``TcpClients``
        """

        clients: List[Client] = []
        for port in range(2000, 2010):
            client = TcpClient(f'tcp://127.0.0.1:{port}')
            try:
                await client.start()
            except OSError as exc:
                if exc.errno not in (
                    111, # "Connect call failed"
                ):
                    raise exc
            else:
                clients.append(client)
        return clients

    async def open(self) -> None:
        reader, writer = await asyncio.open_connection(self._host, self._port)
        self.connection = StreamConnection(reader, writer)
        await StreamClient.open(self)
