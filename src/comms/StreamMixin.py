from typing import Optional

from .StreamConnection import StreamConnection

class StreamMixin:

    connection: Optional[StreamConnection] = None

    def __init__(self, connection: Optional[StreamConnection] = None):
        self.connection = connection

    @property
    def url(self) -> str:
        # Currently we only use standard I/O pipes, but in the future
        # may provide for named pipes
        return 'pipe://stdio'

    async def write(self, message: bytes) -> None:
        assert self.connection
        await self.connection.write(message)

    async def close(self) -> None:
        if self.connection:
            await self.connection.close()
