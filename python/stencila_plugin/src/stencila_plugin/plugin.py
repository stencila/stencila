import asyncio
import json
import os
import sys
import time
import uuid
from collections.abc import Sequence
from typing import Any

from aiohttp import web
from stencila_types import types as T
from stencila_types.types import (
    ExecutionMessage,
    MessageLevel,
    Node,
    SoftwareApplication,
    SoftwareSourceCode,
    Variable,
)
from stencila_types.utilities import from_value, make_stencila_converter

from .assistant import (
    Assistant,
    AssistantId,
    GenerateOptions,
    GenerateOutput,
    GenerateTask,
)
from .kernel import Kernel, KernelId, KernelInstance, KernelName

# https://github.com/kevinheavey/jsonalias
Json = dict[str, "Json"] | list["Json"] | str | int | float | bool | None
JsonDict = dict[str, Json]

# According to the JSON-RPC spec, the id can be a string, integer, or null.
IdType = str | int | None
ParamsType = list | dict | None


def make_rpc_converter():
    """
    Add some additional hooks for local unions that are NOT in the
    stencila_types module.

    SO AWKWARD!
    """
    converter = make_stencila_converter()
    converter.register_structure_hook(
        T.InstructionBlock | T.InstructionInline, lambda o, _: from_value(o)
    )
    return converter


CONVERTER = make_rpc_converter()
structure = CONVERTER.structure
unstructure = CONVERTER.unstructure


# TODO: We should really raise exceptions in the python code.
class RPCErrorCodes:
    """JSON-RPC error codes.

    See https://www.jsonrpc.org/specification
    """

    PARSE_ERROR = -32700
    INVALID_REQUEST = -32600
    METHOD_NOT_FOUND = -32601
    INVALID_PARAMS = -32602
    INTERNAL_ERROR = -32603


def _success(msg_id: IdType, result: Json) -> Json:  # noqa: A002
    return {
        "jsonrpc": "2.0",
        "id": msg_id,
        "result": result,
    }


def _error(msg_id: IdType, code: int, message: str) -> Json:  # noqa: A002
    return {
        "jsonrpc": "2.0",
        "error": {"code": code, "message": message},
        "id": msg_id,
    }


