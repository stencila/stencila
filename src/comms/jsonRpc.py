from typing import Any, ClassVar, Optional

class Request:

    id:int
    count:ClassVar[int] = 0

    def __init__(self, method = None, id: Optional[int] = None):
        self.jsonrpc = '2.0'
        self.method = method
        if id is None:
            Request.count += 1
            id = Request.count
        self.id = id

class Response:
    
    def __init__(self, id: int = None, result: Any = None):
        self.jsonrpc = '2.0'
        self.id = id
        self.result = result

class Error:
    pass
