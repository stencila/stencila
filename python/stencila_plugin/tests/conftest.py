from pathlib import Path

import pytest

from stencila_plugin.testing import HttpHarness, StdioHarness


@pytest.fixture()
def plugin_path():
    """Provide the path to the plugin.

    Here we use the plugin_example.py file in the testing folder.
    In a real plugin, this would be the path to the plugin's main file.
    """
    path = Path(__file__).parent / "plugin_example.py"
    assert path.exists()
    return path


@pytest.fixture()
async def stdio_harness(plugin_path: Path):
    async with StdioHarness(plugin_path) as harness:
        yield harness


@pytest.fixture()
async def http_harness(plugin_path: Path):
    async with HttpHarness(plugin_path) as harness:
        yield harness


@pytest.fixture(params=["stdio_harness", "http_harness"])
def harness(request):
    """Roll up both harnesses together."""
    return request.getfixturevalue(request.param)
