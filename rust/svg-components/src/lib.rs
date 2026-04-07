//! SVG overlay component compiler for Stencila figures.
//!
//! This crate compiles SVG overlays containing `s:` custom elements into
//! standard SVG. It is used by the figure and code chunk execution paths
//! to produce `overlay_compiled` output from authored `overlay` source.
//!
//! # Compilation pipeline
//!
//! [`compile`] runs through these stages:
//!
//! 1. **Detection** — check whether the source contains any `s:` custom
//!    elements or `s:` defs references. Pure standard SVG is passed through
//!    unchanged (`compiled: None`).
//! 2. **Anchor collection** — gather explicit `<s:anchor>` definitions and
//!    generate auto-anchors from the `viewBox` (e.g. `#s:center`,
//!    `#s:top-left`).
//! 3. **Component expansion** — expand each `s:` element into standard SVG
//!    fragments via per-component handlers.
//! 4. **Defs scan** — scan the expanded output for `url(#s:...)` and
//!    `href="#s:..."` references to built-in marker/symbol definitions.
//! 5. **Tree-shaken defs injection** — inject only the referenced built-in
//!    `<defs>` entries, avoiding unused bloat.
//!
//! # Pass-through behavior
//!
//! If the source SVG contains no `s:` elements and no `s:` defs references,
//! the compiler returns `compiled: None` to signal the source should be used
//! directly. This avoids unnecessary rewriting of standard SVG overlays.
//!
//! # Diagnostics
//!
//! Malformed or unsupported `s:` elements produce [`diagnostics::CompilationMessage`]
//! entries at `Error` or `Warning` level rather than panicking. Unknown `s:`
//! element names produce warnings; missing required attributes produce errors.
//! Compilation continues past individual component failures.
//!
//! # Supported components
//!
//! | Component | Purpose |
//! |-----------|---------|
//! | `<s:arrow>` | Connector line with optional arrowhead markers |
//! | `<s:callout>` | Text label with optional leader line and background shape |
//! | `<s:badge>` | Compact pill-shaped label |
//! | `<s:scale-bar>` | Measurement bar with end caps and label |
//! | `<s:dimension>` | Dimension line between two points with caps and label |
//! | `<s:brace>` | Curly brace path between two points |
//! | `<s:roi-rect>` | Rectangular region-of-interest outline |
//! | `<s:roi-ellipse>` | Elliptical region-of-interest outline |
//! | `<s:marker>` | Point marker using defs-backed symbols |
//! | `<s:compass>` | Directional compass rose (arrow or full-axis variant) |
//! | `<s:anchor>` | Named anchor point for coordinate references |
//!
//! # Shared attribute semantics
//!
//! Several attributes have consistent meaning across all components that use them:
//!
//! - `x`, `y` — position coordinates
//! - `at` — anchor reference for single-point components (e.g. `at="#peak"`)
//! - `from`, `to` — anchor references for two-point components (e.g. `from="#peak"`)
//! - `dx`, `dy` — offset from anchor position
//! - `label` — text content
//! - `label-position` — where to place the label relative to the component
//! - `curve` — connector path type: `straight`, `elbow`, `quad`, `cubic`
//! - `tip` — arrowhead placement: `end`, `start`, `both`, `none`
//! - `tip-style` — marker id for arrowheads (default `s:arrow-closed`)
//! - `side` — which side a component extends toward
//! - `stroke-style` — line style: `solid`, `dashed`, `dotted` (ROI components only)
//!
//! # Reserved `s:` id namespace
//!
//! All ids prefixed with `s:` are reserved for built-in definitions:
//!
//! - `s:arrow-closed`, `s:arrow-open`, `s:arrow-dot` — arrow markers
//! - `s:marker-circle`, `s:marker-cross`, `s:marker-pin`, `s:marker-star` — point symbols
//! - `s:cap-line` — dimension/scale-bar end cap marker
//!
//! These are injected into `<defs>` only when referenced (tree-shaken).
//!
//! # Roundtrip support
//!
//! Each expanded component is wrapped in a `<g>` element carrying the original
//! component name and attributes in the `s:` namespace:
//!
//! ```xml
//! <!-- source -->
//! <s:badge x="300" y="50" label="p < 0.05"/>
//!
//! <!-- compiled -->
//! <g s:component="badge" s:label="p &lt; 0.05" s:x="300" s:y="50">
//!   <rect x="258.25" y="41" width="83.5" height="18" rx="9" …/>
//!   <text x="300" y="50" …>p &lt; 0.05</text>
//! </g>
//! ```
//!
//! This makes it possible to reverse-compile (decompile) standard SVG back
//! into `s:` source components. A decompiler walks the SVG tree looking for
//! `<g s:component="…">` groups, reconstructs the original `<s:*>` element
//! from the `s:` attributes, and replaces the group — discarding the expanded
//! SVG inside.
//!
//! The wrapper groups are also useful when editing compiled overlays in
//! external SVG editors (Inkscape, Illustrator, Figma, etc.). Because each
//! component's output is grouped, the editor treats it as a single selectable
//! object that can be moved, reordered, or deleted without disturbing sibling
//! elements. After editing, a decompile pass can recover the `s:` source —
//! updating coordinates from the group's transform if present — enabling a
//! visual-edit → decompile → re-compile roundtrip workflow.

#![warn(clippy::pedantic)]
#![allow(
    clippy::similar_names,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation
)]

mod anchors;
mod bbox;
mod compile;
mod component_attrs;
mod component_bbox;
mod components;
mod defs;
pub mod diagnostics;
mod lint;

pub use compile::{CompilationResult, compile};
pub use lint::{LintResult, lint};

/// Compile and lint an SVG overlay in a single call.
///
/// Runs both the compilation pipeline and the static analysis linter,
/// merging all diagnostic messages into the returned `CompilationResult`.
#[must_use]
pub fn compile_and_lint(source: &str) -> CompilationResult {
    let mut result = compile(source);
    let lint_result = lint(source);
    result.messages.extend(lint_result.messages);
    result
}
