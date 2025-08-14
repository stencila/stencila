use common::eyre::{Result, bail};
use common_dev::pretty_assertions::assert_eq;
use schema::{
    Article, Node,
    shortcuts::{ct, ctg, h1, li, mi, ol, p, t},
};

use crate::structuring;

#[test]
fn test_reference_list_to_references() -> Result<()> {
    // Single reference with DOI
    let mut article = Node::Article(Article::new(vec![
        h1([t("References")]),
        ol([li([t(
            "Author, A. B., & Author, C. D. (Year). Title of article. Journal Name, Volume(Issue), pages. 10.0000/xyz",
        )])]),
    ]));
    structuring(&mut article);
    let Node::Article(Article {
        references: Some(refs),
        ..
    }) = article
    else {
        bail!("Should have references")
    };
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].doi, Some("10.0000/xyz".into()));
    assert_eq!(refs[0].id, Some("ref-1".into()));

    // Multiple references with sequential IDs
    let mut article = Node::Article(Article::new(vec![
        h1([t("References")]),
        ol([
            li([t(
                "First Author. (2020). First paper. Journal A, 1(1), 1-10.",
            )]),
            li([t(
                "Second Author. (2021). Second paper. Journal B, 2(2), 11-20.",
            )]),
            li([t(
                "Third Author. (2022). Third paper. Journal C, 3(3), 21-30.",
            )]),
        ]),
    ]));
    structuring(&mut article);
    let Node::Article(Article {
        references: Some(refs),
        ..
    }) = article
    else {
        bail!("Should have references")
    };
    assert_eq!(refs.len(), 3);
    assert_eq!(refs[0].id, Some("ref-1".into()));
    assert_eq!(refs[1].id, Some("ref-2".into()));
    assert_eq!(refs[2].id, Some("ref-3".into()));

    // "Bibliography" heading should also trigger reference detection
    let mut article = Node::Article(Article::new(vec![
        h1([t("Bibliography")]),
        ol([li([t(
            "Author, A. (2023). Test paper. Test Journal, 1, 1-5.",
        )])]),
    ]));
    structuring(&mut article);
    let Node::Article(Article {
        references: Some(refs),
        ..
    }) = article
    else {
        bail!("Should have references for Bibliography heading")
    };
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].id, Some("ref-1".into()));

    // Case insensitive heading detection
    let mut article = Node::Article(Article::new(vec![
        h1([t("REFERENCES")]),
        ol([li([t(
            "Author, A. (2023). Test paper. Test Journal, 1, 1-5.",
        )])]),
    ]));
    structuring(&mut article);
    let Node::Article(Article {
        references: Some(refs),
        ..
    }) = article
    else {
        bail!("Should have references for uppercase heading")
    };
    assert_eq!(refs.len(), 1);

    // No references section should result in no references
    let mut article = Node::Article(Article::new(vec![
        h1([t("Introduction")]),
        p([t("This is just content.")]),
    ]));
    structuring(&mut article);
    let Node::Article(Article { references, .. }) = article else {
        bail!("Should be an article")
    };
    assert!(references.is_none());

    // Empty reference list should result in no references
    let mut article = Node::Article(Article::new(vec![h1([t("References")]), ol([])]));
    structuring(&mut article);
    let Node::Article(Article { references, .. }) = article else {
        bail!("Should be an article")
    };
    assert!(references.is_none());

    // References section should reset when encountering other high-level headings
    let mut article = Node::Article(Article::new(vec![
        h1([t("References")]),
        ol([li([t(
            "First Author. (2020). First paper. Journal A, 1(1), 1-10.",
        )])]),
        h1([t("Conclusion")]),
        ol([li([t("This should not be treated as a reference")])]),
    ]));
    structuring(&mut article);
    let Node::Article(Article {
        references: Some(refs),
        ..
    }) = article
    else {
        bail!("Should have references")
    };
    assert_eq!(refs.len(), 1);

    // Multiple reference sections should use the last one
    let mut article = Node::Article(Article::new(vec![
        h1([t("References")]),
        ol([li([t(
            "First Author. (2020). First paper. Journal A, 1(1), 1-10.",
        )])]),
        h1([t("Additional References")]),
        h1([t("Bibliography")]),
        ol([
            li([t(
                "Second Author. (2021). Second paper. Journal B, 2(2), 11-20.",
            )]),
            li([t(
                "Third Author. (2022). Third paper. Journal C, 3(3), 21-30.",
            )]),
        ]),
    ]));
    structuring(&mut article);
    let Node::Article(Article {
        references: Some(refs),
        ..
    }) = article
    else {
        bail!("Should have references")
    };
    assert_eq!(refs.len(), 2);
    assert_eq!(refs[0].id, Some("ref-1".into()));
    assert_eq!(refs[1].id, Some("ref-2".into()));

    Ok(())
}

