import sys

from .Server import Server
from .StdioMixin import StdioMixin

class StdioServer(StdioMixin, Server):

    def __init__(self, processor=None, logging=0, input=sys.stdin, output=sys.stdout):
        StdioMixin.__init__(self, input, output)
        Server.__init__(self, processor, logging)
