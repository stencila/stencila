from typing import Optional, Union
import fcntl
import os
import signal

from ..Processor import Processor
from .StreamConnection import StreamConnection
from .JsonEncoder import JsonEncoder
from .jsonRpc import Request
from .StreamClient import StreamClient
from .StreamServer import StreamServer

class CloneClientServer:
    """
    A combined client server for forked processes.

    Currently restricted to JsonEncoders for both client and server to mimimize
    processing time, since connection thoughput is less of an issue.
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
    Either a `StreamClient` or `StreamServer` instance
    """
    worker: Union[StreamClient, StreamServer]

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

            connection = await StreamConnection.from_files(read_file, write_file)
            self.worker = StreamClient(connection)
        else:
            # I am the child server process
            read_file = os.fdopen(nonblocking(server_read_fd), 'r')
            write_file = os.fdopen(nonblocking(client_write_fd), 'w')
            os.close(server_write_fd)
            os.close(client_read_fd)

            connection = await StreamConnection.from_files(read_file, write_file)
            self.worker = StreamServer(self.processor, connection)

        assert self.worker
        await self.worker.start()

    async def execute(self, thing) -> None:
        assert isinstance(self.worker, StreamClient)
        return await self.worker.execute(thing)

    async def stop(self) -> None:
        if self.child_pid:
            await self.worker.stop()
            os.kill(self.child_pid, signal.SIGKILL)
