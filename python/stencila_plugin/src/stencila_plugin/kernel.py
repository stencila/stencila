from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Any, TypeAlias

from stencila_types.types import (
    ExecutionMessage,
    MessageLevel,
    Node,
    SoftwareApplication,
    SoftwareSourceCode,
    Variable,
)

KernelId: TypeAlias = str
KernelName: TypeAlias = str


@dataclass
class KernelInstance:
    instance: str


class Kernel(ABC):
    """
    Base class for kernels.

    We provide do-nothing defaults for all methods so that subclasses can
    override only the methods that they need to implement.
    """

    def __init__(self, ident: KernelId):
        self.ident = ident

    @classmethod
    @abstractmethod
    def get_name(cls) -> str: ...

    async def on_start(self): ...
    async def on_stop(self): ...

    async def get_info(self) -> SoftwareApplication:
        return SoftwareApplication(name=self.__class__.get_name())

    async def get_packages(self) -> list[SoftwareSourceCode]:
        return []

    async def execute(self, code: str) -> tuple[list[Node], list[ExecutionMessage]]:
        return [], [
            ExecutionMessage(message="Not implemented", level=MessageLevel.Warning),
        ]

    async def evaluate(self, code: str) -> tuple[list[Node], list[ExecutionMessage]]:
        return [], [
            ExecutionMessage(message="Not implemented", level=MessageLevel.Warning),
        ]

    async def list_variables(self) -> list[Variable]:
        return []

    async def get_variable(self, name: str) -> Variable | None:
        return None

    async def set_variable(self, name: str, value: Any): ...
    async def remove_variable(self, name: str): ...
