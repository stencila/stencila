use eyre::Result;
use serde_json::{Value, json};
use stencila_node_compare::{CompareOptions, Comparison, compare, compare_with_options};
use stencila_schema::Node;

fn assert_cosmetic(left: Value, right: Value, normalize: &[&str]) -> Result<()> {
    assert_classification(left, right, normalize, true)
}

fn assert_classification(
    left: Value,
    right: Value,
    normalize: &[&str],
    cosmetic: bool,
) -> Result<()> {
    let left: Node = serde_json::from_value(left)?;
    let right: Node = serde_json::from_value(right)?;
    let options: CompareOptions = serde_json::from_value(json!({ "normalize": normalize }))?;
    let faithful = compare(&left, &right)?;
    let classified = compare_with_options(&left, &right, &options)?;
    assert_eq!(faithful.alignment(), classified.alignment());
    assert!(!faithful.is_equal());
    assert!(!classified.is_equal());
    assert!(!classified.is_equal_unfiltered());
    assert_eq!(classified.differences().len(), 1);
    assert_eq!(classified.differences()[0].is_cosmetic(), cosmetic);
    classified.validate(&left, &right)?;
    assert_eq!(
        classified.clone().invert(),
        compare_with_options(&right, &left, &options)?,
    );
    assert_eq!(
        serde_json::from_value::<Comparison>(serde_json::to_value(&classified)?)?,
        classified,
    );
    let faithful = serde_json::to_value(faithful)?;
    let classified = serde_json::to_value(classified)?;
    assert_eq!(classified["differences"].as_array().map(Vec::len), Some(1));
    assert_eq!(
        classified["differences"][0]["left"],
        faithful["differences"][0]["left"]
    );
    assert_eq!(
        classified["differences"][0]["right"],
        faithful["differences"][0]["right"]
    );
    assert!(faithful.get("normalize").is_none());
    assert!(faithful["differences"][0].get("cosmetic").is_none());
    assert_eq!(
        classified["differences"][0]["cosmetic"],
        if cosmetic { json!(true) } else { Value::Null }
    );
    assert_eq!(classified["normalize"], json!(normalize));
    Ok(())
}

#[test]
fn whitespace_collapses_runs_but_preserves_word_boundaries() -> Result<()> {
    assert_cosmetic(
        json!({ "type": "Text", "value": "Auxin\tsignalling\npathways\u{a0}" }),
        json!({ "type": "Text", "value": "Auxin signalling  pathways " }),
        &["whitespace"],
    )?;
    for (left, right) in [
        ("a b", "ab"),
        ("Hello ", "Hello"),
        (" Hello", "Hello"),
        ("Auxin signalling", "Auxin transport"),
        ("Auxin", "auxin"),
    ] {
        assert_classification(
            json!({ "type": "Text", "value": left }),
            json!({ "type": "Text", "value": right }),
            &["whitespace", "typography", "initials"],
            false,
        )?;
    }
    Ok(())
}

#[test]
fn whitespace_keeps_word_boundaries_between_inline_nodes() -> Result<()> {
    let paragraph = |text: &str| {
        json!({
            "type": "Paragraph",
            "content": [
                { "type": "Text", "value": text },
                { "type": "Emphasis", "content": [{ "type": "Text", "value": "world" }] },
            ],
        })
    };
    assert_classification(
        paragraph("Hello "),
        paragraph("Hello"),
        &["whitespace"],
        false,
    )?;
    assert_cosmetic(paragraph("Hello "), paragraph("Hello\n\t"), &["whitespace"])
}

#[test]
fn rules_are_independent_and_can_be_combined() -> Result<()> {
    let left = json!({ "type": "Text", "value": " “Auxin”—signalling… " });
    let right = json!({ "type": "Text", "value": "\t\"Auxin\"-signalling...\n" });
    assert_classification(left.clone(), right.clone(), &["whitespace"], false)?;
    assert_classification(left.clone(), right.clone(), &["typography"], false)?;
    assert_classification(left.clone(), right.clone(), &["initials"], false)?;
    assert_cosmetic(left, right, &["whitespace", "typography"])
}

