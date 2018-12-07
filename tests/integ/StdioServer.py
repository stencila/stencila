#!/usr/bin/env python3

from stencilaschema.comms.StdioServer import StdioServer

import sys; import os; sys.path.append(os.path.dirname(os.path.dirname(__file__)))
from helpers.TestProcessor import TestProcessor

server = StdioServer(TestProcessor())
server.run()