#[test]
fn test_math_inline_to_citation() {
    // Simple superscript citation
    let mut node = p([mi("{ }^{1}", Some("tex"))]);
    structuring(&mut node);
    assert_eq!(node, p([ct("ref-1")]));

    // Range expansion in superscript
    let mut node = p([mi("{ }^{1-3}", Some("tex"))]);
    structuring(&mut node);
    assert_eq!(node, p([ctg(["ref-1", "ref-2", "ref-3"])]));

    // Bracketed citation
    let mut node = p([mi("[5]", Some("tex"))]);
    structuring(&mut node);
    assert_eq!(node, p([ct("ref-5")]));

    // Parentheses citation
    let mut node = p([mi("(7)", Some("tex"))]);
    structuring(&mut node);
    assert_eq!(node, p([ct("ref-7")]));

    // Comma-separated citations in brackets
    let mut node = p([mi("[1,3,5]", Some("tex"))]);
    structuring(&mut node);
    assert_eq!(node, p([ctg(["ref-1", "ref-3", "ref-5"])]));

    // Mixed range and individual citations
    let mut node = p([mi("{ }^{2-4,8}", Some("tex"))]);
    structuring(&mut node);
    assert_eq!(node, p([ctg(["ref-2", "ref-3", "ref-4", "ref-8"])]));

    // Citations with whitespace in parentheses
    let mut node = p([mi("( 10 , 12 )", Some("tex"))]);
    structuring(&mut node);
    assert_eq!(node, p([ctg(["ref-10", "ref-12"])]));

    // Complex range with multiple parts
    let mut node = p([mi("[15-17,20,25-27]", Some("tex"))]);
    structuring(&mut node);
    assert_eq!(
        node,
        p([ctg([
            "ref-15", "ref-16", "ref-17", "ref-20", "ref-25", "ref-26", "ref-27"
        ])])
    );

    // Invalid citation (contains zero) should not be converted
    let mut node = p([mi("{ }^{0,1}", Some("tex"))]);
    structuring(&mut node);
    assert_eq!(node, p([mi("{ }^{0,1}", Some("tex"))]));

    //  Invalid citation (contains letters) should not be converted
    let mut node = p([mi("[1a,2]", Some("tex"))]);
    structuring(&mut node);
    assert_eq!(node, p([mi("[1a,2]", Some("tex"))]));

    // Test en dash in math
    let mut node = p([mi("{ }^{2–4}", Some("tex"))]);
    structuring(&mut node);
    assert_eq!(node, p([ctg(["ref-2", "ref-3", "ref-4"])]));

    // Test em dash in math
    let mut node = p([mi("[15—17]", Some("tex"))]);
    structuring(&mut node);
    assert_eq!(node, p([ctg(["ref-15", "ref-16", "ref-17"])]));
}

