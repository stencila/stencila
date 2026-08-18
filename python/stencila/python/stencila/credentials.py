"""Make provenance-aware Content Credentials accessible from Python.

This public layer coordinates rendering, context resolution, and native
cryptography while returning typed results that keep each guarantee explicit.
"""

from __future__ import annotations

import json
import re
from collections.abc import Callable, Mapping
from dataclasses import dataclass, fields
from os import PathLike
from pathlib import Path
from typing import Any, Literal, TypeVar

from stencila import _stencila

from ._context import absolute
from ._errors import StencilaError
from ._graph import (
    Profile,
    Provenance,
    ProvenanceError,
    ProvenanceNotFoundError,
    _discard,
    _prepare,
    graph,
)
from ._render import UnsupportedPlotError, register_renderer
from ._types import Graph, graph_from_data

T = TypeVar("T")

_CAMEL_BOUNDARY = re.compile(r"(?<!^)(?=[A-Z])")


class CredentialsError(StencilaError):
    """Raised when credential production or verification fails."""


def _call(operation: Callable[..., str], *args: object) -> dict[str, Any]:
    """Run a native credential operation and decode its JSON result.

    Native failures surface as one package-level error so callers do not have
    to distinguish interpreter-level exceptions from credential problems.
    """
    try:
        return json.loads(operation(*args))
    except RuntimeError as error:
        raise CredentialsError(str(error)) from error


def _build(cls: type[T], data: Mapping[str, Any], **overrides: Any) -> T:
    """Map a native camelCase payload onto a result dataclass.

    Fields this version does not model are ignored rather than raising, so a
    newer native library can add report fields without breaking callers.
    """
    names = {field.name for field in fields(cls)}  # pyright: ignore[reportArgumentType]
    values = {
        _CAMEL_BOUNDARY.sub("_", key).lower(): value for key, value in data.items()
    }
    values.update(overrides)
    return cls(**{key: value for key, value in values.items() if key in names})


@dataclass(frozen=True)
class SignedAsset:
    """Describe the signed output and the provenance actually embedded.

    Source and signed digests remain separate because embedding a manifest may
    change the asset bytes even when the represented content is unchanged.
    """

    path: Path
    graph: Graph
    manifest_kind: Literal["embedded", "sidecar"]
    manifest_id: str | None
    sidecar_path: Path | None
    media_type: str
    source_digest: str
    signed_digest: str
    signing_mode: Literal["local", "cloud"]
    profile: Profile
    warnings: tuple[str, ...]


@dataclass(frozen=True)
class LocalIdentity:
    """Report where the local self-signed identity is stored."""

    cert_path: Path
    key_path: Path
    created: bool
    common_name: str


@dataclass(frozen=True)
class ManifestStatus:
    """Separate manifest discovery and activation from signature validity."""

    present: bool
    valid: bool
    active: bool
    from_sidecar: bool


@dataclass(frozen=True)
class SignatureStatus:
    """Keep cryptographic validity distinct from signer trust."""

    valid: bool
    trusted: bool
    signer: str | None


@dataclass(frozen=True)
class AssetBindingStatus:
    """Report whether the manifest is bound to the supplied asset bytes."""

    valid: bool


@dataclass(frozen=True)
class ProvenanceStatus:
    """Describe discovery and validation of the Stencila assertion."""

    assertion_present: bool
    attested: bool
    schema_url: str | None
    schema_known: bool
    assertion: Graph | None
    raw: Mapping[str, Any] | None


@dataclass(frozen=True)
class VerificationReport:
    """Keep independent verification guarantees available to policy callers.

    A structured report avoids collapsing valid signatures, trusted identities,
    asset binding, and provenance presence into one ambiguous boolean.
    """

    manifest: ManifestStatus
    signature: SignatureStatus
    asset_binding: AssetBindingStatus
    provenance: ProvenanceStatus
    reproducibility: str
    summary: Mapping[str, Any]
    problems: tuple[str, ...]


