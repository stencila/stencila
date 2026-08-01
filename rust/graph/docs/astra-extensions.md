# Stencila extensions to ASTRA

This document records experimental ASTRA fields recognized by Stencila before
they are part of the upstream ASTRA schema. These extensions are proposals, not
ASTRA v1 fields. A manifest using them may therefore be rejected by the
upstream `astra validate` command until the corresponding schema change is
accepted.

## Provisional graph mapping

The ASTRA graph projection intentionally uses generic Schema Object payloads
for analyses, decisions, options, and universes. Each has an astraType, local
id, human-facing name, and fully qualified scope, plus the metadata specific to
that ASTRA concept. This mapping is provisional: it avoids assigning the
meaning of an existing Stencila type to an ASTRA structural declaration before
a purpose-built Schema vocabulary exists.

The graph identity and relationships are stable migration boundaries. A future
purpose-built representation must preserve the astra-analysis, astra-decision,
astra-option, astra-insight, astra-evidence, and astra-universe id families,
along with containment, configuration, support, citation, and grounding edges.
Insights and their evidence already use the purpose-aligned Claim and Evidence
types; inputs, outputs, and recipes retain their concrete resource, artifact,
value, and Function mappings.

## `Output.target`

The proposal, motivation, semantics, and examples for this extension are in
[LightconeResearch/astra-spec#58](https://github.com/LightconeResearch/astra-spec/pull/58).

Stencila recognizes the proposed optional field so it can associate a logical
ASTRA output with the URI or path where it is materialized and project that
association into the graph.
