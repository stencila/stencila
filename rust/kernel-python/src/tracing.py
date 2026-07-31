"""Opt-in runtime dependency tracing for Stencila Python execution.

The wrappers in this module are installed once and remain inert unless
``trace_context`` is active. All tracing failures are deliberately swallowed so
instrumentation cannot change the result of user code.
"""

import builtins
import contextlib
import contextvars
import functools
import hashlib
import importlib
import inspect
import io
import json
import os
import sys
import tempfile
import threading
import urllib.parse
import urllib.request
from collections.abc import Iterator
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

_STATE: contextvars.ContextVar[Any] = contextvars.ContextVar(
    "stencila_runtime_trace", default=None
)
_HTTP_DEPTH = contextvars.ContextVar("stencila_runtime_http_depth", default=0)
_SUPPRESSED = contextvars.ContextVar("stencila_runtime_suppressed", default=False)
_ACTIVE = None
_ACTIVE_LOCK = threading.Lock()
_INSTALLED = False
_AUDIT_INSTALLED = False
_PATCHED = set()


@dataclass
class _TraceState:
    options: dict[str, str]
    events: dict[tuple[str, str, str, int], int] = field(default_factory=dict)
    diagnostics: set[str] = field(default_factory=set)


def _current_state() -> Any:
    if _SUPPRESSED.get():
        return None
    state = _STATE.get()
    if state is not None:
        return state
    with _ACTIVE_LOCK:
        return _ACTIVE


def _location(state: _TraceState) -> tuple[str, int]:
    token = _SUPPRESSED.set(True)
    try:
        fallback = ("", 0)
        for frame in inspect.stack(context=0)[2:]:
            source = frame.filename
            if source == __file__ or "tracing.py" in source:
                continue
            safe_source = (
                _path(source, state) if os.path.isabs(source) else source  # noqa: PTH117
            )
            location = (safe_source, max(frame.lineno - 1, 0))
            fallback = location
            if source.startswith("Code chunk #"):
                return location
            try:
                absolute = os.path.abspath(source)  # noqa: PTH100
                if os.path.commonpath((absolute, sys.prefix)) != sys.prefix:
                    return location
            except (OSError, ValueError):
                return location
        return fallback
    except Exception:
        pass
    finally:
        _SUPPRESSED.reset(token)
    return "", 0


def _record(operation: str, resource: str) -> None:
    try:
        state = _current_state()
        if state is None or not resource:
            return
        source, line = _location(state)
        key = (operation, resource, source, line)
        state.events[key] = state.events.get(key, 0) + 1
        if operation.startswith("file_"):
            state.diagnostics.discard("unconfirmed audit event: open")
        elif operation == "import":
            state.diagnostics.discard("unconfirmed audit event: import")
        elif operation.startswith("remote_"):
            state.diagnostics.discard("unconfirmed audit event: urllib.Request")
            state.diagnostics = {
                item
                for item in state.diagnostics
                if not item.startswith("unconfirmed audit event: socket.")
            }
    except Exception:
        pass


def _diagnostic(message: str) -> None:
    try:
        state = _current_state()
        if state is not None:
            state.diagnostics.add(message)
    except Exception:
        pass


def _path(value: Any, state: Any = None) -> str:
    state = state or _current_state()
    token = _SUPPRESSED.set(True)
    try:
        path = os.path.abspath(os.fsdecode(os.fspath(value)))  # noqa: PTH100
        if state is not None:
            cache_dir = os.path.abspath(state.options["cacheDir"])  # noqa: PTH100
            workspace = os.path.dirname(  # noqa: PTH120
                os.path.dirname(os.path.dirname(cache_dir))  # noqa: PTH120
            )
            try:
                if os.path.commonpath((path, workspace)) == workspace:
                    relative = os.path.relpath(path, workspace).replace(os.sep, "/")
                    return f"workspace:{relative}"
            except ValueError:
                pass
            home = os.path.expanduser("~")  # noqa: PTH111
            try:
                if os.path.commonpath((path, home)) == home:
                    relative = os.path.relpath(path, home).replace(os.sep, "/")
                    return f"home:{relative}"
            except ValueError:
                pass
        return path
    except Exception:
        return ""
    finally:
        _SUPPRESSED.reset(token)


def _url(value: Any) -> str:
    token = _SUPPRESSED.set(True)
    try:
        raw = getattr(value, "full_url", value)
        parts = urllib.parse.urlsplit(str(raw))
        if not parts.scheme:
            return ""
        host = parts.hostname or ""
        if parts.port is not None:
            host = f"{host}:{parts.port}"
        return urllib.parse.urlunsplit((parts.scheme, host, parts.path, "", ""))
    except Exception:
        return ""
    finally:
        _SUPPRESSED.reset(token)


def _request_url(args: tuple[Any, ...], kwargs: dict[str, Any]) -> str:
    value = kwargs.get("url")
    if value is not None:
        return _url(value)
    for value in args:
        sanitized = _url(value)
        if sanitized:
            return sanitized
    return ""