def sign(  # noqa: PLR0913
    subject: object | str | PathLike[str],
    output: str | PathLike[str] | None = None,
    *,
    source: str | PathLike[str] | None = None,
    workspace: str | PathLike[str] | None = None,
    title: str | None = None,
    profile: Profile = "public",
    provenance: Provenance = "auto",
    render_options: Mapping[str, Any] | None = None,
    cert: str | PathLike[str] | None = None,
    key: str | PathLike[str] | None = None,
    tsa_url: str | None = None,
) -> SignedAsset:
    """Sign an asset with provenance selected from its workspace.

    Live plots are rendered once and the resulting graph is passed unchanged to
    native signing, ensuring the returned assertion describes the signed input.
    """
    if (cert is None) != (key is None):
        msg = "cert and key must be supplied together"
        raise ValueError(msg)
    input_path = absolute(subject) if isinstance(subject, str | PathLike) else None
    if input_path is None and output is None:
        msg = "output is required for an in-memory plot"
        raise ValueError(msg)
    destination = absolute(output) if output is not None else input_path
    if destination is None:
        msg = "unable to resolve signing destination"
        raise CredentialsError(msg)
    if output is not None:
        destination.parent.mkdir(parents=True, exist_ok=True)
    if destination.is_symlink():
        msg = f"refusing to replace symlink destination: {destination}"
        raise ValueError(msg)

    prepared, temporary = _prepare(
        subject,
        output=destination,
        source=source,
        workspace=workspace,
        profile=profile,
        provenance=provenance,
        render_options=render_options,
        title=title,
    )
    signing_input = temporary or input_path
    if signing_input is None:
        _discard(temporary)
        msg = "unable to resolve signing input"
        raise CredentialsError(msg)
    try:
        result = _call(
            _stencila.credentials.sign_prepared,
            str(signing_input),
            str(destination),
            json.dumps(prepared),
            title,
            profile,
            str(cert) if cert else None,
            str(key) if key else None,
            tsa_url,
        )
    finally:
        _discard(temporary)

    return _build(
        SignedAsset,
        result,
        path=Path(result["path"]),
        graph=graph_from_data(result["graph"]),
        sidecar_path=Path(result["sidecarPath"]) if result["sidecarPath"] else None,
        warnings=tuple(result["warnings"]),
    )


def init(*, force: bool = False) -> LocalIdentity:
    """Initialize the local identity used for convenient development signing.

    The identity is self-signed, so verification can distinguish a valid local
    signature from one rooted in an independently trusted certificate.
    """
    result = _call(_stencila.credentials.init, force)
    return _build(
        LocalIdentity,
        result,
        cert_path=Path(result["certPath"]),
        key_path=Path(result["keyPath"]),
    )


def verify(
    path: str | PathLike[str],
    *,
    require_trusted_signer: bool = False,
    require_stencila_assertion: bool = False,
) -> VerificationReport:
    """Verify signature, trust, asset binding, and Stencila provenance.

    These guarantees are reported independently so callers can enforce trust
    and assertion requirements appropriate to their environment.
    """
    result = _call(
        _stencila.credentials.verify,
        str(path),
        require_trusted_signer,
        require_stencila_assertion,
    )
    provenance = result["provenance"]
    return _build(
        VerificationReport,
        result,
        manifest=_build(ManifestStatus, result["manifest"]),
        signature=_build(SignatureStatus, result["signature"]),
        asset_binding=_build(AssetBindingStatus, result["assetBinding"]),
        provenance=_build(
            ProvenanceStatus,
            provenance,
            assertion=(
                graph_from_data(provenance["assertion"])
                if provenance["assertion"]
                else None
            ),
        ),
        summary=result.get("summary", {}),
        problems=tuple(result["problems"]),
    )


def inspect(path: str | PathLike[str]) -> Mapping[str, Any]:
    """Return raw credential data for diagnostics and advanced integrations."""
    return _call(_stencila.credentials.inspect, str(path))


__all__ = [
    "CredentialsError",
    "LocalIdentity",
    "ProvenanceError",
    "ProvenanceNotFoundError",
    "SignedAsset",
    "UnsupportedPlotError",
    "VerificationReport",
    "graph",
    "init",
    "inspect",
    "register_renderer",
    "sign",
    "verify",
]