#[test]
fn test_text_to_citations() {
    // Simple bracketed citation
    let mut node = p([t("See reference [1] for details.")]);
    structuring(&mut node);
    assert_eq!(
        node,
        p([t("See reference "), ct("ref-1"), t(" for details.")])
    );

    // Simple parenthetical citation
    let mut node = p([t("This is documented (5) in the literature.")]);
    structuring(&mut node);
    assert_eq!(
        node,
        p([
            t("This is documented "),
            ct("ref-5"),
            t(" in the literature.")
        ])
    );

    // Range expansion in brackets
    let mut node = p([t("Studies [1-3] show consistent results.")]);
    structuring(&mut node);
    assert_eq!(
        node,
        p([
            t("Studies "),
            ctg(["ref-1", "ref-2", "ref-3"]),
            t(" show consistent results.")
        ])
    );

    // Comma-separated citations in brackets
    let mut node = p([t("Multiple sources [1,3,5] confirm this.")]);
    structuring(&mut node);
    assert_eq!(
        node,
        p([
            t("Multiple sources "),
            ctg(["ref-1", "ref-3", "ref-5"]),
            t(" confirm this.")
        ])
    );

    // Mixed range and individual citations
    let mut node = p([t("References [2-4,8] are relevant.")]);
    structuring(&mut node);
    assert_eq!(
        node,
        p([
            t("References "),
            ctg(["ref-2", "ref-3", "ref-4", "ref-8"]),
            t(" are relevant.")
        ])
    );

    // Citations with whitespace in parentheses
    let mut node = p([t("See ( 10 , 12 ) for more information.")]);
    structuring(&mut node);
    assert_eq!(
        node,
        p([
            t("See "),
            ctg(["ref-10", "ref-12"]),
            t(" for more information.")
        ])
    );

    // Multiple citations in same text
    let mut node = p([t("First [1] and second [3] citations.")]);
    structuring(&mut node);
    assert_eq!(
        node,
        p([
            t("First "),
            ct("ref-1"),
            t(" and second "),
            ct("ref-3"),
            t(" citations.")
        ])
    );

    // Complex range with multiple parts
    let mut node = p([t("Studies [15-17,20,25-27] are comprehensive.")]);
    structuring(&mut node);
    assert_eq!(
        node,
        p([
            t("Studies "),
            ctg([
                "ref-15", "ref-16", "ref-17", "ref-20", "ref-25", "ref-26", "ref-27"
            ]),
            t(" are comprehensive.")
        ])
    );

    // Invalid citation (contains zero) should not be converted
    let mut node = p([t("Invalid [0,1] citation.")]);
    structuring(&mut node);
    assert_eq!(node, p([t("Invalid [0,1] citation.")]));

    // Invalid citation (contains letters) should not be converted
    let mut node = p([t("Invalid [1a,2] citation.")]);
    structuring(&mut node);
    assert_eq!(node, p([t("Invalid [1a,2] citation.")]));

    // Text without citations should remain unchanged
    let mut node = p([t("Just normal text without any citations.")]);
    structuring(&mut node);
    assert_eq!(node, p([t("Just normal text without any citations.")]));

    // Test en dash ranges
    let mut node = p([t("Studies [1–3] show consistent results.")]);
    structuring(&mut node);
    assert_eq!(
        node,
        p([
            t("Studies "),
            ctg(["ref-1", "ref-2", "ref-3"]),
            t(" show consistent results.")
        ])
    );

    // Test em dash ranges
    let mut node = p([t("References [2—4,8] are relevant.")]);
    structuring(&mut node);
    assert_eq!(
        node,
        p([
            t("References "),
            ctg(["ref-2", "ref-3", "ref-4", "ref-8"]),
            t(" are relevant.")
        ])
    );

    // Test mixed dash types in parentheses
    let mut node = p([t("Multiple (1–3,5—7) citations.")]);
    structuring(&mut node);
    assert_eq!(
        node,
        p([
            t("Multiple "),
            ctg(["ref-1", "ref-2", "ref-3", "ref-5", "ref-6", "ref-7"]),
            t(" citations.")
        ])
    );
}