def _import_resource(name: str, module: Any) -> str:
    top_level = name.split(".", 1)[0]
    source = getattr(module, "__file__", "")
    module_path = _path(source) if source else ""

    # A workspace module can intentionally shadow a standard-library name.
    if module_path.startswith("workspace:"):
        return f"{top_level}|{module_path}"

    stdlib_names = getattr(sys, "stdlib_module_names", ())
    if top_level in sys.builtin_module_names or top_level in stdlib_names:
        return ""

    if source:
        try:
            absolute = os.path.abspath(source)  # noqa: PTH100
            parts = Path(absolute).parts
            in_packages = "site-packages" in parts or "dist-packages" in parts
            if (
                not in_packages
                and os.path.commonpath((absolute, sys.base_prefix)) == sys.base_prefix
            ):
                return ""
        except (OSError, TypeError, ValueError):
            pass

    return f"{top_level}|{module_path}" if module_path else top_level


def _mode_operations(mode: Any) -> tuple[str, ...]:
    text = str(mode or "r")
    if "+" in text:
        return ("file_read", "file_write")
    if any(char in text for char in ("w", "a", "x")):
        return ("file_write",)
    return ("file_read",)


def _flags_operations(flags: int) -> tuple[str, ...]:
    if flags & os.O_RDWR:
        return ("file_read", "file_write")
    write_flags = os.O_WRONLY | os.O_APPEND | os.O_CREAT | os.O_TRUNC
    return ("file_write",) if flags & write_flags else ("file_read",)


def _mark_wrapper(wrapper: Any, original: Any) -> Any:
    wrapper.__stencila_runtime_wrapper__ = True
    wrapper.__stencila_runtime_original__ = original
    return wrapper


def _patch_attr(owner: Any, name: str, factory: Any, key: str) -> None:
    try:
        if key in _PATCHED:
            return
        original = getattr(owner, name)
        if getattr(original, "__stencila_runtime_wrapper__", False):
            _PATCHED.add(key)
            return
        setattr(owner, name, _mark_wrapper(factory(original), original))
        _PATCHED.add(key)
    except Exception as error:
        _diagnostic(f"unable to instrument {key}: {type(error).__name__}")


def _sync_http_wrapper(original: Any) -> Any:
    @functools.wraps(original)
    def wrapped(*args: Any, **kwargs: Any) -> Any:
        if _current_state() is None or _HTTP_DEPTH.get() > 0:
            return original(*args, **kwargs)
        token = _HTTP_DEPTH.set(_HTTP_DEPTH.get() + 1)
        try:
            response = original(*args, **kwargs)
        finally:
            _HTTP_DEPTH.reset(token)
        resource = _request_url(args, kwargs)
        _record("remote_receive", resource)
        has_body = any(
            key in kwargs and kwargs[key] is not None
            for key in ("data", "json", "content")
        )
        if has_body:
            _record("remote_send", resource)
        return response

    return wrapped


def _async_http_wrapper(original: Any) -> Any:
    @functools.wraps(original)
    async def wrapped(*args: Any, **kwargs: Any) -> Any:
        if _current_state() is None or _HTTP_DEPTH.get() > 0:
            return await original(*args, **kwargs)
        token = _HTTP_DEPTH.set(_HTTP_DEPTH.get() + 1)
        try:
            response = await original(*args, **kwargs)
        finally:
            _HTTP_DEPTH.reset(token)
        resource = _request_url(args, kwargs)
        _record("remote_receive", resource)
        has_body = any(
            key in kwargs and kwargs[key] is not None
            for key in ("data", "json", "content")
        )
        if has_body:
            _record("remote_send", resource)
        return response

    return wrapped


def _patch_http_clients() -> None:
    with contextlib.suppress(Exception):
        _patch_attr(
            urllib.request,
            "urlopen",
            _sync_http_wrapper,
            "urllib.request.urlopen",
        )

    requests = sys.modules.get("requests")
    if requests is not None:
        sessions = getattr(requests, "sessions", None)
        session = getattr(sessions, "Session", None)
        if session is not None:
            _patch_attr(
                session,
                "request",
                _sync_http_wrapper,
                "requests.Session.request",
            )

    httpx = sys.modules.get("httpx")
    if httpx is not None:
        client = getattr(httpx, "Client", None)
        async_client = getattr(httpx, "AsyncClient", None)
        if client is not None:
            _patch_attr(client, "request", _sync_http_wrapper, "httpx.Client.request")
        if async_client is not None:
            _patch_attr(
                async_client,
                "request",
                _async_http_wrapper,
                "httpx.AsyncClient.request",
            )

    aiohttp = sys.modules.get("aiohttp")
    if aiohttp is not None:
        session = getattr(aiohttp, "ClientSession", None)
        if session is not None:
            _patch_attr(
                session,
                "_request",
                _async_http_wrapper,
                "aiohttp.ClientSession._request",
            )


