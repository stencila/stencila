from typing import Optional

from .AsyncioConnection import AsyncioConnection

class StdioMixin:

    connection: Optional[AsyncioConnection] = None

    @property
    def url(self):
        # Currently we only use standard I/O pipes, but in the future
        # may provide for named pipes
        return 'pipe://stdio'