# @beartype
class Plugin:
    """A Stencila plugin.

    This routes the requests to the Kernel instances (and other APIs that are coming).
    """

    def __init__(
        self,
        kernels: list[type[Kernel]] | None = None,
        assistants: list[type[Assistant]] | None = None,
    ):
        kernels = kernels or []
        self.kernels: dict[KernelName, type[Kernel]] = {
            k.get_name(): k for k in kernels
        }
        self.kernel_instances: dict[KernelId, Kernel] = {}

        # These are created per assistant (unlike kernels).
        self.assistants: dict[AssistantId, Assistant] = (
            {cls.get_name(): cls() for cls in assistants} if assistants else {}
        )

    async def health(self) -> Json:
        """Get the health of the plugin.

        At present this method is only used to check communication with the
        plugin. In the future, the expected response object may be used for
        more detailed statistics about resource usage etc by the plugin.
        """
        return {
            "timestamp": int(time.time()),
            "status": "OK",
        }

    async def kernel_start(self, kernel: KernelName) -> KernelInstance | None:
        kernel_cls = self.kernels.get(kernel)
        if kernel_cls is None:
            return None

        uid = uuid.uuid4()
        kernel_id = f"{kernel}-{uid}"
        instance = kernel_cls(kernel_id)
        self.kernel_instances[kernel_id] = instance
        await instance.on_start()

        return KernelInstance(kernel_id)

    async def kernel_stop(self, instance: KernelId):
        kernel = self.kernel_instances.pop(instance, None)
        if kernel:
            await kernel.on_stop()

    async def kernel_info(self, instance: KernelId) -> SoftwareApplication | None:
        kernel = self.kernel_instances.get(instance)
        if kernel:
            return await kernel.get_info()
        return None

    async def kernel_packages(self, instance: str) -> list[SoftwareSourceCode]:
        kernel = self.kernel_instances.get(instance)
        if kernel:
            return await kernel.get_packages()
        return []

    async def kernel_execute(
        self, code: str, instance: str
    ) -> tuple[Sequence[Node], list[ExecutionMessage]]:
        kernel = self.kernel_instances.get(instance)
        if kernel:
            return await kernel.execute(code)
        return [], [
            ExecutionMessage(message="Kernel not found", level=MessageLevel.Error)
        ]

    async def kernel_evaluate(
        self, code: str, instance: str
    ) -> tuple[Sequence[Node], list[ExecutionMessage]]:
        kernel = self.kernel_instances.get(instance)
        if kernel:
            return await kernel.evaluate(code)
        return [], [
            ExecutionMessage(message="Kernel not found", level=MessageLevel.Error)
        ]

    async def kernel_list(self, instance: str) -> list[Variable]:
        kernel = self.kernel_instances.get(instance)
        if kernel:
            return await kernel.list_variables()
        return []

    async def kernel_get(self, name: str, instance: str) -> Variable | None:
        kernel = self.kernel_instances.get(instance)
        if kernel:
            return await kernel.get_variable(name)
        return None

    async def kernel_set(self, name: str, value: Any, instance: str):
        kernel = self.kernel_instances.get(instance)
        if kernel:
            await kernel.set_variable(name, value)

    async def kernel_remove(self, name: str, instance: str):
        kernel = self.kernel_instances.get(instance)
        if kernel:
            await kernel.remove_variable(name)

        kernel = self.kernel_instances.get(instance)
        if kernel:
            await kernel.remove_variable(name)

    async def assistant_system_prompt(
        self, task: GenerateTask, options: GenerateOptions, assistant: AssistantId
    ) -> str | None:
        instance = self.assistants.get(assistant)
        # Error?
        if instance is None:
            return None
        task = structure(task, GenerateTask)
        options = structure(options, GenerateOptions)
        return await instance.system_prompt(task, options)

    async def assistant_perform_task(
        self, task: dict[str, Any], options: dict[str, Any], assistant: AssistantId
    ) -> GenerateOutput:
        instance = self.assistants.get(assistant)
        if instance is None:
            # TODO: Unclear how errors are handled here.
            return None

        task = structure(task, GenerateTask)
        options = structure(options, GenerateOptions)
        return await instance.perform_task(task, options)

    async def run(self) -> None:
        """Invoke the plugin.

        This method should be called by the plugin's `__main__` module.
        """
        protocol = os.environ.get("STENCILA_TRANSPORT")
        if protocol == "stdio":
            await _listen_stdio(self)
        elif protocol == "http":
            port = int(os.environ.get("STENCILA_PORT", "0"))
            token = os.environ.get("STENCILA_TOKEN", "")
            await _listen_http(self, port, token)
        else:
            raise RuntimeError(f"Unknown protocol: {protocol}")


async def _handle_json(
    plugin: Plugin,
    request: JsonDict,
) -> Json:
    """Interpret a JSON-RPC request and return a response.

    See https://www.jsonrpc.org/specification
    """
    rpc_version = request.get("jsonrpc")
    if rpc_version != "2.0":
        return _error(
            None, RPCErrorCodes.INVALID_REQUEST, "Invalid or missing JSON-RPC version"
        )

    method = request.get("method")
    if method is None:
        return _error(None, RPCErrorCodes.METHOD_NOT_FOUND, "No method sent")

    # This can be None
    msg_id: IdType = request.get("id")  # type: ignore

    # According to the standard, the params can be an Array or an Object (a dict).
    # We also handle None.
    params = request.get("params")

    if not isinstance(params, dict):
        return _error(None, RPCErrorCodes.INVALID_PARAMS, "")

    # Hm. Still struggling with typing here.
    return await _handle_rpc(plugin, method, params=params, msg_id=msg_id)  # type: ignore