def install_runtime_tracer() -> None:
    global _INSTALLED, _AUDIT_INSTALLED  # noqa: PLW0603
    if _INSTALLED:
        return
    _INSTALLED = True

    def open_factory(original: Any) -> Any:
        @functools.wraps(original)
        def wrapped(file: Any, mode: Any = "r", *args: Any, **kwargs: Any) -> Any:
            result = original(file, mode, *args, **kwargs)
            if _current_state() is None:
                return result
            path = _path(file)
            for operation in _mode_operations(mode):
                _record(operation, path)
            return result

        return wrapped

    def os_open_factory(original: Any) -> Any:
        @functools.wraps(original)
        def wrapped(path: Any, flags: int, *args: Any, **kwargs: Any) -> Any:
            result = original(path, flags, *args, **kwargs)
            if _current_state() is None:
                return result
            resource = _path(path)
            for operation in _flags_operations(flags):
                _record(operation, resource)
            return result

        return wrapped

    def import_factory(original: Any) -> Any:
        @functools.wraps(original)
        def wrapped(name: str, *args: Any, **kwargs: Any) -> Any:
            module = original(name, *args, **kwargs)
            if _current_state() is None:
                return module
            try:
                imported = sys.modules.get(name) or module
                resource = _import_resource(name, imported)
                _record("import", resource)
                _patch_http_clients()
            except Exception:
                pass
            return module

        return wrapped

    def import_module_factory(original: Any) -> Any:
        @functools.wraps(original)
        def wrapped(name: str, package: Any = None) -> Any:
            module = original(name, package)
            if _current_state() is None:
                return module
            try:
                resource = _import_resource(name, module)
                _record("import", resource)
                _patch_http_clients()
            except Exception:
                pass
            return module

        return wrapped

    _patch_attr(builtins, "open", open_factory, "builtins.open")
    _patch_attr(io, "open", open_factory, "io.open")
    _patch_attr(os, "open", os_open_factory, "os.open")
    _patch_attr(builtins, "__import__", import_factory, "builtins.__import__")
    _patch_attr(
        importlib,
        "import_module",
        import_module_factory,
        "importlib.import_module",
    )
    _patch_http_clients()

    if not _AUDIT_INSTALLED and hasattr(sys, "addaudithook"):
        try:
            def audit(event: str, args: Any) -> None:
                if _current_state() is None:
                    return
                supported = event in ("open", "import", "urllib.Request")
                if supported or event.startswith("socket."):
                    _diagnostic(f"unconfirmed audit event: {event}")

            sys.addaudithook(audit)
            _AUDIT_INSTALLED = True
        except Exception as error:
            _diagnostic(f"unable to install audit hook: {type(error).__name__}")


def _cache_path(options: dict[str, str]) -> Path:
    identity = options.get("identity", "")
    filename = hashlib.sha256(identity.encode("utf-8")).hexdigest() + ".json"
    return Path(options["cacheDir"]) / filename


def _persist(state: _TraceState) -> None:
    try:
        path = _cache_path(state.options)
        path.parent.mkdir(parents=True, exist_ok=True)
        previous: dict[str, Any] = {}
        try:
            with path.open(encoding="utf-8") as stream:
                previous = json.load(stream)
        except Exception:
            pass

        merged: dict[tuple[str, str, str, int], int] = {}
        if (
            previous.get("identity") == state.options.get("identity")
            and previous.get("codeDigest") == state.options.get("codeDigest")
        ):
            for event in previous.get("events", []):
                location = event.get("location", {})
                key = (
                    event.get("operation", ""),
                    event.get("resource", ""),
                    location.get("source", ""),
                    int(location.get("line", 0)),
                )
                merged[key] = merged.get(key, 0) + int(event.get("count", 1))
        for key, count in state.events.items():
            merged[key] = merged.get(key, 0) + count

        events = [
            {
                "operation": operation,
                "resource": resource,
                "location": {"source": source, "line": line},
                "count": count,
            }
            for (operation, resource, source, line), count in sorted(merged.items())
        ]
        diagnostics = sorted(
            set(previous.get("diagnostics", [])) | state.diagnostics
            if previous.get("codeDigest") == state.options.get("codeDigest")
            else state.diagnostics
        )
        payload = {
            "version": 1,
            "identity": state.options.get("identity", ""),
            "codeDigest": state.options.get("codeDigest", ""),
            "events": events,
            "diagnostics": diagnostics,
        }

        fd, temporary = tempfile.mkstemp(
            prefix=".runtime-", suffix=".json", dir=path.parent
        )
        temporary_path = Path(temporary)
        try:
            with os.fdopen(fd, "w", encoding="utf-8") as stream:
                json.dump(payload, stream, separators=(",", ":"), sort_keys=True)
                stream.flush()
                os.fsync(stream.fileno())
            temporary_path.replace(path)
        except Exception:
            with contextlib.suppress(Exception):
                temporary_path.unlink()
            raise
    except Exception:
        pass


@contextlib.contextmanager
def trace_context(options: dict[str, str]) -> Iterator[None]:
    global _ACTIVE  # noqa: PLW0603
    install_runtime_tracer()
    state = _TraceState(options)
    token = _STATE.set(state)
    with _ACTIVE_LOCK:
        _ACTIVE = state
    try:
        yield
    finally:
        _STATE.reset(token)
        with _ACTIVE_LOCK:
            if _ACTIVE is state:
                _ACTIVE = None
        _persist(state)
