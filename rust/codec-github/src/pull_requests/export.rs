//! Extract comment-bearing Stencila documents into GitHub pull request exports.
//!
//! This module performs the pure, serializable part of the `ghpr` pipeline:
//! it derives repository provenance, encodes the reviewed source text, resolves
//! source ranges for comments and suggestions, and normalizes them into a
//! transport structure that the push layer can later submit to GitHub.

use std::ops::Range;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    mem,
};

use serde::Serialize;
use serde_with::skip_serializing_none;
use stencila_codec::stencila_schema::{SuggestionStatus, SuggestionType};
use stencila_codec::{
    Mapping, NodeId, NodeMapEntry, NodeProperty, NodeType, Positions,
    eyre::Result,
    stencila_format::Format,
    stencila_schema::{
        Author, Block, Boundary, Comment, Inline, Node, Section, SuggestionBlock, SuggestionInline,
        Visitor, WalkControl,
    },
};
use stencila_codec_markdown_trait::{MarkdownCodec, MarkdownEncodeContext, MarkdownEncodeMode};

use crate::pull_requests::source::PullRequestSource;

use super::source::pull_request_source;

/// The top-level output of the pull request export pipeline.
///
/// Captures everything needed to submit a GitHub pull request from a
/// comment-bearing Stencila document: source provenance, target context,
/// the original source content with mapping, normalized pull request comments,
/// and any diagnostics produced during extraction and resolution.
///
/// Designed to be serializable for snapshot testing and debugging.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestExport {
    pub source: PullRequestSource,
    pub target: PullRequestTarget,
    pub content: PullRequestSourceContent,
    pub items: Vec<PullRequestComment>,
    pub diagnostics: Vec<PullRequestExportDiagnostic>,
}

enum SuggestionRangePreference {
    Node,
    Content,
    Original,
}

fn author_name(authors: Option<&[Author]>) -> Option<String> {
    authors
        .and_then(|authors| authors.first())
        .map(Author::name)
        .filter(|name| !name.trim().is_empty())
}

fn suggestion_original_inlines(suggestion: &SuggestionInline) -> Option<&[Inline]> {
    suggestion.original.as_deref()
}

fn suggestion_original_blocks(suggestion: &SuggestionBlock) -> Option<&[Block]> {
    suggestion.original.as_deref()
}

/// The GitHub pull request submission context.
///
/// Empty in pure exports; populated progressively as target selection,
/// branch creation, anchor commit creation, and PR creation proceed.
/// Separating this from [`PullRequestSource`] prevents conflating source
/// provenance with submission state.
#[skip_serializing_none]
#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestTarget {
    pub repository: Option<String>,
    pub base_branch: Option<String>,
    pub head_branch: Option<String>,
    pub pull_request_number: Option<u64>,
    pub pull_request_branch: Option<String>,
    pub anchor_commit: Option<String>,
    pub side: Option<PullRequestSide>,
}

/// Which side of a GitHub pull request diff a comment anchors to.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PullRequestSide {
    Left,
    Right,
}

/// The original source-format content and its node mapping.
///
/// `text` is the source content (usually Markdown). `mapping` is derived
/// from [`Mapping`] and records which byte ranges in the source correspond
/// to which document nodes — used for position resolution and debugging.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestSourceContent {
    pub text: String,
    pub mapping: Option<Vec<NodeMapEntry>>,
}

/// Whether a pull request comment originated from a `Comment` or `SuggestionInline` node.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PullRequestCommentKind {
    Comment,
    Suggestion,
}

/// A normalized pull request comment derived from either a `Comment`,
/// `SuggestionInline`, or `SuggestionBlock` node.
///
/// For comments, `content` holds the comment text and `replacement_text` is
/// `None`. For suggestions, `replacement_text` holds the proposed content and
/// `content` is `None`. `selected_text` is the original source text at the
/// resolved range (when resolution succeeds).
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestComment {
    pub kind: PullRequestCommentKind,
    pub resolution: PullRequestCommentResolution,
    
    pub source_path: Option<String>,
    pub author_name: Option<String>,
    pub node_id: Option<String>,
    pub parent_node_id: Option<String>,
    pub range: PullRequestCommentRange,
    pub selected_text: Option<String>,
    pub preceding_text: Option<String>,
    pub replacement_text: Option<String>,
    pub content: Option<String>,
    pub suggestion_type: Option<SuggestionType>,
    pub suggestion_status: Option<SuggestionStatus>,
    pub github_suggestion: Option<GitHubSuggestion>,
}

/// Source-relative position of a pull request comment.
///
/// Distinguishes original metadata (`start_location`/`end_location` as raw
/// strings from `Comment` nodes) from resolved coordinates (`start_offset`,
/// `start_line`, etc.). Resolution proceeds through a staged pipeline:
/// explicit `file:line:column` locations, `#id` boundary references,
/// mapping-based node spans, and text refinement.
///
/// These are source-file coordinates, not GitHub diff-hunk anchors. GitHub
/// anchoring is derived later by the submission layer.
#[skip_serializing_none]
#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestCommentRange {
    pub start_location: Option<String>,
    pub end_location: Option<String>,
    pub start_offset: Option<usize>,
    pub end_offset: Option<usize>,
    pub start_line: Option<u32>,
    pub start_column: Option<u32>,
    pub end_line: Option<u32>,
    pub end_column: Option<u32>,
}

/// How confidently a pull request comment was anchored to source positions.
///
/// - `Anchored`: both byte offsets and line/column are resolved.
/// - `FallbackLine`: line/column known (from explicit location metadata)
///   but byte offsets are not available.
/// - `Unanchored`: no position could be determined; the item will degrade
///   to a non-inline pull request comment.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PullRequestCommentResolution {
    Anchored,
    FallbackLine,
    Unanchored,
}

/// Classification of the concrete edit operation a suggestion implies.
///
/// Derived from [`SuggestionType`] when available, otherwise inferred
/// from the relationship between `selected_text` and `replacement_text`.
/// Used to determine how the GitHub suggestion block is constructed.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SuggestionEditKind {
    /// New content inserted at the anchor point; no existing text replaced.
    Insert,
    /// Existing text removed; the replacement is empty.
    Delete,
    /// Existing text replaced with different content.
    Replace,
}

/// A GitHub-ready suggestion block derived from a resolved [`PullRequestComment`].
///
/// Contains the whole-line expansion needed for GitHub's suggestion
/// comment format. GitHub suggestion blocks replace entire lines
/// (`start_line` through `end_line` inclusive) with `replacement_lines`.
///
/// The `body` field holds the formatted Markdown body including the
/// fenced suggestion block, ready to be submitted as a pull request comment.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitHubSuggestion {
    /// The kind of edit this suggestion represents.
    pub edit_kind: SuggestionEditKind,
    /// 1-based start line of the suggestion (whole-line expanded).
    pub start_line: u32,
    /// 1-based end line of the suggestion (whole-line expanded).
    pub end_line: u32,
    /// The full replacement text for the covered lines (without trailing newline).
    pub replacement_lines: String,
    /// The formatted suggestion block body ready for GitHub submission.
    pub body: String,
}

