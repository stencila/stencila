from typing import Optional, Union
import fcntl
import os
import signal

from ..Processor import Processor
from .AsyncioConnection import AsyncioConnection
from .Client import Client
from .Server import Server

class CloneClientServer:
    """
    A combined client server for forked processes.
    """

    """
    The processor that will be cloned in the
    server process
    """
    processor: Processor

    """
    The id of the child server process
    """
    child_pid: Optional[int]

    """
    Either a `Client` or `Server` instance
    """
    worker: Union[Client, Server]

    def __init__(self, processor):
        self.processor = processor

    @property
    def is_client(self) -> bool:
        return self.child_pid != 0

    async def start(self) -> None:
        # Create two pipes for bi-directional communication
        client_read_fd, client_write_fd = os.pipe()
        server_read_fd, server_write_fd = os.pipe()

        # Make a pipe file descriptor non-blocking
        def nonblocking(fd):
            fl = fcntl.fcntl(fd, fcntl.F_GETFL)
            fcntl.fcntl(fd, fcntl.F_SETFL, fl | os.O_NONBLOCK)
            return fd

        # Create a clone child process using system `fork`
        self.child_pid = os.fork()
        if self.child_pid:
            # I am the parent client process
            read_file = os.fdopen(nonblocking(client_read_fd), 'r')
            write_file = os.fdopen(nonblocking(server_write_fd), 'w')
            os.close(client_write_fd)
            os.close(server_read_fd)

            connection = await AsyncioConnection.from_files(read_file, write_file)
            self.worker = CloneClient(connection)
        else:
            # I am the child server process
            read_file = os.fdopen(nonblocking(server_read_fd), 'r')
            write_file = os.fdopen(nonblocking(client_write_fd), 'w')
            os.close(server_write_fd)
            os.close(client_read_fd)

            connection = await AsyncioConnection.from_files(read_file, write_file)
            self.worker = CloneServer(self.processor, connection)

        assert self.worker
        await self.worker.start()

    async def execute(self, thing) -> None:
        assert isinstance(self.worker, Client)
        return await self.worker.execute(thing)

    async def stop(self) -> None:
        if self.child_pid:
            await self.worker.stop()
            os.kill(self.child_pid, signal.SIGKILL)

class CloneMixin:

    def __init__(self, connection):
        self.connection = connection

    async def write(self, message: str) -> None:
        assert self.connection
        await self.connection.write(message)

    async def close(self) -> None:
        if self.connection:
            await self.connection.close()

class CloneClient(CloneMixin, Client):

    async def open(self) -> None:
        self.connection.listen(self.read)


class CloneServer(CloneMixin, Server):

    def __init__(self, processor, connection):
        CloneMixin.__init__(self, connection)
        Server.__init__(self, processor)

    async def open(self) -> None:
        async def callback(message):
            await self.connection.write(await self.receive(message))
        self.connection.listen(callback)
