use codec_text_trait::to_text;
use common::eyre::{Result, bail};
use common_dev::pretty_assertions::assert_eq;
use schema::{
    AdmonitionType, Article, Block, ImageObject, Inline, Node, Strikeout, Strong, Superscript,
    Underline,
    shortcuts::{ct, ctg, em, h1, li, mi, ol, p, sec, stb, t, tbl},
};

use crate::structuring;

/// Shortcut for creating an [`Block::ImageObject`] since there is
/// no shortcut for that
fn imb(url: &str) -> Block {
    Block::ImageObject(ImageObject::new(url.into()))
}

/// Test detection of headings matching references section
#[test]
fn references_detection() -> Result<()> {
    // Basic "References" heading
    let mut article = Node::Article(Article::new(vec![
        h1([t("References")]),
        p([t("Author, A. B. (2020). Reference.")]),
    ]));
    structuring(&mut article);
    let Node::Article(Article {
        references: Some(..),
        ..
    }) = article
    else {
        bail!("Should have references")
    };

    // Test "Works Cited" (MLA style)
    let mut article = Node::Article(Article::new(vec![
        h1([t("Works Cited")]),
        p([t("Smith, John. \"Article Title.\" Journal Name, 2023.")]),
    ]));
    structuring(&mut article);
    let Node::Article(Article {
        references: Some(..),
        ..
    }) = article
    else {
        bail!("Should detect 'Works Cited'")
    };

    // Test "Literature Cited"
    let mut article = Node::Article(Article::new(vec![
        h1([t("Literature Cited")]),
        p([t("Author. Title. Publisher, 2023.")]),
    ]));
    structuring(&mut article);
    let Node::Article(Article {
        references: Some(..),
        ..
    }) = article
    else {
        bail!("Should detect 'Literature Cited'")
    };

    // Test "Citations"
    let mut article = Node::Article(Article::new(vec![
        h1([t("Citations")]),
        p([t("Reference entry here.")]),
    ]));
    structuring(&mut article);
    let Node::Article(Article {
        references: Some(..),
        ..
    }) = article
    else {
        bail!("Should detect 'Citations'")
    };

    // Test "Sources"
    let mut article = Node::Article(Article::new(vec![
        h1([t("Sources")]),
        p([t("Source entry here.")]),
    ]));
    structuring(&mut article);
    let Node::Article(Article {
        references: Some(..),
        ..
    }) = article
    else {
        bail!("Should detect 'Sources'")
    };

    // Test "Reference List"
    let mut article = Node::Article(Article::new(vec![
        h1([t("Reference List")]),
        p([t("Reference entry here.")]),
    ]));
    structuring(&mut article);
    let Node::Article(Article {
        references: Some(..),
        ..
    }) = article
    else {
        bail!("Should detect 'Reference List'")
    };

    // Test numbered heading "1. References"
    let mut article = Node::Article(Article::new(vec![
        h1([t("1. References")]),
        p([t("Reference entry here.")]),
    ]));
    structuring(&mut article);
    let Node::Article(Article {
        references: Some(..),
        ..
    }) = article
    else {
        bail!("Should detect numbered 'References'")
    };

    // Test lettered heading "A. Bibliography"
    let mut article = Node::Article(Article::new(vec![
        h1([t("A. Bibliography")]),
        p([t("Reference entry here.")]),
    ]));
    structuring(&mut article);
    let Node::Article(Article {
        references: Some(..),
        ..
    }) = article
    else {
        bail!("Should detect lettered 'Bibliography'")
    };

    // Test Roman numeral heading "VI. References"
    let mut article = Node::Article(Article::new(vec![
        h1([t("VI. References")]),
        p([t("Reference entry here.")]),
    ]));
    structuring(&mut article);
    let Node::Article(Article {
        references: Some(..),
        ..
    }) = article
    else {
        bail!("Should detect Roman numeral 'References'")
    };

    // Test "Further Reading"
    let mut article = Node::Article(Article::new(vec![
        h1([t("Further Reading")]),
        p([t("Reading material here.")]),
    ]));
    structuring(&mut article);
    let Node::Article(Article {
        references: Some(..),
        ..
    }) = article
    else {
        bail!("Should detect 'Further Reading'")
    };

    // Test "Additional Sources"
    let mut article = Node::Article(Article::new(vec![
        h1([t("Additional Sources")]),
        p([t("Source entry here.")]),
    ]));
    structuring(&mut article);
    let Node::Article(Article {
        references: Some(..),
        ..
    }) = article
    else {
        bail!("Should detect 'Additional Sources'")
    };

    Ok(())
}