/// A diagnostic emitted during pull request export or position resolution.
///
/// Diagnostics provide visibility into resolution failures without
/// aborting the export. They are serialized alongside the pull request comments
/// for snapshot debugging and test assertions.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PullRequestExportDiagnostic {
    pub level: PullRequestExportDiagnosticLevel,
    pub code: String,
    pub message: String,
    pub item_node_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PullRequestExportDiagnosticLevel {
    Warning,
    Error,
}

/// Convert a comment-bearing document node into a serializable [`PullRequestExport`].
///
/// This is a pure conversion with no network I/O. It extracts `Comment` and
/// `SuggestionInline` nodes into normalized [`PullRequestComment`]s and resolves their
/// source positions using explicit location metadata and/or the provided
/// [`Mapping`]. The resulting export can be snapshot-tested, inspected for
/// debugging, and later fed into the GitHub submission layer.
pub fn export_pull_request(
    node: &Node,
    source_text: &str,
    format: Format,
    mapping: Option<&Mapping>,
) -> Result<PullRequestExport> {
    let (source_text, mapping) =
        pull_request_source_content(node, source_text, format.clone(), mapping)?;
    let source = pull_request_source(node, format);

    // Pre-walks: collect indices used by the staged resolver
    let boundary_index = BoundaryIndex::build(node);
    let suggestion_fallback_context = SuggestionFallbackContext::build(node);

    let mut extractor = PullRequestExtractor::new(
        source.path.clone(),
        &source_text,
        mapping.as_ref(),
        &boundary_index,
        &suggestion_fallback_context,
    );

    for comment in root_comments(node) {
        let item = extractor.resolve_comment(comment);
        extractor.push_item(item);
    }

    extractor.walk(node);

    // Build GitHub suggestion blocks for precisely anchored suggestion items.
    // Items that fell back to a coarse parent range or encoded syntax range are
    // excluded — they would produce degenerate suggestion blocks.
    for item in &mut extractor.items {
        if matches!(item.kind, PullRequestCommentKind::Suggestion)
            && matches!(item.resolution, PullRequestCommentResolution::Anchored)
        {
            let has_coarse_diagnostic = extractor.diagnostics.iter().any(|d| {
                d.item_node_id.as_deref() == item.node_id.as_deref()
                    && (d.code == "coarse-parent-range" || d.code == "suggestion-syntax-range")
            });
            if !has_coarse_diagnostic {
                item.github_suggestion = build_github_suggestion(&source_text, item);
            }
        }
    }

    let content_mapping = mapping.as_ref().map(|mapping| mapping.to_nodemap(None));

    let mut export = PullRequestExport {
        source,
        target: PullRequestTarget::default(),
        content: PullRequestSourceContent {
            text: source_text.clone(),
            mapping: content_mapping,
        },
        items: extractor.items,
        diagnostics: extractor.diagnostics,
    };

    normalize_node_ids(&mut export);

    Ok(export)
}

fn root_comments(node: &Node) -> &[Comment] {
    // Work-level comments live under metadata options and are not traversed by
    // the generated walker, so collect them explicitly here.
    match node {
        Node::Article(article) => article.options.comments.as_deref().unwrap_or_default(),
        Node::Datatable(datatable) => datatable.options.comments.as_deref().unwrap_or_default(),
        Node::SoftwareSourceCode(code) => code.options.comments.as_deref().unwrap_or_default(),
        _ => &[],
    }
}

fn pull_request_source_content(
    node: &Node,
    source_text: &str,
    format: Format,
    mapping: Option<&Mapping>,
) -> Result<(String, Option<Mapping>)> {
    if !contains_review_markup(node) {
        return Ok((source_text.to_string(), mapping.cloned()));
    }

    let mut context = MarkdownEncodeContext::new(Some(format), Some(MarkdownEncodeMode::Clean));
    node.to_markdown(&mut context);
    if context.content.ends_with("\n\n") {
        context.content.pop();
    }

    Ok((context.content, Some(context.mapping)))
}

fn contains_review_markup(node: &Node) -> bool {
    !root_comments(node).is_empty() || review_markup_in_node(node)
}

fn review_markup_in_node(node: &Node) -> bool {
    struct Finder {
        found: bool,
    }

    impl Visitor for Finder {
        fn visit_node(&mut self, node: &Node) -> WalkControl {
            if matches!(node, Node::Comment(_)) {
                self.found = true;
                WalkControl::Break
            } else {
                WalkControl::Continue
            }
        }

        fn visit_suggestion_inline(&mut self, _inline: &SuggestionInline) -> WalkControl {
            self.found = true;
            WalkControl::Break
        }
    }

    let mut finder = Finder { found: false };
    finder.walk(node);
    finder.found
}

/// Maps `Boundary.id` values (e.g. `"comment-0-start"`) to their [`NodeId`]s,
/// allowing `#id`-style location references to be resolved via the [`Mapping`].
///
/// Built by a pre-walk over the document before pull request export extraction begins.
struct BoundaryIndex {
    /// boundary.id -> boundary NodeId
    id_to_node_id: HashMap<String, NodeId>,
}

impl BoundaryIndex {
    fn build(node: &Node) -> Self {
        let mut collector = BoundaryCollector {
            id_to_node_id: HashMap::new(),
        };
        collector.walk(node);
        BoundaryIndex {
            id_to_node_id: collector.id_to_node_id,
        }
    }

    /// Look up the char index where content begins after a start-boundary marker.
    ///
    /// A start boundary like `comment-0-start` delimits the beginning of the
    /// annotated content. The content starts at the END of the marker's range.
    fn resolve_content_start(&self, boundary_id: &str, mapping: &Mapping) -> Option<usize> {
        let node_id = self.id_to_node_id.get(boundary_id)?;
        let range = mapping.range_of_node(node_id)?;
        Some(range.end)
    }

    /// Look up the char index where content ends before an end-boundary marker.
    ///
    /// An end boundary like `comment-0-end` delimits the end of the annotated
    /// content. The content ends at the START of the marker's range.
    fn resolve_content_end(&self, boundary_id: &str, mapping: &Mapping) -> Option<usize> {
        let node_id = self.id_to_node_id.get(boundary_id)?;
        let range = mapping.range_of_node(node_id)?;
        Some(range.start)
    }
}

struct BoundaryCollector {
    id_to_node_id: HashMap<String, NodeId>,
}

