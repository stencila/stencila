import inspect
import json
import os
from base64 import b64decode
from pathlib import Path

import pytest
import stencila_types.types as T
from stencila_types.utilities import to_json

from stencila import _stencila, credentials
from stencila._context import resolve_context
from stencila._graph import ProvenanceError, _discard, _prepare
from stencila._render import UnsupportedPlotError, render
from stencila.credentials import graph

# A 1x1 PNG. Signing embeds a manifest into real image bytes, so the rendered
# output has to be a decodable image rather than a placeholder.
PNG = b64decode(
    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg=="
)


class Plot:
    pass


class Figure:
    """Stand in for a Matplotlib Figure without depending on Matplotlib.

    Detection is by module and class name across the MRO, so a class shaped
    and named like the real one exercises the same path.
    """

    __module__ = "matplotlib.figure"

    def __init__(self):
        self.saved = None

    def savefig(self, path, **options):
        Path(path).write_bytes(PNG)
        self.saved = options


class Axes:
    __module__ = "matplotlib.axes._axes"

    def __init__(self, figure):
        self.figure = figure


class WrappedFigure(Figure):
    """A subclass is still a Figure, which is why the check walks the MRO."""

    __module__ = "some_wrapper.figures"


def test_registered_renderer(tmp_path: Path):
    output = tmp_path / "plot.png"

    def renderer(_plot, path, options):
        path.write_bytes(options["content"])

    credentials.register_renderer(Plot, renderer)
    render(Plot(), output, {"content": b"png"})
    assert output.read_bytes() == b"png"


def test_unsupported_renderer(tmp_path: Path):
    with pytest.raises(UnsupportedPlotError, match="register_renderer"):
        render(object(), tmp_path / "plot.png", {})


def test_renders_matplotlib_figure(tmp_path: Path):
    output = tmp_path / "plot.png"
    figure = Figure()

    render(figure, output, {"dpi": 300})

    assert output.read_bytes() == PNG
    assert figure.saved == {"dpi": 300}


def test_renders_matplotlib_axes_via_its_figure(tmp_path: Path):
    output = tmp_path / "plot.png"
    figure = Figure()

    render(Axes(figure), output, {})

    assert output.read_bytes() == PNG


def test_renders_matplotlib_subclass_outside_matplotlib(tmp_path: Path):
    output = tmp_path / "plot.png"

    render(WrappedFigure(), output, {})

    assert output.read_bytes() == PNG


def test_explicit_context(tmp_path: Path):
    source = tmp_path / "analysis.py"
    source.write_text("")
    context = resolve_context(
        tmp_path / "figures" / "plot.png",
        source=source,
        workspace=tmp_path,
        infer_source=False,
    )
    assert context.source == source
    assert context.workspace == tmp_path


def test_existing_asset_context_uses_input_workspace(tmp_path: Path):
    project = tmp_path / "project"
    project.mkdir()
    (project / ".git").mkdir()
    asset = project / "figures" / "plot.png"
    asset.parent.mkdir()
    asset.write_bytes(b"png")

    context = resolve_context(
        tmp_path / "exports" / "plot.png",
        source=None,
        workspace=None,
        infer_source=False,
        asset=asset,
    )

    assert context.workspace == project


def test_unrelated_source_does_not_widen_workspace(tmp_path: Path):
    """A source on another branch must not make the scan root their ancestor.

    A notebook cell under a temporary directory would otherwise resolve the
    workspace to `$HOME` or `/`, and the scan would walk all of it.
    """
    project = tmp_path / "project"
    (project / "figures").mkdir(parents=True)
    (project / ".git").mkdir()
    cell = tmp_path / "kernel" / "cell.py"
    cell.parent.mkdir()
    cell.write_text("")

    context = resolve_context(
        project / "figures" / "plot.png",
        source=cell,
        workspace=None,
        infer_source=False,
    )

    assert context.workspace == project


