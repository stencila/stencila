//! The paths of nodes, by their ids, are those of the occurrences that a
//! comparison reports.

use eyre::{OptionExt, Result};
use stencila_node_compare::{compare, node_paths};
use stencila_schema::{
    Article, Block, Node,
    shortcuts::{p, t},
};

#[test]
fn paths_are_those_of_the_alignment() -> Result<()> {
    let article = Article::new(vec![p([t("First")]), p([t("Second")])]);
    let [Block::Paragraph(first), Block::Paragraph(second)] = article.content.as_slice() else {
        eyre::bail!("expected two paragraphs")
    };
    let (first, second) = (first.node_id(), second.node_id());
    let node = Node::Article(article.clone());

    let paths = node_paths(&node)?;
    let path =
        |id| -> Result<String> { Ok(paths.get(id).ok_or_eyre("expected a path")?.to_string()) };
    assert_eq!(path(&first)?, "content/0");
    assert_eq!(path(&second)?, "content/1");

    let comparison = compare(&node, &node)?;
    assert!(
        comparison
            .alignment()
            .pairs()
            .any(|(left, ..)| left.path.to_string() == "content/1")
    );
    Ok(())
}
