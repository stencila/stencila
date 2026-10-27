use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use roxmltree::{Document, NS_XML_URI, Node, NodeId, ParsingOptions};

use stencila_codec::{
    DecodeInfo, DecodeOptions, Losses, PageMap,
    eyre::{OptionExt, Result, bail},
    stencila_schema::{
        Article, Block, InspectNode, Node as SchemaNode, NodeId as SchemaNodeId,
        PersonOrOrganization, Reference,
    },
};

mod back;
mod bibl;
mod coords;
mod floats;
mod header;
mod text;

use back::decode_back;
use header::decode_header;
use text::{Divisions, decode_body, decode_divisions, decode_unplaced_footnotes};

/// Decode a TEI XML string to a Stencila Schema [`Node`]
///
/// This is the main entry point for decoding. It parses the XML and then
/// traverses the `<TEI>` element, building an [`Article`] from it. Elements and
/// attributes that are not decoded are recorded as losses, as they are by the
/// JATS codec, so that `--input-losses` reports them.
pub fn decode(tei: &str, _options: Option<DecodeOptions>) -> Result<(SchemaNode, DecodeInfo)> {
    let dom = Document::parse_with_options(
        tei,
        ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        },
    )?;

    // The <TEI> element is usually the root but may be within a <teiCorpus>,
    // which is only supported if it has a single <TEI>, because a corpus of
    // several documents is not one article
    let root = if dom.root_element().has_tag_name("TEI") {
        dom.root_element()
    } else {
        let mut tei = dom
            .root()
            .descendants()
            .filter(|node| node.tag_name().name() == "TEI");
        let root = tei
            .next()
            .ok_or_eyre("XML document does not have a <TEI> element")?;
        if tei.next().is_some() {
            bail!("Decoding a <teiCorpus> with more than one <TEI> is not supported");
        }
        root
    };

    let mut context = Context::new(&root);
    let article = decode_tei(&root, &mut context);

    // Only nodes that are in the article are mapped, not any that were decoded
    // but then dropped
    if !context.carriers.is_empty() {
        let mut ids = HashSet::new();
        coords::node_ids(&article, &mut ids);
        context.drop_unkept(&ids);
    }

    let info = DecodeInfo {
        losses: context.losses,
        page_map: context.page_map,
        ..Default::default()
    };

    Ok((SchemaNode::Article(article), info))
}

/// Decode a `<TEI>` element to an [`Article`]
fn decode_tei(root: &Node, context: &mut Context) -> Article {
    // `xml:space` is ignored because GROBID sets it on the root but the
    // whitespace within prose is not significant
    context.lost_attrs(root, &["space", "schemaLocation"]);

    let mut article = Article::default();
    let mut body = Vec::new();
    let mut back = Vec::new();

    for child in root.children() {
        match child.tag_name().name() {
            "teiHeader" => decode_header(&child, &mut article, context),
            "text" => {
                context.lost_attrs(&child, &[]);

                // Older GROBID output (e.g. for patents) puts the divisions of
                // the text directly in <text> rather than in a <body>. Each run
                // of them is decoded together, so that their sections are
                // nested, and kept in order with the <body> and <back>.
                let mut divs = Vec::new();
                let mut in_back = false;
                for grandchild in child.children() {
                    let tag = grandchild.tag_name().name();
                    if grandchild.is_element() && tag == "div" {
                        divs.push(grandchild);
                        continue;
                    }
                    if !matches!(tag, "body" | "back") {
                        context.lost(&grandchild);
                        continue;
                    }

                    let part = if in_back { &mut back } else { &mut body };
                    part.append(&mut decode_divisions(
                        divs.drain(..),
                        0,
                        Divisions::Sections,
                        context,
                    ));

                    if tag == "body" {
                        body.append(&mut decode_body(&grandchild, context));
                    } else {
                        in_back = true;
                        back.append(&mut decode_back(&grandchild, &mut article, context));
                    }
                }
                let part = if in_back { &mut back } else { &mut body };
                part.append(&mut decode_divisions(
                    divs.into_iter(),
                    0,
                    Divisions::Sections,
                    context,
                ));
            }
            // The size of each page, needed only to interpret the coordinates
            // of elements on those pages, is not kept
            "facsimile" => context.lost(&child),
            _ => context.lost(&child),
        }
    }

    // Footnotes that are not referenced from the text go at the end of the
    // body, before the back matter, as a JATS <fn-group> would be
    body.append(&mut decode_unplaced_footnotes(context));

    article.content = body.into_iter().chain(back).collect::<Vec<Block>>();

    let references = std::mem::take(&mut context.references);
    if !references.is_empty() {
        article.references = Some(references);
    }

    article
}