impl Visitor for BoundaryCollector {
    fn visit_inline(&mut self, inline: &Inline) -> WalkControl {
        if let Inline::Boundary(Boundary {
            id: Some(id), uid, ..
        }) = inline
        {
            let node_id = NodeId::new(b"bdy", uid);
            self.id_to_node_id.insert(id.clone(), node_id);
        }
        WalkControl::Continue
    }
}

/// Pre-computed text context used only by fallback suggestion anchoring.
///
/// The preferred anchoring strategy for suggestions is to use the mapping
/// generated while encoding review markup in clean Markdown mode. In that
/// mode, suggestion nodes map directly to the reviewed source span:
///
/// - insert suggestions map to the insertion point itself
/// - delete suggestions map to the content being removed
/// - replace suggestions map to the original content being replaced
///
/// This context exists only for the fallback strategy used when that direct
/// mapping is missing or insufficient. For each suggestion node it stores the
/// encoded sibling content that precedes the suggestion within the same parent,
/// so a best-effort insertion point can be recovered by text matching.
///
/// Built by a direct tree walk (not the `Visitor` trait) so we can inspect
/// inline and block content vectors directly.
struct SuggestionFallbackContext {
    /// suggestion UID → preceding sibling text content
    preceding: HashMap<String, String>,
    /// suggestion UID → preceding sibling block content encoded as markdown
    preceding_blocks: HashMap<String, String>,
}

impl SuggestionFallbackContext {
    fn build(node: &Node) -> Self {
        let mut preceding = HashMap::new();
        let mut preceding_blocks = HashMap::new();

        // Pull request exports currently operate on article documents, so only
        // article block content is traversed here. If additional root node
        // types gain review export support in future, extend this entry point
        // to walk their block content too.
        if let Node::Article(article) = node {
            Self::collect_blocks(&article.content, &mut preceding, &mut preceding_blocks);
        }

        SuggestionFallbackContext {
            preceding,
            preceding_blocks,
        }
    }

    fn collect_blocks(
        blocks: &[Block],
        preceding: &mut HashMap<String, String>,
        preceding_blocks: &mut HashMap<String, String>,
    ) {
        let mut blocks_before: Vec<Block> = Vec::new();

        for block in blocks {
            match block {
                Block::SuggestionBlock(suggestion) => {
                    let uid = suggestion.node_id().uid_str().to_string();
                    preceding_blocks.insert(uid, blocks_to_markdown(&blocks_before));
                }
                _ => {
                    Self::collect_block(block, preceding, preceding_blocks);
                    blocks_before.push(block.clone());
                }
            }
        }
    }

    fn collect_block(
        block: &Block,
        preceding: &mut HashMap<String, String>,
        preceding_blocks: &mut HashMap<String, String>,
    ) {
        match block {
            Block::Paragraph(para) => Self::collect_inlines(&para.content, preceding),
            Block::Section(Section { content, .. }) => {
                Self::collect_blocks(content, preceding, preceding_blocks);
            }
            _ => {}
        }
    }

    fn collect_inlines(inlines: &[Inline], preceding: &mut HashMap<String, String>) {
        let mut text_before = String::new();
        for inline in inlines {
            match inline {
                Inline::SuggestionInline(suggestion) => {
                    let uid = suggestion.node_id().uid_str().to_string();
                    preceding.insert(uid, text_before.clone());
                    // Don't add suggestion content to text_before: for Insert
                    // suggestions the content is new text that doesn't appear
                    // in the source. For Delete, refine_with_text_match handles
                    // resolution separately, so omitting here is safe.
                }
                Inline::Text(text) => {
                    text_before.push_str(&text.value);
                }
                _ => {
                    // Other inline types (Emphasis, Strong, etc.) are skipped.
                    // This is conservative: we may fail to find the insertion
                    // point (and fall back to coarse range) but will never
                    // produce a wrong anchor.
                }
            }
        }
    }
}

/// Walks a document tree to extract pull request comments and resolve their positions.
///
/// Implements the [`Visitor`] trait to collect `Comment` and `SuggestionInline`
/// nodes during a depth-first walk. Maintains a node-id stack for parent
/// resolution and a dedup set to prevent root-level comments (collected
/// before the walk) from being emitted again if the walker also visits them.
struct PullRequestExtractor<'source, 'mapping, 'boundaries, 'insert> {
    root_path: Option<String>,
    source_text: &'source str,
    mapping: Option<&'mapping Mapping>,
    boundary_index: &'boundaries BoundaryIndex,
    suggestion_fallback_context: &'insert SuggestionFallbackContext,
    /// Node-id stack tracking the current walk depth, for parent resolution.
    stack: Vec<String>,
    /// Node IDs already emitted, to prevent duplicates from root-level collection.
    seen_items: HashSet<String>,
    items: Vec<PullRequestComment>,
    diagnostics: Vec<PullRequestExportDiagnostic>,
}