#[test]
fn initials_preserve_full_names_case_order_and_accents() -> Result<()> {
    for (left, right, cosmetic) in [
        (json!(["S. S."]), json!(["S", "S"]), true),
        (json!(["S S"]), json!(["S", "S"]), true),
        (json!(["Sam", "S.S"]), json!(["Sam", "S", "S"]), true),
        (json!(["É. S."]), json!(["É", "S"]), true),
        (json!(["E\u{301}. S."]), json!(["E\u{301}", "S"]), true),
        (
            json!(["E\u{301}\u{323}.S."]),
            json!(["E\u{301}\u{323}", "S"]),
            true,
        ),
        (json!(["S.S"]), json!(["Sam", "Smith"]), false),
        (json!(["SS"]), json!(["S", "S"]), false),
        (json!(["S.S"]), json!(["S", "T"]), false),
        (json!(["S.T"]), json!(["T", "S"]), false),
        (json!(["S.S"]), json!(["s", "s"]), false),
        (json!(["É.S"]), json!(["E", "S"]), false),
        (json!(["E\u{301}.S"]), json!(["E", "S"]), false),
        (json!(["É.S"]), json!(["E\u{301}", "S"]), false),
        (json!(["\u{301}.S"]), json!(["\u{301}", "S"]), false),
        (json!(["Mary Ann"]), json!(["Mary", "Ann"]), false),
        (json!(["S."]), json!([]), false),
        (json!(["."]), json!([]), false),
    ] {
        assert_classification(
            json!({ "type": "Person", "givenNames": left }),
            json!({ "type": "Person", "givenNames": right }),
            &["initials"],
            cosmetic,
        )?;
    }
    Ok(())
}

#[test]
fn whitespace_trims_given_names() -> Result<()> {
    assert_cosmetic(
        json!({ "type": "Person", "givenNames": [" Sam ", "S.\u{a0}S."] }),
        json!({ "type": "Person", "givenNames": ["Sam", "S. S."] }),
        &["whitespace"],
    )
}

#[test]
fn presence_code_math_targets_and_other_properties_stay_literal() -> Result<()> {
    for (left, right) in [
        (
            json!({ "type": "Person" }),
            json!({ "type": "Person", "givenNames": [] }),
        ),
        (
            json!({ "type": "Person", "familyNames": ["S.S"] }),
            json!({ "type": "Person", "familyNames": ["S", "S"] }),
        ),
        (
            json!({ "type": "CodeBlock", "code": "  x = 1" }),
            json!({ "type": "CodeBlock", "code": "x = 1" }),
        ),
        (
            json!({ "type": "MathBlock", "code": "x – 1" }),
            json!({ "type": "MathBlock", "code": "x - 1" }),
        ),
        (
            json!({ "type": "Link", "content": [], "target": "a–b" }),
            json!({ "type": "Link", "content": [], "target": "a-b" }),
        ),
        (
            json!({ "type": "Text", "value": "x − 1" }),
            json!({ "type": "Text", "value": "x - 1" }),
        ),
        (json!(" a "), json!("a")),
    ] {
        assert_classification(
            left,
            right,
            &["whitespace", "typography", "initials"],
            false,
        )?;
    }
    Ok(())
}

