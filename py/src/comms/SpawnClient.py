from typing import Dict, Optional, List
import asyncio
import json
import os
import re

from .Client import Client
from .stencilaFiles import get_homedir
from .StreamConnection import StreamConnection
from .StreamClient import StreamClient

SPAWN_URL_REGEX = re.compile(r'^spawn://(.+)')

class SpawnClient(StreamClient):

    _command: str

    _hello: Optional[Dict]
    """
    The result from calling hello() on the server.

    This is stored so that it is possible to get the capabilities of the
    server without having to start it up.
    """

    _subprocess: Optional[asyncio.subprocess.Process]

    def __init__(self, url: str, hello: Dict = None, encoders=None):
        StreamClient.__init__(self, url=url, encoders=encoders)
        match = SPAWN_URL_REGEX.match(url)
        if match:
            self._command = match.group(1)
        else:
            raise RuntimeError(f'Invalid URL for spawn process: {url}')
        self._hello = hello
        self._subprocess = None

    @staticmethod
    def connectable(url: str) -> bool:
        return url[:8] == 'spawn://'

    @staticmethod
    async def discover() -> List[Client]:
        """
        Discover ``SpawnServer`` types registered on this machine.

        :return: List of ``SpawnClient`` instances
        """
        clients: List[Client] = []
        homedir = get_homedir()
        if os.path.exists(homedir):
            for filename in os.listdir(homedir):
                try:
                    with open(os.path.join(homedir, filename)) as file:
                        manifest = json.load(file)
                        command = manifest['command']
                        hello = manifest['hello']
                except Exception as exc:
                    print(exc) # self.log(level='warning', message=str(exc))
                else:
                    client = SpawnClient(
                        url=f'spawn://{command}',
                        hello=hello
                    )
                    clients.append(client)
        return clients

    async def open(self) -> None:
        # Override of open because the actual opening of the connection
        # is lazily defered until first call.
        pass

    async def hello(self) -> Dict:
        # Override of hello to return the stored manifest
        if self._subprocess or not self._hello:
            return await StreamClient.hello(self)
        else:
            return self._hello

    async def call(self, method: str, **kwargs):
        if not self._subprocess:
            # Start subprocess and wait on it
            self._subprocess = await asyncio.create_subprocess_exec(
                *self._command.split(),
                stdin=asyncio.subprocess.PIPE,
                stdout=asyncio.subprocess.PIPE
            )
            asyncio.ensure_future(self._subprocess.wait())

            # Create an async connection to the subprocess
            reader = self._subprocess.stdout
            writer = self._subprocess.stdin
            assert reader and writer
            self.connection = StreamConnection(reader, writer)
            await StreamClient.open(self)

        return await StreamClient.call(self, method, **kwargs)

    async def close(self) -> None:
        await StreamClient.close(self)
        if self._subprocess:
            self._subprocess.terminate()