async def _handle_rpc(
    plugin: Plugin,
    method: str,
    *,
    params: ParamsType,
    msg_id: IdType = None,
) -> Json:
    """Forward the RPC request to a method and return the result."""
    if params is None:
        args = []
        kwargs = {}
    elif isinstance(params, list):
        # Note: Stencila should send named parameters.
        # This is here for completeness.
        args = params
        kwargs = {}
    elif isinstance(params, dict):
        args = []
        kwargs = params
    else:
        return _error(
            msg_id, RPCErrorCodes.INVALID_PARAMS, "Params are not Array or Object"
        )

    func = getattr(plugin, method, None)
    if callable(func):
        try:
            result = await func(*args, **kwargs)
            try:
                json_result = unstructure(result)
            except Exception as e:
                return _error(
                    msg_id,
                    RPCErrorCodes.INTERNAL_ERROR,
                    f"Cannot convert result to JSON {e}",
                )
            return _success(msg_id, json_result)
        except Exception as e:
            return _error(msg_id, RPCErrorCodes.INTERNAL_ERROR, f"Internal error: {e}")
    else:
        return _error(
            msg_id, RPCErrorCodes.METHOD_NOT_FOUND, f"Method `{method}` not found"
        )


async def _listen_stdio(plugin: Plugin) -> None:
    reader = asyncio.StreamReader()
    reader_protocol = asyncio.StreamReaderProtocol(reader)
    await asyncio.get_running_loop().connect_read_pipe(
        lambda: reader_protocol, sys.stdin
    )

    (
        writer_transport,
        writer_protocol,
    ) = await asyncio.get_running_loop().connect_write_pipe(
        lambda: asyncio.streams.FlowControlMixin(), sys.stdout
    )
    writer = asyncio.StreamWriter(
        writer_transport, writer_protocol, None, asyncio.get_running_loop()
    )

    while True:
        line = await reader.readline()
        # Ignore empty line.
        if line == b"\n":
            continue

        resp: Json
        try:
            request: Json = json.loads(line.decode())
            if not isinstance(request, dict):
                # We need an Object, not scalar
                raise json.JSONDecodeError("Not a JSON object", "", 0)
        except (json.JSONDecodeError, UnicodeDecodeError):
            resp = _error(None, RPCErrorCodes.PARSE_ERROR, "Parse error")
        else:
            resp = await _handle_json(plugin, request)

        writer.write(json.dumps(resp).encode())
        writer.write(b"\n")
        await writer.drain()


async def _listen_http(plugin: Plugin, port: int, token: str) -> None:
    async def handler(request: web.Request) -> web.Response:
        # SECURITY
        # Only accept requests from localhost
        if request.remote not in ("127.0.0.1", "::1"):
            raise web.HTTPForbidden(reason="Local access only")

        # Check if the token is present and matches the expected value
        auth_header = request.headers.get("Authorization", "")
        received_token = (
            auth_header.split(" ")[1] if auth_header.startswith("Bearer ") else None
        )
        if received_token != token:
            raise web.HTTPUnauthorized(reason="Invalid or missing token")

        resp: Json
        try:
            req_json = await request.json()
        except json.JSONDecodeError:
            resp = _error(None, RPCErrorCodes.PARSE_ERROR, "Cannot parse JSON")
        else:
            resp = await _handle_json(plugin, req_json)

        return web.Response(text=json.dumps(resp), content_type="application/json")

    app = web.Application()
    app.add_routes([web.post("/", handler)])
    runner = web.AppRunner(app)
    await runner.setup()
    site = web.TCPSite(runner, "localhost", port)
    await site.start()

    # Now just serve forever; till you are killed.
    try:  # noqa: SIM105
        await asyncio.Event().wait()
    except KeyboardInterrupt:
        pass  # Allow graceful exit on Ctrl+C
