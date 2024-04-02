import pytest
from stencila_types import shortcuts as S
from stencila_types import types as T

from stencila_plugin.assistant import GenerateOptions, GenerateOutput, GenerateTask
from stencila_plugin.kernel import KernelInstance
from stencila_plugin.plugin import Plugin, structure, unstructure
from stencila_plugin.testing import (
    Harness,
    HttpHarness,
    HttpTestingError,
    RPCTestingError,
)


def test_structuring():
    task = GenerateTask(
        instruction=T.InstructionBlock(
            messages=[],
            content=[S.p("hello")],
            suggestion=T.InsertBlock(content=[S.p("something")]),
        ),
        instruction_text="hello",
        format="markdown",
        content_formatted="",
    )
    x = unstructure(task)
    structure(x, GenerateTask)


async def test_assistant_direct():
    from .plugin_example import MyAssistant, MyKernel

    plugin = Plugin(kernels=[MyKernel], assistants=[MyAssistant])
    task = GenerateTask(
        instruction=T.InstructionBlock(
            messages=[],
            content=[S.p("hello")],
            suggestion=T.InsertBlock(content=[S.p("something")]),
        ),
        instruction_text="hello",
        format="markdown",
        content_formatted="",
    )
    options = GenerateOptions()
    output = await plugin.assistant_perform_task(
        unstructure(task),
        unstructure(options),
        MyAssistant.get_name(),
    )
    assert isinstance(output, GenerateOutput)


async def test_authentication_token_works(http_harness: HttpHarness):
    # Mess with the headers
    http_harness.headers = {"Authorization": "Bearer xxx"}
    with pytest.raises(HttpTestingError):
        await http_harness.send_rpc("health")
    http_harness.headers = {}
    with pytest.raises(HttpTestingError):
        await http_harness.send_rpc("health")


async def test_health(harness: Harness):
    res = await harness.send_rpc("health")
    assert isinstance(res, dict)
    assert res["status"] == "OK"


async def test_bad_json(harness: Harness):
    with pytest.raises(RPCTestingError):
        await harness.send_raw({"x": 1})


async def test_kernel_rpc(harness: Harness):
    result = await harness.send_rpc("kernel_start", kernel="test")
    assert result is not None
    ki = structure(result, KernelInstance)

    result = await harness.send_rpc("kernel_info", instance=ki.instance)
    structure(result, T.SoftwareApplication)

    result = await harness.send_rpc("kernel_packages", instance=ki.instance)
    structure(result, list[T.SoftwareSourceCode])

    await harness.send_rpc("kernel_stop", instance=ki.instance)


async def test_assistant(stdio_harness: Harness):
    task = unstructure(
        GenerateTask(
            instruction=T.InstructionBlock(
                messages=[T.InstructionMessage(parts=[T.Text(value="hello")])]
            ),
            instruction_text="hello",
            format="markdown",
        )
    )
    await stdio_harness.send_rpc(
        "assistant_system_prompt", task=task, options={}, assistant="test"
    )
    res = await stdio_harness.send_rpc(
        "assistant_perform_task", task=task, options={}, assistant="test"
    )
    output = structure(res, GenerateOutput)
    assert isinstance(output, GenerateOutput)
