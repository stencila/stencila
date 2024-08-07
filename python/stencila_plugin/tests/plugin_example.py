import asyncio

from stencila_types import shortcuts as S
from stencila_types import types as T

from stencila_plugin.kernel import Kernel
from stencila_plugin.model import (
    Model,
    ModelOutput,
    ModelTask,
)
from stencila_plugin.plugin import Plugin


class MyKernel(Kernel):
    @classmethod
    def get_name(cls) -> str:
        return "test"

    async def get_info(self) -> T.SoftwareApplication:
        return T.SoftwareApplication(
            name="MyKernel",
            version="0.1.0",
            abstract=[S.p("A simple kernel for testing")],
            authors=[T.Person(name="Fred Flintstone")],
        )


class MyModel(Model):
    @classmethod
    def get_name(cls) -> str:
        return "test"

    async def perform_task(self, task: ModelTask) -> ModelOutput:
        return ModelOutput(content="hello")


if __name__ == "__main__":
    """This is essential, as we are running the plugin as a script."""
    plugin = Plugin(kernels=[MyKernel], models=[MyModel])
    asyncio.run(plugin.run())