#[test]
fn reference_list_to_references() -> Result<()> {
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
fn math_inline_to_citation() {
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
fn text_to_citations() {
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

#[test]
fn image_then_caption_to_figure() -> Result<()> {
    let mut article = Node::Article(Article::new(vec![
        imb("test.jpg"),
        p([t("Figure 1. This is a test caption.")]),
    ]));
    structuring(&mut article);
    let Node::Article(Article { content, .. }) = article else {
        bail!("Should be an article")
    };

    assert_eq!(content.len(), 1, "Should have 1 block after structuring");

    let Block::Figure(figure) = &content[0] else {
        bail!("Should have figure block")
    };

    assert_eq!(
        figure.label,
        Some("1".into()),
        "Figure should have label '1'"
    );
    assert_eq!(figure.content.len(), 1, "Figure should have 1 content item");
    assert!(
        matches!(figure.content[0], Block::ImageObject(_)),
        "Figure content should be ImageObject"
    );

    let caption = figure.caption.as_ref().expect("Figure should have caption");
    assert_eq!(caption.len(), 1, "Caption should have 1 block");

    let Block::Paragraph(caption_para) = &caption[0] else {
        bail!("Caption should be paragraph")
    };

    let caption_text = to_text(caption_para);
    assert_eq!(
        caption_text.trim(),
        "This is a test caption.",
        "Caption text should be cleaned"
    );

    Ok(())
}

#[test]
fn caption_then_image_to_figure() -> Result<()> {
    let mut article = Node::Article(Article::new(vec![
        p([t("Fig 2: Another test caption.")]),
        imb("test.jpg"),
    ]));

    structuring(&mut article);

    let Node::Article(Article { content, .. }) = article else {
        bail!("Should be an article")
    };

    assert_eq!(content.len(), 1, "Should have 1 block after structuring");

    let Block::Figure(figure) = &content[0] else {
        bail!("Should have figure block")
    };

    assert_eq!(
        figure.label,
        Some("2".into()),
        "Figure should have label '2'"
    );
    assert_eq!(figure.content.len(), 1, "Figure should have 1 content item");
    assert!(
        matches!(figure.content[0], Block::ImageObject(_)),
        "Figure content should be ImageObject"
    );

    let caption = figure.caption.as_ref().expect("Figure should have caption");
    assert_eq!(caption.len(), 1, "Caption should have 1 block");

    let caption_text = to_text(&caption[0]);
    assert_eq!(
        caption_text.trim(),
        "Another test caption.",
        "Caption text should be cleaned"
    );

    Ok(())
}

#[test]
fn image_and_caption_multiple_figures() -> Result<()> {
    let mut article = Node::Article(Article::new(vec![
        imb("test1.jpg"),
        p([t("Figure 1. First caption.")]),
        p([t("Some intervening text.")]),
        p([t("Figure 2: Second caption.")]),
        imb("test2.jpg"),
    ]));
    structuring(&mut article);
    let Node::Article(Article { content, .. }) = article else {
        bail!("Should be an article")
    };

    assert_eq!(content.len(), 3, "Should have 3 blocks after structuring");

    // First figure
    let Block::Figure(figure1) = &content[0] else {
        bail!("First block should be figure")
    };
    assert_eq!(
        figure1.label,
        Some("1".into()),
        "First figure should have label '1'"
    );

    // Intervening text
    let Block::Paragraph(_) = &content[1] else {
        bail!("Second block should be paragraph")
    };

    // Second figure
    let Block::Figure(figure2) = &content[2] else {
        bail!("Third block should be figure")
    };
    assert_eq!(
        figure2.label,
        Some("2".into()),
        "Second figure should have label '2'"
    );

    Ok(())
}

#[test]
fn images_and_not_captions() -> Result<()> {
    let mut article = Node::Article(Article::new(vec![
        imb("test.jpg"),
        p([t("This is not a figure caption.")]),
        p([t("Figure 1. This caption has no image following.")]),
        p([t("More text.")]),
        imb("test.jpg"),
    ]));
    structuring(&mut article);
    let Node::Article(Article { content, .. }) = article else {
        bail!("Should be an article")
    };

    assert_eq!(content.len(), 5,);

    // All blocks should remain as-is
    assert!(matches!(content[0], Block::ImageObject(_)));
    assert!(matches!(content[1], Block::Paragraph(_)));
    assert!(matches!(content[2], Block::Paragraph(_)));
    assert!(matches!(content[3], Block::Paragraph(_)));
    assert!(matches!(content[4], Block::ImageObject(_)));

    Ok(())
}

#[test]
fn image_and_caption_edge_cases() -> Result<()> {
    // Test case insensitive matching
    let mut article1 = Node::Article(Article::new(vec![
        imb("test1.jpg"),
        p([t("FIGURE 1. Uppercase caption.")]),
    ]));

    structuring(&mut article1);

    let Node::Article(Article { content, .. }) = article1 else {
        bail!("Should be an article")
    };

    assert_eq!(content.len(), 1, "Should have 1 block after structuring");
    assert!(
        matches!(content[0], Block::Figure(_)),
        "Should create figure from uppercase"
    );

    // Test various figure prefix formats
    let test_cases = [
        "Figure 5. Standard format",
        "Fig 10: Colon separator",
        "Fig. 20 Dot format",
        "figure 99 - Dash separator",
    ];

    for (i, case) in test_cases.iter().enumerate() {
        let mut article = Node::Article(Article::new(vec![
            imb(&format!("test{}.jpg", i + 2)),
            p([t(case)]),
        ]));

        structuring(&mut article);

        let Node::Article(Article { content, .. }) = article else {
            bail!("Should be an article")
        };

        assert_eq!(
            content.len(),
            1,
            "Case {i} should have 1 block after structuring"
        );
        let Block::Figure(figure) = &content[0] else {
            bail!("Case {i} should create figure")
        };

        assert!(figure.label.is_some(), "Case {i} should have label");
    }

    // Test that nested content IS processed (figures work in sections too)
    use schema::shortcuts::sec;

    let mut article_nested = Node::Article(Article::new(vec![sec([
        h1([t("Section")]),
        imb("nested.jpg"),
        p([t(
            "Figure 1. This should be structured even when nested in a section.",
        )]),
    ])]));

    structuring(&mut article_nested);

    let Node::Article(Article { content, .. }) = article_nested else {
        bail!("Should be an article")
    };

    assert_eq!(content.len(), 1, "Should still have 1 top-level block");
    let Block::Section(section) = &content[0] else {
        bail!("Should be section")
    };

    // The content inside the section should be structured into a figure
    assert_eq!(
        section.content.len(),
        2,
        "Section should have 2 blocks after structuring"
    );
    assert!(
        matches!(section.content[0], Block::Heading(_)),
        "First should be heading"
    );
    assert!(
        matches!(section.content[1], Block::Figure(_)),
        "Second should be figure"
    );

    // Verify the figure was created correctly
    if let Block::Figure(figure) = &section.content[1] {
        assert_eq!(
            figure.label,
            Some("1".into()),
            "Figure should have label '1'"
        );
        assert!(figure.caption.is_some(), "Figure should have caption");
    }

    Ok(())
}

#[test]
fn nested_figures_in_various_blocks() -> Result<()> {
    use schema::shortcuts::{adm, sec};

    // Test figure in admonition
    let mut article1 = Node::Article(Article::new(vec![adm(
        AdmonitionType::Note,
        Some("Note Title"),
        [
            p([t("This is an admonition.")]),
            imb("admonition.jpg"),
            p([t("Figure 1. Caption inside admonition.")]),
        ],
    )]));

    structuring(&mut article1);

    let Node::Article(Article { content, .. }) = article1 else {
        bail!("Should be an article")
    };

    assert_eq!(content.len(), 1);
    let Block::Admonition(admonition) = &content[0] else {
        bail!("Should be admonition")
    };

    assert_eq!(
        admonition.content.len(),
        2,
        "Admonition should have 2 blocks after structuring"
    );
    assert!(
        matches!(admonition.content[0], Block::Paragraph(_)),
        "First should be paragraph"
    );
    assert!(
        matches!(admonition.content[1], Block::Figure(_)),
        "Second should be figure"
    );

    if let Block::Figure(figure) = &admonition.content[1] {
        assert_eq!(figure.label, Some("1".into()));
    }

    // Test figure in styled block
    let mut article2 = Node::Article(Article::new(vec![stb(
        "info",
        [imb("styled.jpg"), p([t("Fig 2: Caption in styled block.")])],
    )]));

    structuring(&mut article2);

    let Node::Article(Article { content, .. }) = article2 else {
        bail!("Should be an article")
    };

    assert_eq!(content.len(), 1);
    let Block::StyledBlock(styled) = &content[0] else {
        bail!("Should be styled block")
    };

    assert_eq!(
        styled.content.len(),
        1,
        "Styled block should have 1 block after structuring"
    );
    assert!(
        matches!(styled.content[0], Block::Figure(_)),
        "Should be figure"
    );

    if let Block::Figure(figure) = &styled.content[0] {
        assert_eq!(figure.label, Some("2".into()));
    }

    // Test nested sections
    let mut article3 = Node::Article(Article::new(vec![sec([
        h1([t("Main Section")]),
        sec([
            h1([t("Subsection")]),
            p([t("Figure 3. Nested section caption.")]),
            imb("subsection.jpg"),
        ]),
    ])]));

    structuring(&mut article3);

    let Node::Article(Article { content, .. }) = article3 else {
        bail!("Should be an article")
    };

    assert_eq!(content.len(), 1);
    let Block::Section(main_section) = &content[0] else {
        bail!("Should be section")
    };

    assert_eq!(main_section.content.len(), 2);
    let Block::Section(subsection) = &main_section.content[1] else {
        bail!("Should be subsection")
    };

    assert_eq!(
        subsection.content.len(),
        2,
        "Subsection should have 2 blocks after structuring"
    );
    assert!(
        matches!(subsection.content[0], Block::Heading(_)),
        "First should be heading"
    );
    assert!(
        matches!(subsection.content[1], Block::Figure(_)),
        "Second should be figure"
    );

    if let Block::Figure(figure) = &subsection.content[1] {
        assert_eq!(figure.label, Some("3".into()));
    }

    Ok(())
}

#[test]
fn mixed_nested_and_top_level_figures() -> Result<()> {
    let mut article = Node::Article(Article::new(vec![
        // Top-level Image + Caption
        imb("mixed1jpg"),
        p([t("Figure 1. Top-level caption.")]),
        // Nested Caption + Image in section
        sec([
            h1([t("Section")]),
            p([t("Figure 2. Nested caption.")]),
            imb("mixed2.jpg"),
        ]),
    ]));

    structuring(&mut article);

    let Node::Article(Article { content, .. }) = article else {
        bail!("Should be an article")
    };

    // Should have: Figure(1), Section containing Figure(2)
    assert_eq!(content.len(), 2);

    // Check first top-level figure
    let Block::Figure(figure1) = &content[0] else {
        bail!("First should be figure")
    };
    assert_eq!(figure1.label, Some("1".into()));

    // Check section containing nested figure
    let Block::Section(section) = &content[1] else {
        bail!("Second should be section")
    };
    assert_eq!(section.content.len(), 2); // heading + figure

    let Block::Figure(figure2) = &section.content[1] else {
        bail!("Section should contain figure")
    };
    assert_eq!(figure2.label, Some("2".into()));

    Ok(())
}

#[test]
fn figure_caption_with_nested_inline_elements() -> Result<()> {
    // Test figure caption starting with emphasis
    let mut article = Node::Article(Article::new(vec![
        imb("emphasis.jpg"),
        p([em([t("Figure 1.")]), t(" Caption with emphasis at start.")]),
    ]));
    structuring(&mut article);
    let Node::Article(Article { content, .. }) = article else {
        bail!("Should be an article")
    };

    assert_eq!(content.len(), 1, "Should have 1 block after structuring");
    let Block::Figure(figure) = &content[0] else {
        bail!("Should have figure block")
    };

    assert_eq!(
        figure.label,
        Some("1".into()),
        "Figure should have label '1'"
    );
    let caption = figure.caption.as_ref().expect("Figure should have caption");
    let caption_text = to_text(&caption[0]);
    assert_eq!(
        caption_text.trim(),
        "Caption with emphasis at start.",
        "Caption should be properly cleaned"
    );

    // Test figure caption starting with strong
    let mut article2 = Node::Article(Article::new(vec![
        imb("strong.jpg"),
        p([
            Inline::Strong(Strong::new(vec![t("Fig 2:")])),
            t(" Caption with strong at start."),
        ]),
    ]));
    structuring(&mut article2);
    let Node::Article(Article { content, .. }) = article2 else {
        bail!("Should be an article")
    };

    assert_eq!(content.len(), 1, "Should have 1 block after structuring");
    let Block::Figure(figure2) = &content[0] else {
        bail!("Should have figure block")
    };

    assert_eq!(
        figure2.label,
        Some("2".into()),
        "Figure should have label '2'"
    );
    let caption2 = figure2
        .caption
        .as_ref()
        .expect("Figure should have caption");
    let caption_text2 = to_text(&caption2[0]);
    assert_eq!(
        caption_text2.trim(),
        "Caption with strong at start.",
        "Caption should be properly cleaned"
    );

    // Test figure caption where prefix spans multiple inline elements
    let mut article3 = Node::Article(Article::new(vec![
        imb("multi.jpg"),
        p([
            em([t("Figure")]),
            t(" 3. "),
            Inline::Strong(Strong::new(vec![t("Caption")])),
            t(" with multiple elements."),
        ]),
    ]));
    structuring(&mut article3);
    let Node::Article(Article { content, .. }) = article3 else {
        bail!("Should be an article")
    };

    assert_eq!(content.len(), 1, "Should have 1 block after structuring");
    let Block::Figure(figure3) = &content[0] else {
        bail!("Should have figure block")
    };

    assert_eq!(
        figure3.label,
        Some("3".into()),
        "Figure should have label '3'"
    );
    let caption3 = figure3
        .caption
        .as_ref()
        .expect("Figure should have caption");
    let caption_text3 = to_text(&caption3[0]);
    assert_eq!(
        caption_text3.trim(),
        "Caption with multiple elements.",
        "Caption should be properly cleaned after removing multi-element prefix"
    );

    // Test figure caption where prefix is entirely within emphasis
    let mut article4 = Node::Article(Article::new(vec![
        imb("within.jpg"),
        p([
            em([t("Figure 4. Some text within emphasis")]),
            t(" and more text outside."),
        ]),
    ]));
    structuring(&mut article4);
    let Node::Article(Article { content, .. }) = article4 else {
        bail!("Should be an article")
    };

    assert_eq!(content.len(), 1, "Should have 1 block after structuring");
    let Block::Figure(figure4) = &content[0] else {
        bail!("Should have figure block")
    };

    assert_eq!(
        figure4.label,
        Some("4".into()),
        "Figure should have label '4'"
    );
    let caption4 = figure4
        .caption
        .as_ref()
        .expect("Figure should have caption");
    let caption_text4 = to_text(&caption4[0]);
    assert_eq!(
        caption_text4.trim(),
        "Some text within emphasis and more text outside.",
        "Caption should preserve remaining emphasis content and following text"
    );

    // Test nested emphasis and strong
    let mut article5 = Node::Article(Article::new(vec![
        imb("nested.jpg"),
        p([
            em([Inline::Strong(Strong::new(vec![t("Fig")])), t(" 5.")]),
            t(" Nested emphasis and strong at start."),
        ]),
    ]));
    structuring(&mut article5);
    let Node::Article(Article { content, .. }) = article5 else {
        bail!("Should be an article")
    };

    assert_eq!(content.len(), 1, "Should have 1 block after structuring");
    let Block::Figure(figure5) = &content[0] else {
        bail!("Should have figure block")
    };

    assert_eq!(
        figure5.label,
        Some("5".into()),
        "Figure should have label '5'"
    );
    let caption5 = figure5
        .caption
        .as_ref()
        .expect("Figure should have caption");
    let caption_text5 = to_text(&caption5[0]);
    assert_eq!(
        caption_text5.trim(),
        "Nested emphasis and strong at start.",
        "Caption should handle nested inline elements"
    );

    // Test edge case: entire first inline element gets removed
    let mut article6 = Node::Article(Article::new(vec![
        imb("entire.jpg"),
        p([
            em([t("Figure 6.")]),
            t(" Rest of caption after complete removal."),
        ]),
    ]));
    structuring(&mut article6);
    let Node::Article(Article { content, .. }) = article6 else {
        bail!("Should be an article")
    };

    assert_eq!(content.len(), 1, "Should have 1 block after structuring");
    let Block::Figure(figure6) = &content[0] else {
        bail!("Should have figure block")
    };

    assert_eq!(
        figure6.label,
        Some("6".into()),
        "Figure should have label '6'"
    );
    let caption6 = figure6
        .caption
        .as_ref()
        .expect("Figure should have caption");
    let caption_text6 = to_text(&caption6[0]);
    assert_eq!(
        caption_text6.trim(),
        "Rest of caption after complete removal.",
        "Caption should handle complete removal of first inline element"
    );

    // Test figure caption starting with underline
    let mut article7 = Node::Article(Article::new(vec![
        imb("underline.jpg"),
        p([
            Inline::Underline(Underline::new(vec![t("Figure 7.")])),
            t(" Caption with underline at start."),
        ]),
    ]));
    structuring(&mut article7);
    let Node::Article(Article { content, .. }) = article7 else {
        bail!("Should be an article")
    };

    assert_eq!(content.len(), 1, "Should have 1 block after structuring");
    let Block::Figure(figure7) = &content[0] else {
        bail!("Should have figure block")
    };

    assert_eq!(
        figure7.label,
        Some("7".into()),
        "Figure should have label '7'"
    );
    let caption7 = figure7
        .caption
        .as_ref()
        .expect("Figure should have caption");
    let caption_text7 = to_text(&caption7[0]);
    assert_eq!(
        caption_text7.trim(),
        "Caption with underline at start.",
        "Caption should handle underline prefix removal"
    );

    // Test figure caption starting with strikeout
    let mut article8 = Node::Article(Article::new(vec![
        imb("strikeout.jpg"),
        p([
            Inline::Strikeout(Strikeout::new(vec![t("Fig 8:")])),
            t(" Caption with strikeout at start."),
        ]),
    ]));
    structuring(&mut article8);
    let Node::Article(Article { content, .. }) = article8 else {
        bail!("Should be an article")
    };

    assert_eq!(content.len(), 1, "Should have 1 block after structuring");
    let Block::Figure(figure8) = &content[0] else {
        bail!("Should have figure block")
    };

    assert_eq!(
        figure8.label,
        Some("8".into()),
        "Figure should have label '8'"
    );
    let caption8 = figure8
        .caption
        .as_ref()
        .expect("Figure should have caption");
    let caption_text8 = to_text(&caption8[0]);
    assert_eq!(
        caption_text8.trim(),
        "Caption with strikeout at start.",
        "Caption should handle strikeout prefix removal"
    );

    // Test complex nested elements: superscript containing figure prefix
    let mut article9 = Node::Article(Article::new(vec![
        imb("complex.jpg"),
        p([
            Inline::Superscript(Superscript::new(vec![t("Figure 9.")])),
            t(" Complex nested caption."),
        ]),
    ]));
    structuring(&mut article9);
    let Node::Article(Article { content, .. }) = article9 else {
        bail!("Should be an article")
    };

    assert_eq!(content.len(), 1, "Should have 1 block after structuring");
    let Block::Figure(figure9) = &content[0] else {
        bail!("Should have figure block")
    };

    assert_eq!(
        figure9.label,
        Some("9".into()),
        "Figure should have label '9'"
    );
    let caption9 = figure9
        .caption
        .as_ref()
        .expect("Figure should have caption");
    let caption_text9 = to_text(&caption9[0]);
    assert_eq!(
        caption_text9.trim(),
        "Complex nested caption.",
        "Caption should handle superscript prefix removal"
    );

    Ok(())
}

#[test]
fn caption_then_table_to_table_with_caption() -> Result<()> {
    let mut article = Node::Article(Article::new(vec![
        p([t("Table 1. This is a test table caption.")]),
        tbl([]),
    ]));
    structuring(&mut article);
    let Node::Article(Article { content, .. }) = article else {
        bail!("Should be an article")
    };

    assert_eq!(content.len(), 1, "Should have 1 block after structuring");

    let Block::Table(table) = &content[0] else {
        bail!("Should have table block")
    };

    assert_eq!(table.label, Some("1".into()), "Table should have label '1'");

    let caption = table.caption.as_ref().expect("Table should have caption");
    assert_eq!(caption.len(), 1, "Caption should have 1 block");

    let Block::Paragraph(caption_para) = &caption[0] else {
        bail!("Caption should be paragraph")
    };

    let caption_text = to_text(caption_para);
    assert_eq!(
        caption_text.trim(),
        "This is a test table caption.",
        "Caption text should be cleaned"
    );

    Ok(())
}

#[test]
fn table_caption_with_different_formats() -> Result<()> {
    // Test various table caption formats
    let test_cases = [
        ("Table 1. Standard format", "1", "Standard format"),
        ("Table 2: Colon separator", "2", "Colon separator"),
        ("Table 10 - Dash separator", "10", "Dash separator"),
        ("table 99. Lowercase", "99", "Lowercase"),
        ("TABLE 5: Uppercase", "5", "Uppercase"),
    ];

    for (i, (caption_text, expected_number, expected_caption)) in test_cases.iter().enumerate() {
        let mut article = Node::Article(Article::new(vec![p([t(caption_text)]), tbl([])]));

        structuring(&mut article);

        let Node::Article(Article { content, .. }) = article else {
            bail!("Should be an article for case {i}")
        };

        assert_eq!(
            content.len(),
            1,
            "Case {i} should have 1 block after structuring"
        );

        let Block::Table(table) = &content[0] else {
            bail!("Case {i} should have table block")
        };

        assert_eq!(
            table.label,
            Some(expected_number.to_string()),
            "Case {i} should have correct label"
        );

        let caption = table.caption.as_ref().expect("Table should have caption");
        let caption_text = to_text(&caption[0]);
        assert_eq!(
            caption_text.trim(),
            *expected_caption,
            "Case {i} should have correct cleaned caption"
        );
    }

    Ok(())
}

#[test]
fn table_caption_with_nested_inline_elements() -> Result<()> {
    // Test table caption starting with emphasis
    let mut article1 = Node::Article(Article::new(vec![
        p([em([t("Table 1.")]), t(" Caption with emphasis at start.")]),
        tbl([]),
    ]));
    structuring(&mut article1);
    let Node::Article(Article { content, .. }) = article1 else {
        bail!("Should be an article")
    };

    assert_eq!(content.len(), 1, "Should have 1 block after structuring");
    let Block::Table(table1) = &content[0] else {
        bail!("Should have table block")
    };

    assert_eq!(
        table1.label,
        Some("1".into()),
        "Table should have label '1'"
    );
    let caption1 = table1.caption.as_ref().expect("Table should have caption");
    let caption_text1 = to_text(&caption1[0]);
    assert_eq!(
        caption_text1.trim(),
        "Caption with emphasis at start.",
        "Caption should be properly cleaned"
    );

    // Test table caption starting with strong
    let mut article2 = Node::Article(Article::new(vec![
        p([
            Inline::Strong(Strong::new(vec![t("Table 2:")])),
            t(" Caption with strong at start."),
        ]),
        tbl([]),
    ]));
    structuring(&mut article2);
    let Node::Article(Article { content, .. }) = article2 else {
        bail!("Should be an article")
    };

    assert_eq!(content.len(), 1, "Should have 1 block after structuring");
    let Block::Table(table2) = &content[0] else {
        bail!("Should have table block")
    };

    assert_eq!(
        table2.label,
        Some("2".into()),
        "Table should have label '2'"
    );
    let caption2 = table2.caption.as_ref().expect("Table should have caption");
    let caption_text2 = to_text(&caption2[0]);
    assert_eq!(
        caption_text2.trim(),
        "Caption with strong at start.",
        "Caption should handle strong formatting"
    );

    Ok(())
}

#[test]
fn table_without_caption_unchanged() -> Result<()> {
    let mut article = Node::Article(Article::new(vec![
        p([t("This is not a table caption.")]),
        tbl([]),
        p([t("Table 1. This caption has no table following.")]),
        p([t("More text.")]),
    ]));
    structuring(&mut article);
    let Node::Article(Article { content, .. }) = article else {
        bail!("Should be an article")
    };

    assert_eq!(content.len(), 4, "Should have 4 blocks unchanged");

    // All blocks should remain as-is since no valid table captions are detected
    assert!(matches!(content[0], Block::Paragraph(_)));
    assert!(matches!(content[1], Block::Table(_)));
    assert!(matches!(content[2], Block::Paragraph(_)));
    assert!(matches!(content[3], Block::Paragraph(_)));

    // The table should not have been modified
    let Block::Table(table) = &content[1] else {
        bail!("Should have table block")
    };

    assert!(table.label.is_none(), "Table should not have label");
    assert!(table.caption.is_none(), "Table should not have caption");

    Ok(())
}

#[test]
fn table_caption_in_nested_sections() -> Result<()> {
    // Test table caption detection works in nested content
    let mut article = Node::Article(Article::new(vec![sec([
        h1([t("Section")]),
        p([t("Table 1. This table is in a section.")]),
        tbl([]),
    ])]));

    structuring(&mut article);

    let Node::Article(Article { content, .. }) = article else {
        bail!("Should be an article")
    };

    assert_eq!(content.len(), 1, "Should still have 1 top-level block");
    let Block::Section(section) = &content[0] else {
        bail!("Should be section")
    };

    // The content inside the section should be structured into a table with caption
    assert_eq!(
        section.content.len(),
        2,
        "Section should have 2 blocks after structuring"
    );
    assert!(
        matches!(section.content[0], Block::Heading(_)),
        "First should be heading"
    );
    assert!(
        matches!(section.content[1], Block::Table(_)),
        "Second should be table"
    );

    // Verify the table was structured correctly
    if let Block::Table(table) = &section.content[1] {
        assert_eq!(table.label, Some("1".into()), "Table should have label '1'");
        assert!(table.caption.is_some(), "Table should have caption");

        let caption = table.caption.as_ref().expect("Table should have caption");
        let caption_text = to_text(&caption[0]);
        assert_eq!(
            caption_text.trim(),
            "This table is in a section.",
            "Caption should be properly cleaned"
        );
    }

    Ok(())
}

#[test]
fn multiple_table_captions() -> Result<()> {
    let mut article = Node::Article(Article::new(vec![
        p([t("Table 1. First table caption.")]),
        tbl([]),
        p([t("Some intervening text.")]),
        p([t("Table 2: Second table caption.")]),
        tbl([]),
    ]));
    structuring(&mut article);
    let Node::Article(Article { content, .. }) = article else {
        bail!("Should be an article")
    };

    assert_eq!(content.len(), 3, "Should have 3 blocks after structuring");

    // First table
    let Block::Table(table1) = &content[0] else {
        bail!("First block should be table")
    };
    assert_eq!(
        table1.label,
        Some("1".into()),
        "First table should have label '1'"
    );

    // Intervening text
    let Block::Paragraph(_) = &content[1] else {
        bail!("Second block should be paragraph")
    };

    // Second table
    let Block::Table(table2) = &content[2] else {
        bail!("Third block should be table")
    };
    assert_eq!(
        table2.label,
        Some("2".into()),
        "Second table should have label '2'"
    );

    Ok(())
}