/// The state of decoding
///
/// Holds the losses, and information gathered before decoding which is needed
/// to resolve references between elements regardless of their order.
pub(super) struct Context<'a, 'input> {
    /// Elements and attributes that were not decoded
    pub losses: Losses,

    /// The ids of all elements, used to check that the target of a reference exists
    ids: HashSet<String>,

    /// The footnotes of the text
    ///
    /// GROBID puts footnotes at the end of the body, so each is decoded where
    /// it is first referenced instead, and any that are not referenced are
    /// decoded at the end of the body.
    footnotes: Vec<Node<'a, 'input>>,

    /// The footnotes that have been decoded where they are referenced, or
    /// where they are, if within a paragraph
    placed_footnotes: HashSet<NodeId>,

    /// The funders of the header, by the id of each grant `<org>` that they
    /// refer to, so that the grants of the back matter can name their funders
    funders: Vec<(String, PersonOrOrganization)>,

    /// The references of the article, from each `<listBibl>` in the text
    ///
    /// GROBID puts the reference list in the back matter, but older output,
    /// such as for patents, has lists of references within the body.
    references: Vec<Reference>,

    /// The boxes on the pages of the PDF, from the `coords` of elements, where
    /// decoded nodes are
    page_map: PageMap,

    /// The elements whose `id` or `coords` were given to decoded nodes
    ///
    /// Used to record the attributes as lost if those nodes are later dropped.
    carriers: Vec<Carrier>,
}

/// An element whose `id` or `coords` were given to a decoded node
struct Carrier {
    /// The element
    element: NodeId,

    /// The path of the element, used as the label of any loss
    path: String,

    /// The id of the decoded node
    node_id: SchemaNodeId,

    /// Whether the node has the id of the element
    ///
    /// Not when the node is of only a part of the element, nor when it is one
    /// of several decoded from the element and the id is on another.
    id: bool,

    /// Whether the node has the boxes of the element's coordinates
    coords: bool,
}

/// Whether an attribute of an element was carried by any decoded node, and by
/// any that was kept
#[derive(Default)]
struct Carried {
    any: bool,
    kept: bool,
}

impl Carried {
    fn add(&mut self, carried: bool, kept: bool) {
        self.any |= carried;
        self.kept |= carried && kept;
    }

    fn is_lost(&self) -> bool {
        self.any && !self.kept
    }
}

impl<'a, 'input> Context<'a, 'input> {
    fn new(root: &Node<'a, 'input>) -> Self {
        let ids = root
            .descendants()
            .filter_map(|node| xml_id(&node))
            .map(String::from)
            .collect();

        let footnotes = root
            .descendants()
            .filter(|node| {
                node.has_tag_name("note")
                    && node.attribute("place") == Some("foot")
                    // Notes within a figure are its notes rather than footnotes of the text
                    && !node.ancestors().any(|ancestor| ancestor.has_tag_name("figure"))
            })
            .collect();

        Self {
            losses: Losses::none(),
            ids,
            footnotes,
            placed_footnotes: HashSet::new(),
            funders: Vec::new(),
            references: Vec::new(),
            page_map: PageMap::none(),
            carriers: Vec::new(),
        }
    }

