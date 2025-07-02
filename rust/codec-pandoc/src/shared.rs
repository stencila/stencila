use codec::{
    common::{inflector::Inflector, serde::Serialize, tracing},
    format::Format,
    schema::{node_url_jzb64, node_url_path, NodePath, NodePosition, NodeSlot, StripNode},
    Losses, NodeProperty, NodeType,
};
use pandoc_types::definition::{self as pandoc, Attr, Target};

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
    node_path: NodePath,
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
        }
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

    /// Create a Pandoc link with a [`NodeUrl`] allowing the node to be reconstituted at a later time
    pub fn reproducible_link<T>(
        &mut self,
        node_type: NodeType,
        node: &T,
        position: Option<NodePosition>,
        attrs: Attr,
        content: Vec<pandoc::Inline>,
    ) -> pandoc::Inline
    where
        T: Serialize + Clone + StripNode,
    {
        let url = if matches!(self.format, Format::GDocx) {
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

        pandoc::Inline::Link(attrs, content, Target { url, title })
    }
}

/// The context for decoding from Pandoc AST
#[derive(Default)]
pub(super) struct PandocDecodeContext {
    pub format: Format,
    pub losses: Losses,
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
