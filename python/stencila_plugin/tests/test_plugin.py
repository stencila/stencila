import pytest
from stencila_types import types as T

from stencila_plugin.kernel import KernelInstance
from stencila_plugin.testing import (
    Harness,
    HttpHarness,
    HttpTestingError,
    RPCTestingError,
)


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
    ki = KernelInstance(**result)

    result = await harness.send_rpc("kernel_info", instance=ki.instance)

    # Will throw if it cannot reconstruct it.
    T.SoftwareApplication(**result)

    result = await harness.send_rpc("kernel_packages", instance=ki.instance)
    [T.SoftwareSourceCode(**dct) for dct in result]

    await harness.send_rpc("kernel_stop", instance=ki.instance)


async def test_kernel_invoke(harness: Harness):
    """Try out harness methods that reconstruct the result."""
    ki = await harness.send_rpc("kernel_start", kernel="test")
    instance = ki.get("instance")

    result = await harness.invoke("kernel_info", instance=instance)
    assert isinstance(result, T.SoftwareApplication)
    assert result.authors[0].name.startswith("Fred")