    /// Whether an element with an id exists
    fn has_id(&self, id: &str) -> bool {
        self.ids.contains(id)
    }

    /// Record that a decoded node, or a part of it, is where an element is on
    /// the pages of the PDF, as given by the element's `coords`
    ///
    /// An element decoded to several nodes, such as a paragraph split around
    /// a list within it, is recorded for each of them. Coordinates that are
    /// malformed are recorded as lost.
    fn locate(&mut self, element: &Node, node: &dyn InspectNode, part: Option<&str>) {
        let id = xml_id(element).filter(|_| part.is_none());
        let coords = element.attribute("coords");
        if id.is_none() && coords.is_none() {
            return;
        }
        let (Some(node_type), Some(node_id)) = (node.node_type(), node.node_id()) else {
            return;
        };

        let boxes = coords.and_then(|coords| {
            let boxes = coords::page_boxes(coords);
            if boxes.is_none() {
                self.lost_label([&path(element), "/@coords (malformed)"].concat());
            }
            boxes
        });

        self.carriers.push(Carrier {
            element: element.id(),
            path: path(element),
            node_id: node_id.clone(),
            id: id.is_some_and(|id| coords::has_id(node, id)),
            coords: boxes.is_some(),
        });

        if let Some(boxes) = boxes {
            self.page_map
                .add(node_type, node_id, part.map(String::from), boxes)
        }
    }

    /// Remove the page map entries of nodes that were decoded but then dropped,
    /// such as empty footnotes, given the ids of the nodes that were kept
    ///
    /// The `id` and `coords` of an element are recorded as lost if no kept
    /// node has them.
    fn drop_unkept(&mut self, kept: &HashSet<SchemaNodeId>) {
        // For each element, whether each of its attributes was carried by any
        // node, and by any node that was kept
        let mut elements = HashMap::<NodeId, (&str, Carried, Carried)>::new();
        for carrier in &self.carriers {
            let is_kept = kept.contains(&carrier.node_id);
            let (_, id, coords) = elements.entry(carrier.element).or_insert((
                &carrier.path,
                Carried::default(),
                Carried::default(),
            ));
            id.add(carrier.id, is_kept);
            coords.add(carrier.coords, is_kept);
        }

        for (path, id, coords) in elements.into_values() {
            if id.is_lost() {
                self.losses.add([path, "/@id"].concat());
            }
            if coords.is_lost() {
                self.losses.add([path, "/@coords"].concat());
            }
        }

        self.page_map.retain(|entry| kept.contains(&entry.node_id));
    }

    /// Record that a decoded node is where an element is on the pages of the
    /// PDF, and return the node
    fn located<T: InspectNode>(&mut self, element: &Node, node: T) -> T {
        self.locate(element, &node, None);
        node
    }

    /// Get a footnote, by its id, to decode where it is referenced
    ///
    /// Returns `None` if there is no such footnote, or if it has already been
    /// decoded at an earlier reference to it.
    fn place_footnote(&mut self, id: &str) -> Option<Node<'a, 'input>> {
        let note = self
            .footnotes
            .iter()
            .find(|note| xml_id(note) == Some(id))
            .copied()?;

        self.placed_footnotes.insert(note.id()).then_some(note)
    }

    /// Record that a footnote is being decoded where it is
    ///
    /// Returns `false` if it has already been decoded at a reference to it.
    fn placed_footnote(&mut self, note: &Node) -> bool {
        self.placed_footnotes.insert(note.id())
    }

    /// Take the footnotes that have not been decoded elsewhere
    fn take_unplaced_footnotes(&mut self) -> Vec<Node<'a, 'input>> {
        std::mem::take(&mut self.footnotes)
            .into_iter()
            .filter(|note| !self.placed_footnotes.contains(&note.id()))
            .collect()
    }

    /// Record a loss with a label
    fn lost_label(&mut self, label: impl AsRef<str>) {
        self.losses.add(label)
    }

