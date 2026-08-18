"""Type stubs for the native extension module.

Every function here is synchronous, takes its options as keyword arguments, and
returns either a plain value or a JSON string for the public Python layer to decode.
"""

# These are actually modules, declared as classes so their members can be typed.

class compare:  # noqa: N801
    @staticmethod
    def nodes(
        left: str,
        right: str,
        *,
        include: list[str] | None = None,
        exclude: list[str] | None = None,
        alignment_cell_budget: int | None = None,
        reports: list[str] | None = None,
        summary: bool | None = None,
        left_label: str | None = None,
        right_label: str | None = None,
    ) -> str: ...
    @staticmethod
    def strings(
        left: str,
        right: str,
        *,
        left_format: str | None = None,
        right_format: str | None = None,
        include: list[str] | None = None,
        exclude: list[str] | None = None,
        alignment_cell_budget: int | None = None,
        reports: list[str] | None = None,
        summary: bool | None = None,
        left_label: str | None = None,
        right_label: str | None = None,
    ) -> str: ...
    @staticmethod
    def paths(
        left: str,
        right: str,
        *,
        left_format: str | None = None,
        right_format: str | None = None,
        include: list[str] | None = None,
        exclude: list[str] | None = None,
        alignment_cell_budget: int | None = None,
        reports: list[str] | None = None,
        summary: bool | None = None,
        left_label: str | None = None,
        right_label: str | None = None,
    ) -> str: ...
    @staticmethod
    def is_equal(comparison: str) -> bool: ...

class convert:  # noqa: N801
    @staticmethod
    def from_string(string: str, format: str | None = None) -> str: ...
    @staticmethod
    def from_path(path: str, format: str | None = None) -> str: ...
    @staticmethod
    def to_string(
        json: str,
        format: str | None = None,
        standalone: bool | None = None,
        compact: bool | None = None,
    ) -> str: ...
    @staticmethod
    def to_path(
        json: str,
        path: str,
        format: str | None = None,
        standalone: bool | None = None,
        compact: bool | None = None,
    ) -> None: ...
    @staticmethod
    def from_to(
        input: str | None = None,
        output: str | None = None,
        from_format: str | None = None,
        to_format: str | None = None,
        to_standalone: bool | None = None,
        to_compact: bool | None = None,
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
