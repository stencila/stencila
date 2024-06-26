from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from typing import Any, Literal, TypeAlias

from stencila_types.types import InstructionBlock, InstructionInline, Node

AssistantId: TypeAlias = str
GenerateOptions: TypeAlias = dict[str, Any]


@dataclass(kw_only=True)
class GenerateTask:
    """
    An assistant generation task
    """

    # The instruction to be executed
    instruction: InstructionBlock | InstructionInline

    # The aggregated text of the messages in the instruction
    # Available individually in the `instruction.messages` but provided here,
    # joined into a single string, for convenience.
    instruction_text: str

    # The content of the instruction, if any, formatted using the
    # `content-format` specified for the assistant in `stencila-plugin.toml`
    content_formatted: str | None = None

    # The context of the instruction
    # This is available to assistants so that they can tailor their responses
    # given the broader context of the document that the instruction is within.
    context: dict[str, Any] | None = field(default_factory=dict)

    # The input type of the task
    input: Literal["text", "audio", "image", "video"] = "text"

    # The output type of the task
    output: Literal["text", "audio", "image", "video"] = "text"

    # The desired output format of the task
    format: str

    # The rendered system prompt of the assistant
    # If a `system-prompt` is specified for the assistant in
    # `stencila-plugin.toml` then this will rendered using the task itself as
    # the content before calling the plugin.
    system_prompt: str | None = None


@dataclass(kw_only=True)
class GenerateOutput:
    """
    The output generated by an assistant
    """

    # The kind of the generated content
    #
    # Used by Stencila to determine how to handle the `content` before decoding
    # it into nodes
    kind: Literal["text", "url"] | None = None

    # The format of the generated content
    # Used by Stencila to decode the generated `content` into a set of Stencila
    # Schema nodes.
    format: str | None = None

    # The content generated by the assistant
    content: str | None = None

    # The nodes generated by the assistant
    #
    # If supplied then `format`, `kind` and `content` will be ignored and
    # `nodes` will be used directly.
    nodes: list[Node] | None = None


class Assistant(ABC):
    """
    A base class for implementing the Assistant API for Stencila.
    """

    @classmethod
    @abstractmethod
    def get_name(cls) -> str:
        """Provide a name for the assistant.

        This is required, and should be the same name that is provided in the
        [[assistants]] section of the stencila_plugin.toml file.
        """

    async def system_prompt(
        self, task: GenerateTask, options: GenerateOptions
    ) -> str | None:
        """
        Prepare the system prompt for an assistant

        This method is called by Stencila before executing an
        `InstructionBlock` or `InstructionInline` node so that the plugin assistant
        can provide a system prompt template to models.

        It receives a `GenerateTask` and `GenerateOptions` and should return a
        `string`. This default implementation returns an empty string.

        Args:
            task: The task to create a system prompt template for
            options: Options for generation
            assistant: The id of the assistant that should create the system prompt
        Returns:


        @return string
        """
        return ""

    @abstractmethod
    async def perform_task(
        self, task: GenerateTask, options: GenerateOptions
    ) -> GenerateOutput:
        """
        Execute an instruction using an assistant

        This method is called by Stencila when executing `InstructionBlock` and
        `InstructionInline` nodes.

        It receives a `GenerateTask` and `GenerateOptions` and should return
        a `GenerateOutput`. This default implementation raises an error.
        """
        raise NotImplementedError