#[test]
fn classifies_paired_text_and_authors_inside_documents() -> Result<()> {
    let article = |text: &str, names: Value| -> Result<Node> {
        Ok(serde_json::from_value(json!({
            "type": "Article",
            "authors": [{ "type": "Person", "familyNames": ["Smith"], "givenNames": names }],
            "content": [{ "type": "Paragraph", "content": [{ "type": "Text", "value": text }] }],
        }))?)
    };
    let left = article("Auxin’s role", json!(["S.S"]))?;
    let right = article("Auxin's  role", json!(["S", "S"]))?;
    let options =
        serde_json::from_value(json!({ "normalize": ["whitespace", "typography", "initials"] }))?;
    let comparison = compare_with_options(&left, &right, &options)?;
    assert_eq!(comparison.alignment(), compare(&left, &right)?.alignment());
    assert_eq!(comparison.differences().len(), 2);
    assert!(
        comparison
            .differences()
            .iter()
            .all(|difference| difference.is_cosmetic())
    );

    let filtered_options = serde_json::from_value(json!({
        "normalize": ["whitespace", "typography", "initials"],
        "filter": { "exclude": ["Text"] },
    }))?;
    let filtered = compare_with_options(&left, &right, &filtered_options)?;
    assert_eq!(filtered.differences().len(), 1);
    assert_eq!(filtered.suppressed_differences(), 1);
    assert_eq!(filtered.normalizations(), comparison.normalizations());
    assert!(filtered.differences()[0].is_cosmetic());
    Ok(())
}

#[test]
fn loading_keeps_the_recorded_classification() -> Result<()> {
    let left = serde_json::from_value(json!({ "type": "Text", "value": "Auxin  signalling" }))?;
    let right = serde_json::from_value(json!({ "type": "Text", "value": "Auxin signalling" }))?;
    let options = serde_json::from_value(json!({ "normalize": ["whitespace"] }))?;
    let comparison = compare_with_options(&left, &right, &options)?;
    assert!(comparison.differences()[0].is_cosmetic());

    // A classification recorded under different rules than today's still loads as recorded
    let mut artifact = serde_json::to_value(&comparison)?;
    artifact["differences"][0]["cosmetic"] = json!(false);
    let loaded = serde_json::from_value::<Comparison>(artifact)?;
    assert!(!loaded.differences()[0].is_cosmetic());
    loaded.validate(&left, &right)?;

    // But nothing can be cosmetic without a rule to explain it
    let mut artifact = serde_json::to_value(&comparison)?;
    artifact["normalize"] = json!([]);
    assert!(serde_json::from_value::<Comparison>(artifact).is_err());
    Ok(())
}

#[test]
fn loading_rejects_one_change_recorded_twice_with_different_classifications() -> Result<()> {
    let left = serde_json::from_value(json!({ "type": "Text", "value": "Auxin  signalling" }))?;
    let right = serde_json::from_value(json!({ "type": "Text", "value": "Auxin signalling" }))?;
    let options = serde_json::from_value(json!({ "normalize": ["whitespace"] }))?;
    let comparison = compare_with_options(&left, &right, &options)?;

    for cosmetic in [json!(true), json!(false)] {
        let mut artifact = serde_json::to_value(&comparison)?;
        let mut duplicate = artifact["differences"][0].clone();
        duplicate["cosmetic"] = cosmetic;
        if let Some(differences) = artifact["differences"].as_array_mut() {
            differences.push(duplicate);
        }
        assert!(serde_json::from_value::<Comparison>(artifact).is_err());
    }
    Ok(())
}

#[test]
fn whitespace_is_classified_without_changing_raw_values() -> Result<()> {
    assert_cosmetic(
        json!({ "type": "Text", "value": ". Auxin signalling " }),
        json!({ "type": "Text", "value": ".\tAuxin  signalling\n" }),
        &["whitespace"],
    )
}

#[test]
fn typography_is_classified_without_changing_raw_values() -> Result<()> {
    assert_cosmetic(
        json!({ "type": "Text", "value": "Auxin’s role–signalling" }),
        json!({ "type": "Text", "value": "Auxin's role-signalling" }),
        &["typography"],
    )
}

#[test]
fn initials_are_classified_without_changing_raw_values() -> Result<()> {
    assert_cosmetic(
        json!({ "type": "Person", "givenNames": ["S.S"] }),
        json!({ "type": "Person", "givenNames": ["S", "S"] }),
        &["initials"],
    )
}
