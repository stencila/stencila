import pytest

from stencilaschema.comms.StdioClient import StdioClient
from stencilaschema.comms.StdioServer import StdioServer

from helpers.processors import TestProcessor

# It is difficult to test StdIO client/servers under pytest
# because if captures stdin and stdout. Even turning it of
# using the `capsys` fixture still causes issues
# So see integration tests for tests that connect the client and server

def test_client():
    client = StdioClient([])
    assert client.url == None

def test_server():
    server = StdioServer(TestProcessor())
    assert server.url == None