def test_unmarked_source_and_output_stay_within_the_output_directory(tmp_path: Path):
    source = tmp_path / "scripts" / "analysis.py"
    source.parent.mkdir()
    source.write_text("")
    output = tmp_path / "figures" / "plot.png"
    output.parent.mkdir()

    context = resolve_context(
        output,
        source=source,
        workspace=None,
        infer_source=False,
    )

    assert context.workspace == output.parent


def test_source_project_used_when_output_has_none(tmp_path: Path):
    project = tmp_path / "project"
    (project / "scripts").mkdir(parents=True)
    (project / ".git").mkdir()
    source = project / "scripts" / "analysis.py"
    source.write_text("")
    output = tmp_path / "exports" / "plot.png"
    output.parent.mkdir()

    context = resolve_context(output, source=source, workspace=None, infer_source=False)

    assert context.workspace == project


def test_inferred_source_line_is_zero_based(tmp_path: Path):
    frame = inspect.currentframe()
    assert frame is not None
    call_line = frame.f_lineno + 1
    context = resolve_context(
        tmp_path / "plot.png",
        source=None,
        workspace=tmp_path,
        infer_source=True,
    )

    assert context.source == Path(__file__).resolve()
    assert context.source_line == call_line - 1


def test_graph_is_typed(tmp_path: Path):
    asset = tmp_path / "plot.png"
    asset.write_bytes(b"unsigned image bytes")
    value = graph(asset, workspace=tmp_path, provenance="none")
    assert isinstance(value, T.Graph)  # pyright: ignore[reportAttributeAccessIssue]
    assert not isinstance(value.nodes[0], dict)
    assert value.subject == "asset:plot.png"
    assert '"type": "Graph"' in to_json(value)


def test_existing_asset_is_staged_as_a_stable_signing_input(tmp_path: Path):
    asset = tmp_path / "plot.png"
    asset.write_bytes(PNG)

    _prepared, staged = _prepare(
        asset,
        output=asset,
        source=None,
        workspace=tmp_path,
        profile="public",
        provenance="none",
        render_options=None,
    )
    assert staged is not None
    try:
        assert staged != asset
        assert staged.read_bytes() == PNG
        asset.write_bytes(b"changed after preparation")
        assert staged.read_bytes() == PNG
    finally:
        _discard(staged)


def test_existing_asset_keeps_lineage_when_output_path_changes(tmp_path: Path):
    source = tmp_path / "analysis.py"
    source.write_text(
        'from stencila import credentials\ncredentials.sign("original.png")\n'
    )
    asset = tmp_path / "original.png"
    asset.write_bytes(PNG)

    value = graph(asset, output=tmp_path / "exported.png", workspace=tmp_path)
    ids = {node.id for node in value.nodes}

    assert value.subject == "asset:exported.png"
    assert "code:analysis.py" in ids
    assert any(
        edge.source == "code:analysis.py" and edge.target == "asset:signed"
        for edge in value.edges
    )


def test_explicit_source_replaces_conflicting_discovered_producer(tmp_path: Path):
    old_source = tmp_path / "old.py"
    old_source.write_text(
        'from stencila import credentials\ncredentials.sign("plot.png")\n'
    )
    requested_source = tmp_path / "new.py"
    requested_source.write_text("# runtime producer\n")
    asset = tmp_path / "plot.png"
    asset.write_bytes(PNG)

    value = graph(
        asset,
        source=requested_source,
        workspace=tmp_path,
        provenance="required",
    )
    ids = {node.id for node in value.nodes}

    assert "code:new.py" in ids
    assert "code:old.py" not in ids
    assert any(
        edge.source == "code:new.py" and edge.target == "asset:signed"
        for edge in value.edges
    )


def test_rendered_bytes_are_removed_when_preparation_fails(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
):
    """A ValueError from the native layer must not leak the rendered file.

    Cleanup used to run only for RuntimeError, so any other failure left the
    temporary render behind in the caller's output directory.
    """

    def fail(*_args):
        msg = "malformed payload"
        raise ValueError(msg)

    monkeypatch.setattr(_stencila.graph, "prepare", fail)

    with pytest.raises(ValueError, match="malformed payload"):
        graph(Figure(), output=tmp_path / "plot.png", workspace=tmp_path)

    assert list(tmp_path.iterdir()) == []