    /// Record that an XML node was lost
    ///
    /// Whitespace-only text, which arises from indentation, is not a loss.
    fn lost(&mut self, node: &Node) {
        if node.is_element() {
            self.losses.add(path(node))
        } else if node.is_text()
            && !node
                .text()
                .map(|text| text.trim().is_empty())
                .unwrap_or(true)
            && let Some(parent) = node.parent_element()
        {
            self.losses.add([&path(&parent), "/text()"].concat())
        }
    }

    /// Record that the attributes of an element, other than those that are
    /// decoded (named by their local names), were lost
    fn lost_attrs(&mut self, node: &Node, decoded: &[&str]) {
        for attribute in node.attributes() {
            let name = attribute.name();
            if !decoded.contains(&name) {
                self.losses.add([&path(node), "/@", name].concat());
            }
        }
    }

    /// Record that the attributes of an element, named by their local names,
    /// were lost, after having been passed to [`Self::lost_attrs`] as decoded
    ///
    /// For when it is only known after decoding an element that nothing
    /// carries them, as for its `id` and `coords` when it decodes to no node.
    fn lost_attrs_named(&mut self, node: &Node, names: &[&str]) {
        for attribute in node.attributes() {
            let name = attribute.name();
            if names.contains(&name) {
                self.losses.add([&path(node), "/@", name].concat());
            }
        }
    }
}

/// Get the XPath-like path of an element, used as the label of a loss
///
/// For example `//TEI/text/body/div/p/rs`.
fn path(node: &Node) -> String {
    let names = node
        .ancestors()
        .filter(Node::is_element)
        .map(|node| node.tag_name().name())
        .collect_vec();

    ["//", &names.into_iter().rev().join("/")].concat()
}

/// Get the id of an element
///
/// GROBID uses `xml:id` but older output, and other producers, use `id`.
pub(super) fn xml_id<'a>(node: &Node<'a, '_>) -> Option<&'a str> {
    node.attribute((NS_XML_URI, "id"))
        .or_else(|| node.attribute("id"))
}

/// Get the ids that the `target` of a `<ref>` or `<ptr>` points to
fn targets(node: &Node) -> Vec<String> {
    fragment_ids(node.attribute("target"))
}

/// Get the ids that a pointer attribute, such as `target` or `ref`, points to
///
/// A pointer is a space separated list of URIs, of which those that are
/// fragments (e.g. `#b0`) address elements in the same document.
fn fragment_ids(uris: Option<&str>) -> Vec<String> {
    uris.unwrap_or_default()
        .split_whitespace()
        .filter_map(|uri| uri.strip_prefix('#'))
        .map(String::from)
        .collect()
}

/// Collapse each run of whitespace in a string to a single space
///
/// TEI prose is not whitespace sensitive and GROBID's older output is
/// indented, so line breaks and indentation within it are not significant.
pub(super) fn collapse_whitespace(text: &str) -> String {
    let mut collapsed = String::with_capacity(text.len());
    let mut in_whitespace = false;
    for char in text.chars() {
        if char.is_whitespace() {
            if !in_whitespace {
                collapsed.push(' ');
            }
            in_whitespace = true;
        } else {
            collapsed.push(char);
            in_whitespace = false;
        }
    }
    collapsed
}

/// Get all the text within an element, with whitespace collapsed and trimmed
pub(super) fn text_of(node: &Node) -> String {
    let text = node
        .descendants()
        .filter(Node::is_text)
        .filter_map(|node| node.text())
        .collect::<String>();
    collapse_whitespace(&text).trim().to_string()
}

/// Get the text within an element if it has any
pub(super) fn non_empty_text(node: &Node) -> Option<String> {
    let text = text_of(node);
    (!text.is_empty()).then_some(text)
}

/// Get the first child element with a tag name
pub(super) fn child<'a, 'input>(node: &Node<'a, 'input>, tag: &str) -> Option<Node<'a, 'input>> {
    node.children().find(|child| child.has_tag_name(tag))
}
