import pytest
from stencila_types import types as T

from stencila_plugin.kernel import KernelInstance
from stencila_plugin.model import ModelOutput, ModelTask
from stencila_plugin.plugin import Plugin, structure, unstructure
from stencila_plugin.testing import (
    Harness,
    HttpHarness,
    HttpTestingError,
    RPCTestingError,
)


def test_model_structuring():
    task = ModelTask(messages=[])
    x = unstructure(task)
    structure(x, ModelTask)


async def test_model_direct():
    from .plugin_example import MyModel

    plugin = Plugin(models=[MyModel])
    task = ModelTask(messages=[])
    output = await plugin.model_perform_task(
        unstructure(task),
        MyModel.get_name(),
    )
    assert isinstance(output, ModelOutput)


async def test_authentication_token_works(http_harness: HttpHarness):
    # Mess with the headers
    http_harness.headers = {"Authorization": "Bearer xxx"}
    with pytest.raises(HttpTestingError):
        await http_harness.send_rpc("health")
    http_harness.headers = {}
    with pytest.raises(HttpTestingError):
        await http_harness.send_rpc("health")


async def test_health(stdio_harness: Harness):
    res = await stdio_harness.send_rpc("health")
    assert isinstance(res, dict)
    assert res["status"] == "OK"


async def test_bad_json(stdio_harness: Harness):
    with pytest.raises(RPCTestingError):
        await stdio_harness.send_raw({"x": 1})


async def test_kernel_rpc(stdio_harness: Harness):
    result = await stdio_harness.send_rpc("kernel_start", kernel="test")
    assert result is not None
    ki = structure(result, KernelInstance)

    result = await stdio_harness.send_rpc("kernel_info", instance=ki.instance)
    structure(result, T.SoftwareApplication)

    result = await stdio_harness.send_rpc("kernel_packages", instance=ki.instance)
    structure(result, list[T.SoftwareSourceCode])

    await stdio_harness.send_rpc("kernel_stop", instance=ki.instance)


async def test_model_rpc(stdio_harness: Harness):
    task = unstructure(
        ModelTask(
            messages=[T.InstructionMessage(parts=[T.Text(value="hello")])],
        )
    )
    res = await stdio_harness.send_rpc("model_perform_task", task=task, model="test")
    output = structure(res, ModelOutput)
    assert isinstance(output, ModelOutput)
