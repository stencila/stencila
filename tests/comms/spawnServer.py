from stencilaschema.comms.SpawnServer import SpawnServer

import sys; import os; sys.path.append(os.path.dirname(os.path.dirname(__file__)))
from helpers.processors import PersonProcessor

server = SpawnServer(PersonProcessor())
server.run()
