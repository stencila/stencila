import json
import sys
import time

class LogEncoder(json.JSONEncoder):

    def default(self, o):
        return o.__dict__

class Logger:

    def log(self, **kwargs) -> None:
        kwargs['timestamp'] = time.time()
        kwargs['class'] = self.__class__.__name__
        kwargs['id'] = hex(id(self))
        sys.stderr.write(json.dumps(kwargs, cls=LogEncoder) + '\n')
