//! Tests of the `ProbeNode` derive
//!
//! Probing traverses directly to a [`NodePath`] within a node, which every path-based
//! consumer of the schema relies on.

use stencila_codec_text_trait::to_text;

use stencila_node_path::{NodePath, NodeSlot};
use stencila_node_type::NodeProperty;
use stencila_schema::{Article, ArticleOptions, DateTime, NodeSet, get, shortcuts::t};

/// A path reaches a property held in a generated `*Options` struct
///
/// The derive pops the property slot before matching it, so it has to put the slot back
/// before delegating to the options struct. Without that, no optional property is
/// reachable by path at all.
#[test]
fn a_path_reaches_an_options_property() {
    let article = Article {
        options: Box::new(ArticleOptions {
            date_accepted: Some(DateTime::new("2020-05-01T09:00:00".to_string())),
            ..Default::default()
        }),
        ..Article::new(vec![])
    };

    let path = NodePath::from([NodeSlot::Property(NodeProperty::DateAccepted)]);
    let Ok(NodeSet::One(node)) = get(&article, path) else {
        panic!("Expected to reach the `dateAccepted` property");
    };

    assert_eq!(to_text(&node), "2020-05-01T09:00:00");

    // A core property, which the derive matches directly, still resolves
    let article = Article {
        title: Some(vec![t("A title")]),
        ..Article::new(vec![])
    };
    let path = NodePath::from([NodeSlot::Property(NodeProperty::Title)]);
    let Ok(NodeSet::Many(nodes)) = get(&article, path) else {
        panic!("Expected to reach the `title` property");
    };
    assert_eq!(to_text(&nodes), "A title");
}
