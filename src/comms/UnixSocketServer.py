from typing import Optional
import asyncio
import os
import random
import string
import tempfile

from ..Processor import Processor
from .stencilaFiles import create_tempfile, delete_tempfile
from .StreamMultiServer import StreamMultiServer

class UnixSocketServer(StreamMultiServer):
    """
    A Server communicating over UNIX domain sockets
    """

    _path: Optional[str]

    def __init__(self, processor: Processor):
        StreamMultiServer.__init__(self, processor)
        self._id = 'unix-py-' + ''.join(random.choices(string.ascii_lowercase + string.digits, k=32))
        self._path = None

    @property
    def url(self):
        return f'unix://{self._path}'

    async def open(self) -> None:
        """
        Start the UNIX socket server and create an
        async connections when a client connects.
        """
        self._path = create_tempfile(self._id)
        await asyncio.start_unix_server(self.on_client_connected, self._path)
        # Change the permissions on the file so that no other user can read/write it
        # This needs to be done after the server starts
        os.chmod(self._path, 0o600)

    async def close(self) -> None:
        await StreamMultiServer.close(self)
        delete_tempfile(self._path)
