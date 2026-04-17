use serde::Serialize;
use std::collections::HashMap;

use inflector::Inflector;
use pandoc_types::definition::{self as pandoc, Target};
use stencila_codec::{
    Losses, NodeProperty, NodeType,
    stencila_format::Format,
    stencila_schema::{
        Author, DateTime, NodePath, NodePosition, NodeSlot, StripNode, SuggestionType,
        node_url_file, node_url_jzb64, node_url_path,
    },
};

/// The context for encoding to Pandoc AST
pub(super) struct PandocEncodeContext {
    /// The format to encode
    pub format: Format,

    /// Encode the outputs, rather than the source, of executable nodes
    pub render: bool,

    /// Highlight the rendered outputs of executable nodes
    pub highlight: bool,

    /// Encode such that changes in the encoded document can be applied back to its source
    pub reproducible: bool,

    /// Encode paragraphs as Pandoc `Plain` blocks in places
    /// like figure and table captions.
    pub paragraph_to_plain: bool,

    /// An enumeration of the losses while encoding
    pub losses: Losses,

    /// The path to the current node
    pub node_path: NodePath,

    /// The repository URL from the root Article (for file link URLs)
    pub repository: Option<String>,

    /// The commit hash from the root Article (for file link URLs)
    pub commit: Option<String>,

    /// Mapping from boundary ids to encoded Pandoc comment start spans.
    pub comment_start_spans: HashMap<String, pandoc::Inline>,

    /// Mapping from boundary ids to encoded Pandoc comment end spans.
    pub comment_end_spans: HashMap<String, pandoc::Inline>,
}

impl PandocEncodeContext {
    pub fn new(format: Format, render: bool, highlight: bool, reproducible: bool) -> Self {
        Self {
            format,
            render,
            highlight,
            reproducible,
            paragraph_to_plain: false,
            losses: Losses::default(),
            node_path: NodePath::new(),
            repository: None,
            commit: None,
            comment_start_spans: HashMap::new(),
            comment_end_spans: HashMap::new(),
        }
    }

    /// Whether the target format is a flavor of DOCX
    pub fn is_docx_flavor(&self) -> bool {
        matches!(self.format, Format::Docx | Format::GDocx | Format::M365Docx)
    }

    /// Run an encoding function within the scope of a node property
    ///
    /// Modifies the context's node path before and after executing the function
    /// so that calls to `reproducible_link` contain the correct `path` field
    pub fn within_property<F, T>(&mut self, property: NodeProperty, func: F) -> T
    where
        F: Fn(&mut Self) -> T,
    {
        self.node_path.push_back(NodeSlot::from(property));
        let result = func(self);
        self.node_path.pop_back();
        result
    }

    /// Run an encoding function within the scope of a node index
    pub fn within_index<F, T>(&mut self, index: usize, mut func: F) -> T
    where
        F: FnMut(&mut Self) -> T,
    {
        self.node_path.push_back(NodeSlot::from(index));
        let result = func(self);
        self.node_path.pop_back();
        result
    }

    /// Create a [`pandoc::Inline::Link`] with a [`NodeUrl`] allowing the node to be reconstituted at a later time
    pub fn reproducible_link<T>(
        &mut self,
        node_type: NodeType,
        node: &T,
        position: Option<NodePosition>,
        content: pandoc::Inline,
    ) -> pandoc::Inline
    where
        T: Serialize + Clone + StripNode,
    {
        let style = match self.highlight {
            true => "Reproducible Highlighted",
            false => "Reproducible",
        };

        let span = pandoc::Inline::Span(
            attrs_attributes(vec![("custom-style".into(), style.into())]),
            vec![content],
        );

        let url = if matches!(self.format, Format::GDocx | Format::M365Docx) {
            match node_url_jzb64(node_type, node, position) {
                Ok(url) => url,
                Err(error) => {
                    tracing::error!("While encoding node url: {error}");
                    node_url_path(node_type, self.node_path.clone(), position)
                }
            }
        } else {
            node_url_path(node_type, self.node_path.clone(), position)
        };

        let url = url.to_string();
        let title = node_type.to_string().to_sentence_case();

        pandoc::Inline::Link(attrs_empty(), vec![span], Target { url, title })
    }

