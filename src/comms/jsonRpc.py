from typing import Any, ClassVar, Dict, List, Optional, Type, TypeVar, Union

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

    # Extract a parameter by name from Object or by index from Array
    def param(self, index: int, name: str, required: bool = True):
        if not self.params:
            raise Error.invalid_request('missing "params" property')
        if isinstance(self.params, list):
            try:
                return self.params[index]
            except IndexError:
                if required:
                    raise Error.invalid_params(f'param {index} is missing')
        else:
            try:
                return self.params[name]
            except KeyError:
                if required:
                    raise Error.invalid_params(f'"{name}" is missing')


class Response:

    def __init__(self, id: int = None, result: Any = None, error: Any = None):
        self.jsonrpc = '2.0'
        self.id = id
        self.result = result
        self.error = error

class Error(RuntimeError):
    """
    A JSON-RPC 2.0 [error object](https://www.jsonrpc.org/specification#error_object).

    This class extends `RuntimeError` so that it can be `raise`d.
    It provides several static methods for convieniently creating errors without
    having to remember error codes.
    """

    code: int
    """
    A mumber that indicates the error type that occurred.
    This MUST be an integer.
    """

    message: str
    """
    A string providing a short description of the error.
    The message SHOULD be limited to a concise single sentence.
    """

    data: Optional[Any]
    """
    A primitive or structured value that contains additional information about the error.
    This may be omitted.
    The value of this member is defined by the Server (e.g. detailed error information,
    nested errors etc.).
    """

    def __init__(self, code: int, message: str, data: Optional[Any] = None):
        RuntimeError.__init__(self, message)
        self.code = code
        self.message = message
        self.data = data

    @staticmethod
    def parse_error(message: str, data: Optional[Any] = None) -> 'Error':
        return Error(-32700, 'Error parsing request: ' + message, data)

    @staticmethod
    def invalid_request(message: str, data: Optional[Any] = None) -> 'Error':
        return Error(-32600, 'Error, invalid request: ' + message, data)

    @staticmethod
    def method_not_found(message: str, data: Optional[Any] = None) -> 'Error':
        return Error(-32601, 'Error, method not found: ' + message, data)

    @staticmethod
    def invalid_params(message: str, data: Optional[Any] = None) -> 'Error':
        return Error(-32603, 'Error, invalid params: ' + message, data)

    @staticmethod
    def application_error(message: str, data: Optional[Any] = None) -> 'Error':
        return Error(-32700, 'Error in application: ' + message, data)

RequestOrResponse = TypeVar('RequestOrResponse', Request, Response)
