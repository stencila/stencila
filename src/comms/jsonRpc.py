from typing import Any, ClassVar, Dict, List, Optional, Union

class Request:

    id: int

    count: ClassVar[int] = 0

    method: str

    params: Union[Dict[str, Any], List[Any]]

    def __init__(self, method=None, params=None, id: Optional[int] = None):
        self.jsonrpc = '2.0'
        self.method = method
        if id is None:
            Request.count += 1
            id = Request.count
        self.id = id
        self.params = params

class Response:

    def __init__(self, id: int = None, result: Any = None, error: Any = None):
        self.jsonrpc = '2.0'
        self.id = id
        self.result = result
        self.error = error

class Error:
    pass
