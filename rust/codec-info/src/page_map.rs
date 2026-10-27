use serde::Serialize;
use serde_with::skip_serializing_none;

use stencila_node_id::NodeId;
use stencila_node_type::NodeType;

/// A mapping between nodes and the regions of the pages of a source where they are
///
/// For sources that are laid out on pages, such as a PDF or a TEI document with page
/// coordinates from the PDF it was parsed from. This is the paged counterpart of a
/// [`Mapping`](crate::Mapping) between nodes and character positions in a textual source.
/// Page regions are information about the source, not the content, so they are
/// kept beside the decoded node rather than within it.
#[derive(Debug, Default, Clone, PartialEq, Serialize)]
#[serde(transparent)]
pub struct PageMap {
    entries: Vec<PageMapEntry>,
}

/// An entry in a [`PageMap`]
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageMapEntry {
    /// The type of the node
    pub node_type: NodeType,

    /// The id of the node
    pub node_id: NodeId,

    /// The part of the node that the boxes are of, if not the whole node
    ///
    /// For example, `persName` for the name of a person, or `marker` for the
    /// marker of a footnote rather than its content.
    pub part: Option<String>,

    /// The boxes on the pages that the node, or part of it, occupies
    pub boxes: Vec<PageBox>,
}

/// A box on a page
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PageBox {
    /// The page number, starting at 1
    pub page: u32,

    /// The distance of the left of the box from the left of the page, in points
    pub x: f64,

    /// The distance of the top of the box from the top of the page, in points
    pub y: f64,

    /// The width of the box, in points
    pub width: f64,

    /// The height of the box, in points
    pub height: f64,
}

impl PageMap {
    /// Create an empty page map
    ///
    /// Equivalent to [`PageMap::default`] but provided to make it more explicit
    /// when a codec provides no page map (i.e. it returns `PageMap::none()`)
    pub fn none() -> Self {
        Self::default()
    }

    /// Add a new entry, unless there are no boxes
    pub fn add(
        &mut self,
        node_type: NodeType,
        node_id: NodeId,
        part: Option<String>,
        boxes: Vec<PageBox>,
    ) {
        if boxes.is_empty() {
            return;
        }

        self.entries.push(PageMapEntry {
            node_type,
            node_id,
            part,
            boxes,
        })
    }

    /// Whether there are no entries
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the entries in the page map
    pub fn entries(&self) -> &[PageMapEntry] {
        &self.entries
    }

    /// Retain only the entries for which a predicate is true
    ///
    /// Used to remove the entries for nodes that were decoded but then dropped.
    pub fn retain(&mut self, predicate: impl FnMut(&PageMapEntry) -> bool) {
        self.entries.retain(predicate)
    }

    /// Get the entries for a node
    pub fn entries_for<'map>(
        &'map self,
        node_id: &'map NodeId,
    ) -> impl Iterator<Item = &'map PageMapEntry> {
        self.entries
            .iter()
            .filter(move |entry| &entry.node_id == node_id)
    }
}
