//! The paths of nodes, by their ids, are those of the occurrences that a
//! comparison reports.

use eyre::{OptionExt, Result};
use stencila_node_compare::{compare, node_paths};
use stencila_schema::{
    Article, Author, AuthorRole, AuthorRoleName, Block, Node, Person,
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

#[test]
fn grouped_authors_keep_the_first_source_path_and_all_wrapper_paths() -> Result<()> {
    let person = Person {
        given_names: Some(vec!["Ada".into()]),
        family_names: Some(vec!["Lovelace".into()]),
        ..Default::default()
    };
    let person_id = person.node_id();
    let software = AuthorRole::person(person.clone(), AuthorRoleName::Software);
    let validation = AuthorRole::person(person, AuthorRoleName::Validation);
    let (software_id, validation_id) = (software.node_id(), validation.node_id());
    let node = Node::Article(Article {
        authors: Some(vec![
            Author::AuthorRole(software),
            Author::AuthorRole(validation),
        ]),
        ..Default::default()
    });
    let paths = node_paths(&node)?;
    assert_eq!(
        paths
            .get(&person_id)
            .ok_or_eyre("Missing person")?
            .to_string(),
        "authors/0/author"
    );
    assert_eq!(
        paths
            .get(&software_id)
            .ok_or_eyre("Missing Software wrapper")?
            .to_string(),
        "authors/0"
    );
    assert_eq!(
        paths
            .get(&validation_id)
            .ok_or_eyre("Missing Validation wrapper")?
            .to_string(),
        "authors/1"
    );
    let comparison = compare(&node, &node)?;
    assert!(
        comparison
            .alignment()
            .pairs()
            .any(|(left, ..)| Some(&left.path) == paths.get(&person_id))
    );
    Ok(())
}
