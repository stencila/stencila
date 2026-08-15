//! Versioned JSON snapshots of representative alignment and comparison artifacts
//!
//! The fixtures are ordinary Stencila-native document structures, so a snapshot change
//! is a change in this crate rather than in any codec.

use eyre::Result;

use stencila_node_compare::{align, compare};
use stencila_schema::{
    Article, Block, Figure, Node, Paragraph, Person, Reference, Section,
    shortcuts::{art, h1, p, sec, t, tbl, td, th, tr},
};

/// A paragraph with an explicit id
fn identified(id: &str, text: &str) -> Block {
    Block::Paragraph(Paragraph {
        id: Some(id.to_string()),
        ..Paragraph::new(vec![t(text)])
    })
}

/// A section with an explicit id
fn identified_section(id: &str, content: Vec<Block>) -> Block {
    Block::Section(Section {
        id: Some(id.to_string()),
        ..Section::new(content)
    })
}

/// An article with authors and references
fn scholarly(title: &str, sentence: &str) -> Node {
    let mut article = Article::new(vec![
        h1([t(title)]),
        p([t(sentence)]),
        Block::Figure(Figure {
            caption: Some(vec![p([t("Figure 1. An illustration")])]),
            ..Figure::new(vec![p([t("The figure content")])])
        }),
        tbl([
            tr([th([t("Sample")]), th([t("Value")])]),
            tr([td([t("A")]), td([t("1")])]),
        ]),
    ]);
    article.authors = Some(vec![stencila_schema::Author::Person(Person {
        family_names: Some(vec!["Adams".to_string()]),
        ..Person::default()
    })]);
    article.references = Some(vec![Reference {
        id: Some("ref-one".to_string()),
        title: Some(vec![t("An earlier study")]),
        ..Reference::default()
    }]);

    Node::Article(article)
}

/// An alignment of an insertion, a deletion and a local reordering
#[test]
fn alignment_of_edited_siblings() -> Result<()> {
    let left = art([
        p([t("A distinctive opening sentence")]),
        p([t("Repeated boilerplate")]),
        p([t("A distinctive closing sentence")]),
    ]);
    let right = art([
        p([t("Repeated boilerplate")]),
        p([t("A distinctive opening sentence")]),
        p([t("An entirely new sentence")]),
    ]);

    insta::assert_json_snapshot!(align(&left, &right)?);

    Ok(())
}

/// A comparison of an edited scholarly article
#[test]
fn comparison_of_an_edited_article() -> Result<()> {
    let left = scholarly(
        "A study of something",
        "The measured effect was substantial.",
    );
    let right = scholarly(
        "A study of something else",
        "The measured effect was modest.",
    );

    insta::assert_json_snapshot!(compare(&left, &right)?);

    Ok(())
}

/// A comparison of a subtree that moved to a different parent
#[test]
fn comparison_of_a_cross_parent_move() -> Result<()> {
    let left = art([
        identified_section(
            "first",
            vec![
                identified("kept", "A sentence that stays where it is"),
                identified("moved", "A sentence that moves elsewhere"),
            ],
        ),
        identified_section(
            "second",
            vec![identified("other", "A sentence in the other section")],
        ),
    ]);
    let right = art([
        identified_section(
            "first",
            vec![identified("kept", "A sentence that stays where it is")],
        ),
        identified_section(
            "second",
            vec![
                identified("other", "A sentence in the other section"),
                identified("moved", "A sentence that moves elsewhere"),
            ],
        ),
    ]);

    insta::assert_json_snapshot!(compare(&left, &right)?);

    Ok(())
}

/// A comparison of a cross-type pair whose content is unchanged
#[test]
fn comparison_of_a_cross_type_pair() -> Result<()> {
    let left = art([sec([p([t("Words that do not change at all")])])]);
    let right = art([sec([h1([t("Words that do not change at all")])])]);

    insta::assert_json_snapshot!(compare(&left, &right)?);

    Ok(())
}
