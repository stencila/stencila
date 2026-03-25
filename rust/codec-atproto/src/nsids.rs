//! AT Protocol Namespaced Identifier (NSID) constants.
//!
//! Contains constants for OXA block types, OXA richtext facets, Leaflet richtext
//! facets, and Bluesky richtext facets. These NSIDs are used as `$type` values
//! in encoded AT Protocol JSON.

// OXA block-level NSIDs
// TODO: These are provisional and subject to change as the OXA Lexicon is finalized

/// NSID for an OXA paragraph block.
pub const OXA_PARAGRAPH: &str = "pub.oxa.blocks.defs#paragraph";
/// NSID for an OXA heading block.
pub const OXA_HEADING: &str = "pub.oxa.blocks.defs#heading";
/// NSID for an OXA code block.
pub const OXA_CODE: &str = "pub.oxa.blocks.defs#code";
/// NSID for an OXA thematic break block.
pub const OXA_THEMATIC_BREAK: &str = "pub.oxa.blocks.defs#thematicBreak";
/// NSID for an OXA math block.
pub const OXA_MATH: &str = "pub.oxa.blocks.defs#math";
/// NSID for an OXA blockquote block.
pub const OXA_BLOCKQUOTE: &str = "pub.oxa.blocks.defs#blockquote";
/// NSID for an OXA ordered list block.
pub const OXA_ORDERED_LIST: &str = "pub.oxa.blocks.defs#orderedList";
/// NSID for an OXA unordered list block.
pub const OXA_UNORDERED_LIST: &str = "pub.oxa.blocks.defs#unorderedList";

// OXA richtext facet NSIDs
// TODO: These are provisional and subject to change as the OXA Lexicon is finalized

/// NSID for an OXA emphasis (italic) facet feature.
pub const OXA_EMPHASIS: &str = "pub.oxa.richtext.facet#emphasis";
/// NSID for an OXA strong (bold) facet feature.
pub const OXA_STRONG: &str = "pub.oxa.richtext.facet#strong";
/// NSID for an OXA inline code facet feature.
pub const OXA_INLINE_CODE: &str = "pub.oxa.richtext.facet#inlineCode";
/// NSID for an OXA subscript facet feature.
pub const OXA_SUBSCRIPT: &str = "pub.oxa.richtext.facet#subscript";
/// NSID for an OXA superscript facet feature.
pub const OXA_SUPERSCRIPT: &str = "pub.oxa.richtext.facet#superscript";
/// NSID for an OXA strikethrough facet feature.
pub const OXA_STRIKETHROUGH: &str = "pub.oxa.richtext.facet#strikethrough";
/// NSID for an OXA underline facet feature.
pub const OXA_UNDERLINE: &str = "pub.oxa.richtext.facet#underline";
/// NSID for an OXA link facet feature.
pub const OXA_LINK: &str = "pub.oxa.richtext.facet#link";

// Leaflet richtext facet NSIDs

/// NSID for a Leaflet italic facet feature.
pub const LEAFLET_ITALIC: &str = "pub.leaflet.richtext.facet#italic";
/// NSID for a Leaflet bold facet feature.
pub const LEAFLET_BOLD: &str = "pub.leaflet.richtext.facet#bold";
/// NSID for a Leaflet code facet feature.
pub const LEAFLET_CODE: &str = "pub.leaflet.richtext.facet#code";
/// NSID for a Leaflet strikethrough facet feature.
pub const LEAFLET_STRIKETHROUGH: &str = "pub.leaflet.richtext.facet#strikethrough";
/// NSID for a Leaflet underline facet feature.
pub const LEAFLET_UNDERLINE: &str = "pub.leaflet.richtext.facet#underline";
/// NSID for a Leaflet link facet feature.
pub const LEAFLET_LINK: &str = "pub.leaflet.richtext.facet#link";

// Bluesky richtext facet NSIDs

/// NSID for a Bluesky link facet feature.
pub const BLUESKY_LINK: &str = "app.bsky.richtext.facet#link";
