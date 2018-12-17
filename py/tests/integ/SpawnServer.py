#!/usr/bin/env python3

from stencilaschema.comms.SpawnServer import SpawnServer

import sys; import os; sys.path.append(os.path.dirname(os.path.dirname(__file__)))
from helpers.processors import TestProcessor

server = SpawnServer(TestProcessor())
server.run()