impl<'source, 'mapping, 'boundaries, 'insert>
    PullRequestExtractor<'source, 'mapping, 'boundaries, 'insert>
{
    /// Create an extractor that prefers direct mapping-based anchoring and
    /// retains fallback context only for degraded suggestion resolution.
    fn new(
        root_path: Option<String>,
        source_text: &'source str,
        mapping: Option<&'mapping Mapping>,
        boundary_index: &'boundaries BoundaryIndex,
        suggestion_fallback_context: &'insert SuggestionFallbackContext,
    ) -> Self {
        Self {
            root_path,
            source_text,
            mapping,
            boundary_index,
            suggestion_fallback_context,
            stack: Vec::new(),
            seen_items: HashSet::new(),
            items: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    fn push_item(&mut self, item: PullRequestComment) {
        if let Some(node_id) = &item.node_id
            && !self.seen_items.insert(node_id.clone())
        {
            return;
        }

        self.items.push(item);
    }

    fn range_from_char_offsets(&self, offsets: Range<usize>) -> PullRequestCommentRange {
        let mut range = PullRequestCommentRange::default();
        apply_offsets(self.source_text, &mut range, offsets);
        range
    }

    fn push_warning(&mut self, code: &str, message: &str, item_node_id: &str) {
        self.diagnostics.push(PullRequestExportDiagnostic {
            level: PullRequestExportDiagnosticLevel::Warning,
            code: code.into(),
            message: message.into(),
            item_node_id: Some(item_node_id.to_string()),
        });
    }

    fn missing_mapping_range(&mut self, node_id: &str) -> PullRequestCommentRange {
        self.push_warning(
            "missing-mapping",
            "No mapping available for suggestion position resolution",
            node_id,
        );
        PullRequestCommentRange::default()
    }

    fn coarse_parent_range(
        &mut self,
        node_id: &str,
        parent_offsets: Range<usize>,
    ) -> PullRequestCommentRange {
        let range = self.range_from_char_offsets(parent_offsets);
        self.push_warning(
            "coarse-parent-range",
            "Suggestion resolved to parent node range, not exact target",
            node_id,
        );
        range
    }

    fn missing_direct_suggestion_mapping_range(
        &mut self,
        node_id: &str,
    ) -> PullRequestCommentRange {
        self.push_warning(
            "missing-direct-suggestion-mapping",
            "Suggestion node did not resolve through clean-markdown mapping and no parent fallback was available",
            node_id,
        );
        PullRequestCommentRange::default()
    }

    fn resolve_parent_node_id(&self, current: Option<&str>) -> Option<String> {
        match (self.stack.last(), current) {
            (Some(last), Some(current)) if last == current => {
                self.stack.get(self.stack.len().saturating_sub(2)).cloned()
            }
            _ => self.stack.last().cloned(),
        }
    }

    fn resolve_comment(&mut self, comment: &Comment) -> PullRequestComment {
        let node_id = comment.node_id().uid_str().to_string();
        let range = self.resolve_range(
            Some(&node_id),
            comment.options.start_location.as_deref(),
            comment.options.end_location.as_deref(),
            None,
        );

        PullRequestComment {
            kind: PullRequestCommentKind::Comment,
            source_path: source_path_from_location(comment.options.start_location.as_deref())
                .or_else(|| comment.options.path.clone())
                .or_else(|| self.root_path.clone()),
            author_name: author_name(comment.authors.as_deref()),
            node_id: Some(node_id.clone()),
            parent_node_id: self.resolve_parent_node_id(Some(&node_id)),
            selected_text: slice_text(self.source_text, &range),
            preceding_text: None,
            replacement_text: None,
            content: Some(blocks_to_markdown(&comment.content)),
            suggestion_type: None,
            suggestion_status: None,
            resolution: resolution_for_range(&range),
            github_suggestion: None,
            range,
        }
    }

    /// Resolve a suggestion's target range using the parent node's mapped span.
    ///
    /// Suggestions don't carry their own source location metadata. Instead,
    /// the resolver finds the enclosing (parent) node's source range from the
    /// mapping, then attempts to locate the suggestion's content within that
    /// parent span using local text matching.
    fn resolve_suggestion(&mut self, suggestion: &SuggestionInline) -> PullRequestComment {
        let node_id = suggestion.node_id().uid_str().to_string();
        let replacement_text = inlines_to_markdown(&suggestion.content);
        let parent_uid = self.resolve_parent_node_id(Some(&node_id));

        // Try to resolve via parent node's mapped range
        let range = self.resolve_suggestion_range(&node_id, parent_uid.as_deref(), suggestion);

        PullRequestComment {
            kind: PullRequestCommentKind::Suggestion,
            source_path: self.root_path.clone(),
            author_name: author_name(suggestion.authors.as_deref()),
            node_id: Some(node_id.clone()),
            parent_node_id: parent_uid,
            selected_text: slice_text(self.source_text, &range),
            preceding_text: self
                .suggestion_fallback_context
                .preceding
                .get(&node_id)
                .cloned(),
            replacement_text: Some(replacement_text),
            content: None,
            suggestion_type: suggestion.suggestion_type,
            suggestion_status: suggestion.suggestion_status,
            resolution: resolution_for_range(&range),
            github_suggestion: None,
            range,
        }
    }

    fn resolve_suggestion_block(&mut self, suggestion: &SuggestionBlock) -> PullRequestComment {
        let node_id = suggestion.node_id().uid_str().to_string();
        let replacement_text = blocks_to_markdown(&suggestion.content);
        let parent_uid = self.resolve_parent_node_id(Some(&node_id));
        let range =
            self.resolve_suggestion_block_range(&node_id, parent_uid.as_deref(), suggestion);

        PullRequestComment {
            kind: PullRequestCommentKind::Suggestion,
            source_path: self.root_path.clone(),
            author_name: author_name(suggestion.authors.as_deref()),
            node_id: Some(node_id.clone()),
            parent_node_id: parent_uid,
            selected_text: if suggestion.suggestion_type == Some(SuggestionType::Insert) {
                Some(String::new())
            } else {
                slice_text(self.source_text, &range)
            },
            preceding_text: None,
            replacement_text: Some(replacement_text),
            content: None,
            suggestion_type: suggestion.suggestion_type,
            suggestion_status: suggestion.suggestion_status,
            resolution: resolution_for_range(&range),
            github_suggestion: None,
            range,
        }
    }

    fn resolve_suggestion_block_range(
        &mut self,
        node_id: &str,
        parent_uid: Option<&str>,
        suggestion: &SuggestionBlock,
    ) -> PullRequestCommentRange {
        let Some(_mapping) = self.mapping else {
            return self.missing_mapping_range(node_id);
        };

        if suggestion.suggestion_type == Some(SuggestionType::Insert)
            && let Some(offsets) = self.resolve_github_compatible_block_insert_range(node_id)
        {
            return self.range_from_char_offsets(offsets);
        }

        if suggestion.suggestion_type == Some(SuggestionType::Insert)
            && let Some(parent_offsets) = self.resolve_mapped_node_range_by_uid(parent_uid)
            && let Some(offsets) =
                self.refine_block_insert_position_with_fallback_context(node_id, parent_offsets)
        {
            return self.range_from_char_offsets(offsets);
        }

        // Preferred strategy: use the direct clean-markdown mapping of the
        // suggestion node itself. In clean mode, suggestion nodes map to the
        // reviewed source span rather than their review-markup syntax.
        if let Some(offsets) = self.resolve_optimal_suggestion_block_range(node_id, suggestion) {
            return self.range_from_char_offsets(offsets);
        }

        // Fallback strategy: use the enclosing parent span and refine within it
        // using text matching or preceding-sibling context. This is retained as
        // a compatibility path when direct mapping is absent or incomplete.
        let parent_range = self.resolve_mapped_node_range_by_uid(parent_uid);

        if let Some(parent_offsets) = parent_range {
            if suggestion.suggestion_type != Some(SuggestionType::Insert)
                && let Some(offsets) = self
                    .refine_block_range_with_fallback_text_match(parent_offsets.clone(), suggestion)
            {
                return self.range_from_char_offsets(offsets);
            }

            if suggestion.suggestion_type == Some(SuggestionType::Insert)
                && let Some(offsets) = self.refine_block_insert_position_with_fallback_context(
                    node_id,
                    parent_offsets.clone(),
                )
            {
                return self.range_from_char_offsets(offsets);
            }

            return self.coarse_parent_range(node_id, parent_offsets);
        }

        self.missing_direct_suggestion_mapping_range(node_id)
    }

    /// Resolve a suggestion's source range.
    ///
    /// Preferred strategy: use the direct clean-markdown mapping recorded for
    /// the suggestion node itself. In clean mode, suggestion nodes map to the
    /// reviewed source span: insertions to an insertion point, deletions to the
    /// deleted content, and replacements to the original content.
    ///
    /// When property-level mapping is available, we refine the node-level span
    /// further to align with GitHub review semantics:
    ///
    /// - inserts use the end of the preceding sibling content when available,
    ///   otherwise the clean-mapped node span
    /// - deletes prefer the `Content` property and trim a trailing newline so the
    ///   anchor targets the deleted content rather than the following block break
    /// - replaces prefer the `Original` property for the same reason
    ///
    /// Fallback strategy: if that direct mapping is unavailable, use the mapped
    /// parent range and attempt to refine within it using text matching or
    /// preceding-sibling context.
    fn resolve_suggestion_range(
        &mut self,
        node_id: &str,
        parent_uid: Option<&str>,
        suggestion: &SuggestionInline,
    ) -> PullRequestCommentRange {
        let Some(_mapping) = self.mapping else {
            return self.missing_mapping_range(node_id);
        };

        if suggestion.suggestion_type == Some(SuggestionType::Insert)
            && let Some(parent_offsets) = self.resolve_mapped_node_range_by_uid(parent_uid)
            && let Some(insert_pos) =
                self.refine_inline_insert_position_with_fallback_context(node_id, parent_offsets)
        {
            return self.range_from_char_offsets(insert_pos);
        }

        if let Some(offsets) = self.resolve_optimal_suggestion_inline_range(node_id, suggestion) {
            return self.range_from_char_offsets(offsets);
        }

        let parent_range = self.resolve_mapped_node_range_by_uid(parent_uid);

        if let Some(parent_offsets) = parent_range {
            if suggestion.suggestion_type != Some(SuggestionType::Insert) {
                let refined = self.refine_inline_range_with_fallback_text_match(
                    parent_offsets.clone(),
                    suggestion,
                );
                if let Some(offsets) = refined {
                    return self.range_from_char_offsets(offsets);
                }
            }

            if suggestion.suggestion_type == Some(SuggestionType::Insert)
                && let Some(insert_pos) = self.refine_inline_insert_position_with_fallback_context(
                    node_id,
                    parent_offsets.clone(),
                )
            {
                return self.range_from_char_offsets(insert_pos);
            }

            // Fall back to the full parent range
            return self.coarse_parent_range(node_id, parent_offsets);
        }

        self.missing_direct_suggestion_mapping_range(node_id)
    }

    /// Resolve the clean-markdown-mapped range of a suggestion node by UID.
    fn resolve_suggestion_node_range(&self, node_uid: &str) -> Option<Range<usize>> {
        let offsets = self.resolve_mapped_node_range_by_uid(Some(node_uid))?;

        if offsets.start > offsets.end {
            let anchor = offsets.end.saturating_add(1);
            return Some(anchor..anchor);
        }

        Some(offsets)
    }

    /// Resolve the preferred range for an inline suggestion using direct mapping.
    fn resolve_optimal_suggestion_inline_range(
        &self,
        node_uid: &str,
        suggestion: &SuggestionInline,
    ) -> Option<Range<usize>> {
        self.resolve_optimal_suggestion_range(node_uid, suggestion.suggestion_type)
    }

    /// Resolve a GitHub-compatible range for a block insertion suggestion.
    ///
    /// GitHub suggestions operate on line replacements. For inserted blocks we
    /// want the anchor to sit on the blank separator line that follows the new
    /// block content so that the suggestion inserts a new block, rather than
    /// replacing text on the preceding content line.
    fn resolve_github_compatible_block_insert_range(&self, node_uid: &str) -> Option<Range<usize>> {
        let node_range = self.resolve_suggestion_node_range(node_uid)?;
        let start_byte = char_index_to_byte(self.source_text, node_range.start);
        let tail = self.source_text.get(start_byte..)?;
        let relative = tail.find("\n\n")?;
        let anchor_byte = start_byte + relative + 2;
        let anchor_char = self.source_text[..anchor_byte].chars().count();

        Some(anchor_char..anchor_char)
    }

    /// Resolve the preferred range for a block suggestion using direct mapping.
    fn resolve_optimal_suggestion_block_range(
        &self,
        node_uid: &str,
        suggestion: &SuggestionBlock,
    ) -> Option<Range<usize>> {
        self.resolve_optimal_suggestion_range(node_uid, suggestion.suggestion_type)
    }

    fn resolve_optimal_suggestion_range(
        &self,
        node_uid: &str,
        suggestion_type: Option<SuggestionType>,
    ) -> Option<Range<usize>> {
        let preference = match suggestion_type {
            Some(SuggestionType::Delete) => SuggestionRangePreference::Content,
            Some(SuggestionType::Replace) => SuggestionRangePreference::Original,
            _ => SuggestionRangePreference::Node,
        };

        self.resolve_suggestion_range_by_preference(node_uid, preference)
    }

    fn resolve_suggestion_range_by_preference(
        &self,
        node_uid: &str,
        preference: SuggestionRangePreference,
    ) -> Option<Range<usize>> {
        match preference {
            SuggestionRangePreference::Node => self.resolve_suggestion_node_range(node_uid),
            SuggestionRangePreference::Content => self
                .resolve_mapped_property_range_by_uid(node_uid, NodeProperty::Content)
                .map(|range| trim_trailing_newline_range(self.source_text, range))
                .or_else(|| self.resolve_suggestion_node_range(node_uid)),
            SuggestionRangePreference::Original => self
                .resolve_mapped_property_range_by_uid(node_uid, NodeProperty::Original)
                .map(|range| trim_trailing_newline_range(self.source_text, range))
                .or_else(|| self.resolve_suggestion_node_range(node_uid)),
        }
    }

    /// Resolve a mapped node range by exported UID.
    ///
    /// The mapping stores internal `NodeId` values, while pull request items use
    /// the external UID string. This helper bridges that gap so callers can use
    /// direct mapping lookup without re-scanning or re-implementing the logic.
    fn resolve_mapped_node_range_by_uid(&self, node_uid: Option<&str>) -> Option<Range<usize>> {
        let mapping = self.mapping?;
        let node_uid = node_uid?;

        mapping
            .entries()
            .iter()
            .find(|entry| entry.node_id.uid_str() == node_uid && entry.property.is_none())
            .and_then(|entry| mapping.range_of_node(&entry.node_id))
    }

    /// Resolve a mapped property range by exported UID and property.
    fn resolve_mapped_property_range_by_uid(
        &self,
        node_uid: &str,
        property: NodeProperty,
    ) -> Option<Range<usize>> {
        let mapping = self.mapping?;

        mapping
            .entries()
            .iter()
            .find(|entry| entry.node_id.uid_str() == node_uid && entry.property == Some(property))
            .and_then(|entry| mapping.range_of_property(&entry.node_id, property))
    }

    fn refine_range_with_unique_text_match(
        &self,
        parent_char_offsets: Range<usize>,
        search_text: &str,
    ) -> Option<Range<usize>> {
        if search_text.is_empty() {
            return None;
        }

        let byte_start = char_index_to_byte(self.source_text, parent_char_offsets.start);
        let byte_end = char_index_to_byte(self.source_text, parent_char_offsets.end);
        let parent_text = self.source_text.get(byte_start..byte_end)?;

        let mut matches = parent_text.match_indices(search_text);
        let first = matches.next()?;
        if matches.next().is_some() {
            return None;
        }

        let chars_before_match = parent_text[..first.0].chars().count();
        let match_char_len = search_text.chars().count();
        let abs_start = parent_char_offsets.start + chars_before_match;
        let abs_end = abs_start + match_char_len;
        Some(abs_start..abs_end)
    }

    /// Attempt to find the suggestion's content text within the
    /// parent's mapped source span (for delete/replace suggestions).
    ///
    /// The content *is* the text to be deleted/replaced, so a match gives
    /// the exact target range. Only returns a match if it is unique within
    /// the parent span to avoid mis-anchoring on repeated phrases.
    fn refine_inline_range_with_fallback_text_match(
        &self,
        parent_char_offsets: Range<usize>,
        suggestion: &SuggestionInline,
    ) -> Option<Range<usize>> {
        let search_text = suggestion_original_inlines(suggestion).map_or_else(
            || inlines_to_markdown(&suggestion.content),
            inlines_to_markdown,
        );

        self.refine_range_with_unique_text_match(parent_char_offsets, &search_text)
    }

    fn refine_block_insert_position_with_fallback_context(
        &self,
        node_id: &str,
        parent_char_offsets: Range<usize>,
    ) -> Option<Range<usize>> {
        let preceding = self
            .suggestion_fallback_context
            .preceding_blocks
            .get(node_id)?;

        let node_offsets = self.resolve_suggestion_node_range(node_id)?;

        if node_offsets.start == node_offsets.end {
            return Some(node_offsets);
        }

        let byte_start = char_index_to_byte(self.source_text, parent_char_offsets.start);
        let byte_end = char_index_to_byte(self.source_text, parent_char_offsets.end);
        let parent_text = self.source_text.get(byte_start..byte_end)?;

        if preceding.is_empty() {
            return Some(parent_char_offsets.end..parent_char_offsets.end);
        }

        let mut matches = parent_text.match_indices(preceding.as_str());
        let first = matches.next()?;
        if matches.next().is_some() {
            return None;
        }

        let chars_to_insert_point = parent_text[..first.0 + preceding.len()].chars().count();
        let abs = parent_char_offsets.start + chars_to_insert_point;
        Some(abs..abs)
    }

    fn refine_block_range_with_fallback_text_match(
        &self,
        parent_char_offsets: Range<usize>,
        suggestion: &SuggestionBlock,
    ) -> Option<Range<usize>> {
        let search_text = suggestion_original_blocks(suggestion).map_or_else(
            || blocks_to_markdown(&suggestion.content),
            blocks_to_markdown,
        );

        self.refine_range_with_unique_text_match(parent_char_offsets, &search_text)
    }

    /// Find the insertion point for an insert suggestion by
    /// locating where the preceding sibling text ends in the source.
    ///
    /// For insert suggestions the content to be inserted does not exist
    /// in the original source, so content-based text matching cannot
    /// work. Instead, we use the pre-computed [`SuggestionFallbackContext`] to find
    /// the text that appears immediately before the suggestion in the
    /// document tree, search for it within the parent's source span,
    /// and place a zero-width insertion point right after it.
    ///
    /// Returns `None` if the preceding text cannot be uniquely located,
    /// causing resolution to fall back to the coarse parent range.
    fn refine_inline_insert_position_with_fallback_context(
        &self,
        node_id: &str,
        parent_char_offsets: Range<usize>,
    ) -> Option<Range<usize>> {
        let preceding = self.suggestion_fallback_context.preceding.get(node_id)?;

        let byte_start = char_index_to_byte(self.source_text, parent_char_offsets.start);
        let byte_end = char_index_to_byte(self.source_text, parent_char_offsets.end);
        let parent_text = self.source_text.get(byte_start..byte_end)?;

        if preceding.is_empty() {
            // No preceding text → insertion at the start of the parent
            return Some(parent_char_offsets.start..parent_char_offsets.start);
        }

        // Search for the preceding text within the parent's source span
        let mut matches = parent_text.match_indices(preceding.as_str());
        let first = matches.next()?;
        if matches.next().is_some() {
            // Ambiguous — multiple matches
            return None;
        }

        // Insertion point is right after the preceding text
        let chars_to_insert_point = parent_text[..first.0 + preceding.len()].chars().count();
        let insert_char = parent_char_offsets.start + chars_to_insert_point;
        Some(insert_char..insert_char)
    }

    /// Resolve a [`PullRequestCommentRange`] using a staged strategy:
    ///
    /// 1. If `start_location`/`end_location` contain `file:line:column`,
    ///    use them directly (highest confidence).
    /// 2. If locations are `#id` boundary references, resolve via the
    ///    [`BoundaryIndex`] and [`Mapping`] to get byte offsets.
    /// 3. Otherwise, look up the node in the [`Mapping`] by node id and
    ///    optional property to get byte offsets, then convert to line/column.
    /// 4. If no mapping is available, emit a diagnostic and return unresolved.
    fn resolve_range(
        &mut self,
        node_id: Option<&str>,
        start_location: Option<&str>,
        end_location: Option<&str>,
        property: Option<NodeProperty>,
    ) -> PullRequestCommentRange {
        let mut range = PullRequestCommentRange {
            start_location: start_location.map(str::to_string),
            end_location: end_location.map(str::to_string),
            ..Default::default()
        };

        // Stage 1: explicit file:line:column
        if let Some((_, line, column)) = parse_location(start_location) {
            range.start_line = Some(line);
            range.start_column = Some(column);
        }
        if let Some((_, line, column)) = parse_location(end_location) {
            range.end_line = Some(line);
            range.end_column = Some(column);
        }
        if range.start_line.is_some() || range.end_line.is_some() {
            return range;
        }

        // Stage 2: #id boundary references — require BOTH sides to resolve.
        // If only one side resolves, substituting 0 or file-length for the
        // missing side would produce a bogus file-spanning range. Instead,
        // emit a diagnostic and fall through to Stage 3.
        if let Some(mapping) = self.mapping {
            let has_start_ref = parse_boundary_ref(start_location).is_some();
            let has_end_ref = parse_boundary_ref(end_location).is_some();

            let start_offset = parse_boundary_ref(start_location)
                .and_then(|id| self.boundary_index.resolve_content_start(id, mapping));
            let end_offset = parse_boundary_ref(end_location)
                .and_then(|id| self.boundary_index.resolve_content_end(id, mapping));

            match (start_offset, end_offset) {
                (Some(s), Some(e)) => {
                    apply_offsets(self.source_text, &mut range, s..e);
                    return range;
                }
                (Some(_), None) | (None, Some(_)) => {
                    let missing = if start_offset.is_none() {
                        "start"
                    } else {
                        "end"
                    };
                    self.diagnostics.push(PullRequestExportDiagnostic {
                        level: PullRequestExportDiagnosticLevel::Warning,
                        code: "partial-boundary".into(),
                        message: format!(
                            "Only the {missing} boundary could be resolved; \
                             falling back to node lookup"
                        ),
                        item_node_id: node_id.map(str::to_string),
                    });
                    // Fall through to Stage 3
                }
                (None, None) if has_start_ref || has_end_ref => {
                    self.diagnostics.push(PullRequestExportDiagnostic {
                        level: PullRequestExportDiagnosticLevel::Warning,
                        code: "unresolved-boundary".into(),
                        message: "Boundary reference(s) could not be resolved".into(),
                        item_node_id: node_id.map(str::to_string),
                    });
                }
                _ => {}
            }
        }

        // Stage 3: mapping-based node lookup
        let Some(mapping) = self.mapping else {
            if node_id.is_some() {
                self.diagnostics.push(PullRequestExportDiagnostic {
                    level: PullRequestExportDiagnosticLevel::Warning,
                    code: "missing-mapping".into(),
                    message: "No mapping available for review position resolution".into(),
                    item_node_id: node_id.map(str::to_string),
                });
            }
            return range;
        };

        let Some(node_id) = node_id else {
            return range;
        };

        let internal_id = mapping
            .entries()
            .iter()
            .find(|entry| entry.node_id.uid_str() == node_id && entry.property == property)
            .map(|entry| entry.node_id.clone());

        let offsets = match (internal_id, property) {
            (Some(id), Some(property)) => mapping.range_of_property(&id, property),
            (Some(id), None) => mapping.range_of_node(&id),
            _ => None,
        };

        if let Some(offsets) = offsets {
            apply_offsets(self.source_text, &mut range, offsets);
        }

        range
    }
}

impl Visitor for PullRequestExtractor<'_, '_, '_, '_> {
    fn enter_struct(&mut self, _node_type: NodeType, node_id: NodeId) -> WalkControl {
        self.stack.push(node_id.uid_str().to_string());
        WalkControl::Continue
    }

    fn exit_struct(&mut self) {
        let _ = self.stack.pop();
    }

    fn visit_node(&mut self, node: &Node) -> WalkControl {
        if let Node::Comment(comment) = node {
            let item = self.resolve_comment(comment);
            self.push_item(item);
        }

        WalkControl::Continue
    }

    fn visit_suggestion_inline(&mut self, inline: &SuggestionInline) -> WalkControl {
        let item = self.resolve_suggestion(inline);
        self.push_item(item);
        WalkControl::Continue
    }

    fn visit_suggestion_block(&mut self, block: &SuggestionBlock) -> WalkControl {
        let item = self.resolve_suggestion_block(block);
        self.push_item(item);
        WalkControl::Continue
    }
}

fn blocks_to_markdown(blocks: &[Block]) -> String {
    let mut context = MarkdownEncodeContext::default();
    for block in blocks {
        block.to_markdown(&mut context);
    }
    trim_markdown(context.content)
}

fn inlines_to_markdown(inlines: &[Inline]) -> String {
    let mut context = MarkdownEncodeContext::default();
    for inline in inlines {
        inline.to_markdown(&mut context);
    }
    trim_markdown(context.content)
}

fn trim_markdown(mut markdown: String) -> String {
    while markdown.ends_with('\n') {
        markdown.pop();
    }

    markdown
}

/// Parse a `file:line:column` location string, returning `(path, line, column)`.
///
/// Uses `rsplitn` to handle file paths that may themselves contain colons.
/// Returns `None` for `#id`-style boundary references or malformed strings.
fn parse_location(location: Option<&str>) -> Option<(String, u32, u32)> {
    let location = location?;
    let mut parts = location.rsplitn(3, ':');
    let column = parts.next()?.parse::<u32>().ok()?;
    let line = parts.next()?.parse::<u32>().ok()?;
    let path = parts.next()?.to_string();
    Some((path, line, column))
}

fn source_path_from_location(location: Option<&str>) -> Option<String> {
    parse_location(location).map(|(path, ..)| path)
}

/// Extract the boundary ID from a `#id`-style location reference.
///
/// Returns `Some("comment-0-start")` for `"#comment-0-start"`, `None` otherwise.
fn parse_boundary_ref(location: Option<&str>) -> Option<&str> {
    location.and_then(|loc| loc.strip_prefix('#'))
}

/// Convert character indices to 1-based line/column positions and byte offsets,
/// then populate the range.
///
/// The `char_offsets` parameter contains UTF-8 character indices (as returned
/// by [`Mapping`]). These are converted to byte offsets for storage in
/// `start_offset`/`end_offset` so that downstream consumers can use them
/// directly for string slicing.
fn apply_offsets(
    source_text: &str,
    range: &mut PullRequestCommentRange,
    char_offsets: Range<usize>,
) {
    let positions = Positions::new(source_text);
    let start = positions.position8_at_index(char_offsets.start);
    let end = positions.position8_at_index(char_offsets.end);

    range.start_offset = Some(char_index_to_byte(source_text, char_offsets.start));
    range.end_offset = Some(char_index_to_byte(source_text, char_offsets.end));
    range.start_line = Some(start.line as u32 + 1);
    range.start_column = Some(start.column as u32 + 1);
    range.end_line = Some(end.line as u32 + 1);
    range.end_column = Some(end.column as u32 + 1);
}

/// Convert a UTF-8 character index to a byte offset.
///
/// Returns `source_text.len()` when `char_index` is at or past the end
/// of the string (i.e. the one-past-end position).
fn char_index_to_byte(source_text: &str, char_index: usize) -> usize {
    source_text
        .char_indices()
        .nth(char_index)
        .map(|(byte_off, _)| byte_off)
        .unwrap_or(source_text.len())
}

fn slice_text(source_text: &str, range: &PullRequestCommentRange) -> Option<String> {
    let start = range.start_offset?;
    let end = range.end_offset?;
    source_text.get(start..end).map(str::to_string)
}

/// Trim trailing newline characters from a mapped character range when present.
///
/// Clean-markdown mapping for block content can legitimately include the line
/// break that separates a block from whatever follows. For GitHub inline review
/// anchors we generally want the content span itself, excluding that separator.
fn trim_trailing_newline_range(source_text: &str, range: Range<usize>) -> Range<usize> {
    if range.start >= range.end {
        return range;
    }

    let mut end = range.end;

    while end > range.start {
        let end_byte = char_index_to_byte(source_text, end);
        let start_byte = char_index_to_byte(source_text, end.saturating_sub(1));

        match source_text.get(start_byte..end_byte) {
            Some("\n") => end = end.saturating_sub(1),
            _ => break,
        }
    }

    range.start..end
}

/// Classify a range's resolution confidence based on which fields were populated.
fn resolution_for_range(range: &PullRequestCommentRange) -> PullRequestCommentResolution {
    if range.start_offset.is_some() && range.start_line.is_some() {
        PullRequestCommentResolution::Anchored
    } else if range.start_line.is_some() {
        PullRequestCommentResolution::FallbackLine
    } else {
        PullRequestCommentResolution::Unanchored
    }
}

/// Classify the concrete edit operation for a suggestion item.
///
/// Uses [`SuggestionType`] when available, otherwise infers from the
/// relationship between `selected_text` and `replacement_text`.
fn classify_suggestion_edit(item: &PullRequestComment) -> SuggestionEditKind {
    match item.suggestion_type {
        Some(SuggestionType::Insert) => SuggestionEditKind::Insert,
        Some(SuggestionType::Delete) => SuggestionEditKind::Delete,
        Some(SuggestionType::Replace) => SuggestionEditKind::Replace,
        _ => {
            // Infer from text content
            match (&item.selected_text, &item.replacement_text) {
                (None | Some(_), Some(r)) if r.is_empty() => SuggestionEditKind::Delete,
                (None, Some(_)) => SuggestionEditKind::Insert,
                _ => SuggestionEditKind::Replace,
            }
        }
    }
}

/// Expand byte offsets to the boundaries of the lines they fall on.
///
/// Returns `(line_start_byte, line_end_byte)` where `line_start_byte` is
/// the byte offset of the first character on the start line and
/// `line_end_byte` is the byte offset just past the last non-newline
/// character on the end line (i.e. before the trailing `\n`, if any).
fn expand_to_line_boundaries(source_text: &str, start: usize, end: usize) -> (usize, usize) {
    let line_start = source_text[..start]
        .rfind('\n')
        .map(|pos| pos + 1)
        .unwrap_or(0);

    let line_end = source_text[end..]
        .find('\n')
        .map(|pos| end + pos)
        .unwrap_or(source_text.len());

    (line_start, line_end)
}

/// Build a [`GitHubSuggestion`] from a resolved suggestion [`PullRequestComment`].
///
/// Returns `None` if the item lacks the byte offsets or line/column
/// information needed to construct a whole-line suggestion block (i.e.
/// items with `Unanchored` or `FallbackLine` resolution).
///
/// The suggestion block is constructed by:
/// 1. Classifying the edit as insert/delete/replace.
/// 2. Expanding the resolved byte range to cover whole source lines.
/// 3. Replacing only the selected portion within those lines according
///    to the edit kind.
/// 4. Formatting the result as a GitHub suggestion Markdown block.
fn build_github_suggestion(
    source_text: &str,
    item: &PullRequestComment,
) -> Option<GitHubSuggestion> {
    let start_offset = item.range.start_offset?;
    let end_offset = item.range.end_offset?;
    let start_line = item.range.start_line?;
    let end_line = item.range.end_line?;
    let replacement_text = item.replacement_text.as_deref()?;

    let edit_kind = classify_suggestion_edit(item);

    if edit_kind == SuggestionEditKind::Insert
        && item.selected_text.as_deref() == Some("")
        && item.preceding_text.is_none()
    {
        let body = format!("```suggestion\n{replacement_text}\n```");

        return Some(GitHubSuggestion {
            edit_kind,
            start_line,
            end_line,
            replacement_lines: replacement_text.to_string(),
            body,
        });
    }

    let (line_start, line_end) = expand_to_line_boundaries(source_text, start_offset, end_offset);

    // Safety: if offsets are within source_text, line boundaries will be too
    let full_lines = source_text.get(line_start..line_end)?;
    let prefix = full_lines.get(..start_offset - line_start)?;
    let suffix = full_lines.get(end_offset - line_start..)?;

    let replacement_lines = match edit_kind {
        SuggestionEditKind::Insert => {
            // Insert: add content at the anchor point
            format!("{prefix}{replacement_text}{suffix}")
        }
        SuggestionEditKind::Delete => {
            // Delete: remove the selected text, replacement_text is the content
            // being deleted (from SuggestionInline.content), not the replacement
            format!("{prefix}{suffix}")
        }
        SuggestionEditKind::Replace => {
            // Replace: swap selected text with replacement
            format!("{prefix}{replacement_text}{suffix}")
        }
    };

    let body = format!("```suggestion\n{replacement_lines}\n```");

    Some(GitHubSuggestion {
        edit_kind,
        start_line,
        end_line,
        replacement_lines,
        body,
    })
}

/// Replace runtime-generated node IDs with deterministic sequential IDs
/// (`node-1`, `node-2`, ...) so that snapshot tests produce stable output
/// regardless of execution order or random ID generation.
fn normalize_node_ids(export: &mut PullRequestExport) {
    let mut ids = BTreeMap::new();
    let mut next = 1usize;

    if let Some(mapping) = &mut export.content.mapping {
        for entry in mapping {
            let id = mem::take(&mut entry.node_id);
            entry.node_id = normalize_node_id(id, &mut ids, &mut next);
        }
    }

    for item in &mut export.items {
        normalize_node_id_option(&mut item.node_id, &mut ids, &mut next);
        normalize_node_id_option(&mut item.parent_node_id, &mut ids, &mut next);
    }

    for diagnostic in &mut export.diagnostics {
        normalize_node_id_option(&mut diagnostic.item_node_id, &mut ids, &mut next);
    }
}

fn normalize_node_id_option(
    value: &mut Option<String>,
    ids: &mut BTreeMap<String, String>,
    next: &mut usize,
) {
    if let Some(id) = value.take() {
        *value = Some(normalize_node_id(id, ids, next));
    }
}

fn normalize_node_id(id: String, ids: &mut BTreeMap<String, String>, next: &mut usize) -> String {
    if let Some(normalized) = ids.get(&id) {
        return normalized.clone();
    }

    let normalized = format!("node-{next}");
    *next += 1;
    ids.insert(id, normalized.clone());
    normalized
}
