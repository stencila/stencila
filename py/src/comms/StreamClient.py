from typing import Optional

from .StreamConnection import StreamConnection
from .Client import Client
from .StreamMixin import StreamMixin

class StreamClient(StreamMixin, Client):

    def __init__(self, connection: Optional[StreamConnection] = None, url: str = None, encoders=None, ):
        StreamMixin.__init__(self, connection)
        Client.__init__(self, url=url, encoders=encoders)

    async def open(self) -> None:
        assert self.connection
        self.connection.listen(self.read)
