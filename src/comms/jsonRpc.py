from typing import Any, ClassVar, Optional

class Request:

    id:int
    count:ClassVar[int] = 0

    def __init__(self, method, id: Optional[int] = None):
        self.method = method
        if id is None:
            Request.count += 1
            id = Request.count
        self.id = id

class Response:
    
    def __init__(self, id: int, result: Any = None):
        self.id = id
        self.result = result

class Error:
    pass
