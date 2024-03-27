from abc import ABC, abstractmethod
from collections.abc import Sequence
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
    """A wrapper to meet Stencila's requirements."""

    instance: str


class Kernel(ABC):
    """
    A base class for implementing a Kernel API for Stencila.

    We provide do-nothing defaults for most methods so that subclasses can
    override only the methods that they need to implement. Implementations
    must provide a `get_name` class method to give the name of the kernel.
    """

    def __init__(self, kernel_id: KernelId):
        self.kernel_id = kernel_id

    @classmethod
    @abstractmethod
    def get_name(cls) -> str:
        """Provide a name for the kernel.

        This is required, and should be the same names that is provided in the
        [[kernels]] section of the stencila_plugin.toml file.
        """

    async def on_start(self):
        """
        Called when Stencila starts an instance of this kernel.

        This is a good place to do any initialization specific to this kernel.
        """

    async def on_stop(self):
        """Call by Stencila when a kernel instance is stopped.

        plugin instance at that time, this method only needs to be implemented
        for plugins that host more than one kernel instance at a time, or that
        need to perform clean up for a stopped kernel instance.
        """

    async def get_info(self) -> SoftwareApplication:
        """Get information about the kernel.

        This is used to provide information about the kernel to the user.
        The default simply provides the name of the kernel. But see the
        SoftwareApplication class for other properties that can be set.
        """
        return SoftwareApplication(name=self.__class__.get_name())

    async def get_packages(self) -> list[SoftwareSourceCode]:
        """
        Get a list of packages available in a kernel instance.

        This method is called by Stencila to obtain a list of packages
        available in a kernel instance. This is used for improving assistant
        code generation (reducing hallucination of packages) and other
        purposes. This method should be implemented by plugins that provide
        kernels which have the concept of installable packages.

        This default implementation returns an empty list.
        """
        return []

    async def execute(self, code: str) -> tuple[Sequence[Node], list[ExecutionMessage]]:
        """
        Execute code in a kernel instance.

        This method is called by Stencila when executing `CodeChunk`s.
        It should be implemented for most kernels.

        This default implementation returns no outputs.
        """
        return [], [
            ExecutionMessage(message="Not implemented", level=MessageLevel.Warning),
        ]

    async def evaluate(
        self, code: str
    ) -> tuple[Sequence[Node], list[ExecutionMessage]]:
        """
        Evaluate code in a kernel instance.

        This method is called by Stencila when evaluating code expressions
        in `CodeExpression`, `ForBlock` and other node types.
        It should be implemented for most kernels.

        This default implementation returns no output.
        """
        return [], [
            ExecutionMessage(message="Not implemented", level=MessageLevel.Warning),
        ]

    async def list_variables(self) -> list[Variable]:
        """
        Get a list of variables available in a kernel instance.

        This method is called by Stencila to obtain a list of variables
        available in a kernel instance. This is used for improving assistant
        code generation (reducing hallucination of variables) and other
        purposes. This method should be implemented by plugins that provide
        kernels which maintain variables as part of the kernel state.

        This default implementation returns an empty list.
        """
        return []

    async def get_variable(self, name: str) -> Variable | None:
        """
        Get a variable from a kernel instance.

        This method is called by Stencila to obtain a variables so it
        can be displayed or "mirrored" to another kernel. This method should
        be implemented by plugins that provide kernels which maintain variables
        as part of the kernel state.

        This default implementation returns `null` (the return value when a
        variable does not exist).
        """
        return None

    async def set_variable(self, name: str, value: Any):
        """
        Set a variable in a kernel instance.

        This method is called by Stencila to set `Parameter` values or
        to "mirror" variable from another kernel. This method should
        be implemented by plugins that provide kernels which maintain variables
        as part of the kernel state.

        This default implementation does nothing.
        """

    async def remove_variable(self, name: str):
        """
        Remove a variable from a kernel instance

        This method is called by Stencila to keep the variables in a kernel
        instance in sync with the variables defined in the code in a document.
        For example, if a `CodeChunk` that declares a variable is removed from
        from the document, then the variable should be removed from the kernel
        (so that it is not accidentally reused later).

        This default implementation does nothing.
        """