def test_preparation_errors_are_wrapped(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
):
    def fail(*_args):
        msg = "workspace unreadable"
        raise RuntimeError(msg)

    monkeypatch.setattr(_stencila.graph, "prepare", fail)
    asset = tmp_path / "plot.png"
    asset.write_bytes(b"png")

    with pytest.raises(ProvenanceError, match="workspace unreadable"):
        graph(asset, workspace=tmp_path)


def test_verification_report_ignores_unmodelled_fields(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
):
    """Native reports may gain fields this package does not model yet.

    Splatting the payload into the dataclasses used to raise TypeError the
    moment a field was added on the Rust side.
    """
    payload = {
        "manifest": {
            "present": True,
            "valid": True,
            "active": True,
            "fromSidecar": False,
            "futureField": "ignored",
        },
        "signature": {
            "valid": True,
            "trusted": False,
            "signer": "Stencila Local",
            "futureField": "ignored",
        },
        "assetBinding": {"valid": True, "futureField": "ignored"},
        "provenance": {
            "assertionPresent": True,
            "attested": True,
            "schemaUrl": "https://stencila.org/context.jsonld",
            "schemaKnown": True,
            "assertion": None,
            "raw": None,
            "futureField": "ignored",
        },
        "reproducibility": "not-attempted",
        "problems": [],
        "futureField": "ignored",
    }
    monkeypatch.setattr(
        _stencila.credentials, "verify", lambda *_args: json.dumps(payload)
    )

    report = credentials.verify(tmp_path / "plot.png")

    assert report.manifest.from_sidecar is False
    assert report.signature.signer == "Stencila Local"
    assert report.asset_binding.valid is True
    assert report.provenance.schema_known is True
    assert report.reproducibility == "not-attempted"
    assert report.summary == {}
    assert report.problems == ()


def test_native_errors_become_credentials_errors(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
):
    def fail(*_args):
        msg = "no signing identity"
        raise RuntimeError(msg)

    monkeypatch.setattr(_stencila.credentials, "init", fail)

    with pytest.raises(credentials.CredentialsError, match="no signing identity"):
        credentials.init()


def test_sign_rejects_partial_signing_material(tmp_path: Path):
    with pytest.raises(ValueError, match="cert and key"):
        credentials.sign(Figure(), tmp_path / "plot.png", cert="cert.pem")


def test_sign_requires_output_for_in_memory_plots():
    with pytest.raises(ValueError, match="output is required"):
        credentials.sign(Figure())


def test_sign_refuses_symlink_destinations(tmp_path: Path):
    real = tmp_path / "real.png"
    real.write_bytes(b"png")
    link = tmp_path / "link.png"
    link.symlink_to(real)

    with pytest.raises(ValueError, match="symlink"):
        credentials.sign(Figure(), link)


@pytest.mark.skipif(
    not os.environ.get("STENCILA_TEST_SIGNING"),
    reason="writes a signing identity to the user's config directory",
)
def test_sign_and_verify_round_trip(tmp_path: Path):
    """Exercise the full native path; opt-in because `init` is not hermetic.

    `init` writes the local identity under the platform config directory, so
    this is gated rather than run by default.
    """
    identity = credentials.init()
    assert identity.cert_path.exists()

    source = tmp_path / "analysis.py"
    source.write_text("from stencila import credentials\n")
    signed = credentials.sign(
        Figure(),
        tmp_path / "figures" / "plot.png",
        source=source,
        workspace=tmp_path,
    )

    assert signed.path.exists()
    assert signed.profile == "public"
    assert signed.source_digest.startswith("sha256")
    assert isinstance(signed.warnings, tuple)

    report = credentials.verify(signed.path, require_stencila_assertion=True)
    assert report.signature.valid
    assert report.asset_binding.valid
    assert report.provenance.assertion_present
    assert credentials.inspect(signed.path)
