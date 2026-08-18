from typing import TypedDict

class DecodeOptions(TypedDict):
    format: str | None

class EncodeOptions(TypedDict):
    format: str | None
    standalone: bool | None
    compact: bool | None

class CompareOptions(TypedDict):
    alignment_cell_budget: int | None
    include: list[str] | None
    exclude: list[str] | None
    left_format: str | None
    right_format: str | None
    left_label: str | None
    right_label: str | None
    reports: list[str] | None
    summary: bool | None

# This is actually a module.
class compare:  # noqa: N801
    @staticmethod
    def nodes(left: str, right: str, options: CompareOptions) -> str: ...
    @staticmethod
    def strings(left: str, right: str, options: CompareOptions) -> str: ...
    @staticmethod
    def paths(left: str, right: str, options: CompareOptions) -> str: ...
    @staticmethod
    def is_equal(comparison: str) -> bool: ...

# This is actually a module.
class convert:  # noqa: N801
    @staticmethod
    async def from_string(string: str, options: DecodeOptions) -> str: ...
    @staticmethod
    async def from_path(path: str, options: DecodeOptions) -> str: ...
    @staticmethod
    async def to_string(json: str, options: EncodeOptions) -> str: ...
    @staticmethod
    async def to_path(json: str, path: str, options: EncodeOptions) -> str: ...
    @staticmethod
    async def from_to(
        input: str,  # noqa: A002
        output: str,
        decode_options: DecodeOptions,
        encode_options: EncodeOptions,
    ) -> str: ...

class graph:  # noqa: N801
    @staticmethod
    def prepare(
        input_path: str,
        asset_path: str,
        lookup_path: str,
        workspace: str,
        source: str | None = None,
        source_line: int | None = None,
        profile: str = "public",
        provenance: str = "auto",
        title: str | None = None,
    ) -> str: ...

class credentials:  # noqa: N801
    @staticmethod
    def sign_prepared(
        input_path: str,
        output_path: str,
        prepared: str,
        title: str | None = None,
        profile: str = "public",
        cert: str | None = None,
        key: str | None = None,
        tsa_url: str | None = None,
    ) -> str: ...
    @staticmethod
    def init(force: bool = False) -> str: ...
    @staticmethod
    def verify(
        path: str,
        require_trusted_signer: bool = False,
        require_stencila_assertion: bool = False,
    ) -> str: ...
    @staticmethod
    def inspect(path: str) -> str: ...