    /// Convert a local file path to a stencila.link URL
    ///
    /// Returns the target unchanged if it already has a URL scheme or is an anchor.
    /// For local file paths, creates a stencila.link URL with the context's
    /// repository and commit for enabling GitHub permalinks.
    pub fn file_url(&self, target: &str) -> String {
        node_url_file(target, self.repository.clone(), self.commit.clone()).to_string()
    }

    /// Create a [`pandoc::Inline::Span`] for an output
    pub fn output_span(&self, content: pandoc::Inline) -> pandoc::Inline {
        let style = match self.highlight {
            true => "Output Highlighted",
            false => "Output",
        };

        pandoc::Inline::Span(
            attrs_attributes(vec![("custom-style".into(), style.into())]),
            vec![content],
        )
    }
}

pub(super) fn append_suggestion_attrs(
    attrs: &mut pandoc::Attr,
    authors: &Option<Vec<Author>>,
    date_published: &Option<DateTime>,
) {
    if let Some(author) = authors.as_ref().map(|authors| {
        authors
            .iter()
            .map(|author| author.name())
            .collect::<Vec<_>>()
            .join(";")
    }) {
        attrs.attributes.push(("author".into(), author));
    }

    if let Some(date) = date_published
        .as_ref()
        .map(|date_time| date_time.value.clone())
    {
        attrs.attributes.push(("date".into(), date));
    }
}

pub(super) fn decode_suggestion_attrs(attrs: &pandoc::Attr) -> DecodedSuggestionAttrs {
    let suggestion_type = suggestion_type_from_attrs(attrs);

    let authors = get_attr(attrs, "author").map(|names| {
        names
            .split(';')
            .flat_map(|name| name.parse().ok())
            .collect()
    });

    let date_published = get_attr(attrs, "date").and_then(|date| date.parse().ok());

    DecodedSuggestionAttrs {
        suggestion_type,
        authors,
        date_published,
    }
}

pub(super) fn suggestion_type_from_attrs(attrs: &pandoc::Attr) -> SuggestionType {
    if attrs.classes.iter().any(|class| class == "deletion") {
        SuggestionType::Delete
    } else {
        SuggestionType::Insert
    }
}

/// Data collected from a Pandoc comment-start span during decoding
pub(super) struct PendingComment {
    /// The Pandoc comment id (e.g. "0", "1")
    pub pandoc_id: String,
    /// The comment author name
    pub author: Option<String>,
    /// The comment date string
    pub date: Option<String>,
    /// The comment body as Pandoc inlines (may contain LineBreak)
    pub body_inlines: Vec<pandoc::Inline>,
    /// The Pandoc id of the parent comment (set when this is a reply)
    pub parent_pandoc_id: Option<String>,
}

/// The context for decoding from Pandoc AST
#[derive(Default)]
pub(super) struct PandocDecodeContext {
    pub format: Format,
    pub losses: Losses,
    pub pending_comments: Vec<PendingComment>,
}

pub(super) struct DecodedSuggestionAttrs {
    pub suggestion_type: SuggestionType,
    pub authors: Option<Vec<Author>>,
    pub date_published: Option<DateTime>,
}

/// Create an empty Pandoc `Attr` tuple
pub(super) fn attrs_empty() -> pandoc::Attr {
    pandoc::Attr::default()
}

/// Create an empty Pandoc `Attr` tuple
pub(super) fn attrs_classes(classes: Vec<String>) -> pandoc::Attr {
    pandoc::Attr {
        classes,
        ..Default::default()
    }
}

/// Create an empty Pandoc `Attr` tuple
pub(super) fn attrs_attributes(attributes: Vec<(String, String)>) -> pandoc::Attr {
    pandoc::Attr {
        attributes,
        ..Default::default()
    }
}

/// Get an attribute from a Pandoc `Attr` tuple struct
pub(super) fn get_attr(attrs: &pandoc::Attr, name: &str) -> Option<String> {
    match name {
        "id" => match attrs.identifier.is_empty() {
            true => None,
            false => Some(attrs.identifier.clone()),
        },
        "classes" => match attrs.classes.is_empty() {
            true => None,
            false => Some(attrs.classes.join(" ")),
        },
        _ => attrs.attributes.iter().find_map(|(key, value)| {
            if key == name {
                Some(value.clone())
            } else {
                None
            }
        }),
    }
}
