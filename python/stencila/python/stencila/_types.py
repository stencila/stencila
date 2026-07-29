"""Return typed graphs across supported Stencila Schema package versions.

The compatibility model keeps this feature usable before Graph reaches the
minimum dependency version without duplicating conversion logic elsewhere.
"""

from __future__ import annotations

import json
from dataclasses import dataclass
from typing import TYPE_CHECKING, Any, Literal, TypeAlias

import stencila_types.types as schema_types
from stencila_types.utilities import from_json

if TYPE_CHECKING:
    Graph: TypeAlias = Any
    GraphEvidence: Any
    GraphEdge: Any
    GraphNode: Any
    _USING_GRAPH_FALLBACK = False
else:
    try:
        Graph = schema_types.Graph  # pyright: ignore[reportAttributeAccessIssue]
        _USING_GRAPH_FALLBACK = False
    except AttributeError:
        _USING_GRAPH_FALLBACK = True

        @dataclass(frozen=True)
        class GraphEvidence:
            """Represent graph evidence when generated types are unavailable."""

            kind: str
            type: Literal["GraphEvidence"] = "GraphEvidence"
            id: str | None = None
            confidence: str | None = None
            code_location: dict[str, Any] | None = None
            source: object | str | None = None
            recorded_at: object | None = None
            details: dict[str, Any] | None = None
            description: str | None = None

        @dataclass(frozen=True)
        class GraphEdge:
            """Represent a typed graph edge for older Schema packages."""

            source: str
            target: str
            kind: str
            type: Literal["GraphEdge"] = "GraphEdge"
            id: str | None = None
            evidence: list[GraphEvidence] | None = None
            actions: list[dict[str, Any]] | None = None

        @dataclass(frozen=True)
        class GraphNode:
            """Represent a typed graph node for older Schema packages."""

            id: str
            node: dict[str, Any]
            type: Literal["GraphNode"] = "GraphNode"

        @dataclass(frozen=True)
        class Graph:
            """Provide the Graph shape missing from older type packages.

            This fallback can disappear once every supported release ships the
            generated Graph model.
            """

            subject: str
            nodes: list[GraphNode]
            edges: list[GraphEdge]
            type: Literal["Graph"] = "Graph"
            id: str | None = None

        schema_types.GraphEvidence = GraphEvidence  # pyright: ignore[reportAttributeAccessIssue]
        schema_types.GraphEdge = GraphEdge  # pyright: ignore[reportAttributeAccessIssue]
        schema_types.GraphNode = GraphNode  # pyright: ignore[reportAttributeAccessIssue]
        schema_types.Graph = Graph  # pyright: ignore[reportAttributeAccessIssue]


def graph_from_data(data: dict[str, Any]) -> Graph:
    """Use the installed generated Graph model whenever it is available.

    Falling back only for the known compatibility shape preserves the normal
    Schema deserializer for current package versions.
    """
    if _USING_GRAPH_FALLBACK:
        return Graph(  # pyright: ignore[reportCallIssue]
            type=data.get("type", "Graph"),
            id=data.get("id"),
            subject=data["subject"],
            nodes=[
                GraphNode(
                    type=node.get("type", "GraphNode"),
                    id=node["id"],
                    node=node["node"],
                )
                for node in data.get("nodes", [])
            ],
            edges=[
                GraphEdge(
                    type=edge.get("type", "GraphEdge"),
                    id=edge.get("id"),
                    source=edge["source"],
                    target=edge["target"],
                    kind=edge["kind"],
                    evidence=(
                        [
                            GraphEvidence(
                                type=evidence.get("type", "GraphEvidence"),
                                id=evidence.get("id"),
                                kind=evidence["kind"],
                                confidence=evidence.get("confidence"),
                                code_location=evidence.get("codeLocation"),
                                source=evidence.get("source"),
                                recorded_at=evidence.get("recordedAt"),
                                details=evidence.get("details"),
                                description=evidence.get("description"),
                            )
                            for evidence in edge["evidence"]
                        ]
                        if edge.get("evidence")
                        else None
                    ),
                    actions=edge.get("actions"),
                )
                for edge in data.get("edges", [])
            ],
        )
    return from_json(json.dumps(data))
