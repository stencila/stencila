//! Author identity should survive differences in affiliation metadata, without losing
//! stronger identity signals such as ORCID iDs and email addresses.

use eyre::{Result, ensure};
use serde_json::{Value, json};
use stencila_node_compare::{AlignmentSignal, Comparison, Difference, compare};
use stencila_node_type::{NodeProperty, NodeType};
use stencila_schema::Node;

const ORCID: &str = "0000-0002-1825-0097";

const LONG_AFFILIATION: &str = "Laboratory of Growth Regulators, The Czech Academy of Sciences, Institute of Experimental Botany & Palacký University";

/// An article with the given authors
fn article(authors: &[Value]) -> Result<Node> {
    Ok(serde_json::from_value(json!({
        "type": "Article", "content": [], "authors": authors
    }))?)
}

/// An affiliation list with one organization
fn affiliation(name: &str) -> Value {
    json!([{"type": "Organization", "name": name}])
}

/// A person with the given names and, optionally, affiliations
fn person(names: &Value, affiliations: Option<&Value>) -> Value {
    let mut person = names.clone();
    person["type"] = json!("Person");
    if let Some(affiliations) = affiliations {
        person["affiliations"] = affiliations.clone();
    }
    person
}

/// The number of paired occurrences of a type in a comparison
fn pairs_of(comparison: &Comparison, node_type: NodeType) -> usize {
    comparison
        .alignment()
        .pairs()
        .filter(|(left, ..)| left.node_type == node_type)
        .count()
}

/// The number of paired people in a comparison
fn person_pairs(comparison: &Comparison) -> usize {
    pairs_of(comparison, NodeType::Person)
}

/// Compare articles with the given authors in both directions, requiring that the
/// number of paired people does not depend on the order of the inputs
fn compare_authors(left: &[Value], right: &[Value]) -> Result<(Comparison, usize)> {
    let (left, right) = (article(left)?, article(right)?);
    let comparison = compare(&left, &right)?;
    let pairs = person_pairs(&comparison);
    let reverse = person_pairs(&compare(&right, &left)?);
    ensure!(
        pairs == reverse,
        "Paired {pairs} people forwards but {reverse} backwards"
    );
    Ok((comparison, pairs))
}

#[test]
fn identical_names_pair_despite_missing_affiliations() -> Result<()> {
    let (comparison, pairs) = compare_authors(
        &[json!({
            "type": "Person", "givenNames": ["Jakub"], "familyNames": ["Hajný"],
            "affiliations": affiliation(LONG_AFFILIATION)
        })],
        &[json!({"type": "Person", "givenNames": ["Jakub"], "familyNames": ["Hajný"]})],
    )?;
    assert_eq!(pairs, 1);
    assert!(comparison.alignment().pairs().any(|(.., info)| {
        info.evidence
            .iter()
            .any(|evidence| evidence.signal == AlignmentSignal::PersonIdentity)
    }));
    assert!(comparison.differences().iter().any(|difference| matches!(
        difference,
        Difference::PropertyPresenceChanged { .. }
            if difference.property() == Some(NodeProperty::Affiliations)
    )));
    assert!(!comparison.is_equal());
    Ok(())
}

#[test]
fn shared_affiliations_do_not_pair_different_names() -> Result<()> {
    let person = |name| json!({"type": "Person", "name": name, "affiliations": affiliation(LONG_AFFILIATION)});
    let (_, pairs) = compare_authors(&[person("Xu")], &[person("Friml")])?;
    assert_eq!(pairs, 0);
    Ok(())
}

#[test]
fn named_author_pairs_with_changed_affiliation_and_extra_empty_author() -> Result<()> {
    let person = |organization| json!({"type": "Person", "name": "Jiří Friml", "affiliations": affiliation(organization)});
    let (comparison, pairs) = compare_authors(
        &[person("Institute of Experimental Botany")],
        &[person("University of Vienna"), json!({"type": "Person"})],
    )?;
    assert_eq!(pairs, 1);
    assert!(comparison.alignment().has_one_sided());
    assert!(!comparison.is_equal());
    Ok(())
}

#[test]
fn shared_orcid_pairs_an_unnamed_author_with_a_named_one() -> Result<()> {
    let (_, pairs) = compare_authors(
        &[json!({"type": "Person", "orcid": ORCID})],
        &[
            json!({"type": "Person", "orcid": format!("https://orcid.org/{ORCID}"), "name": "Alice Smith"}),
        ],
    )?;
    assert_eq!(pairs, 1);
    Ok(())
}

#[test]
fn shared_email_pairs_an_unnamed_author_with_a_named_one() -> Result<()> {
    let (_, pairs) = compare_authors(
        &[json!({"type": "Person", "emails": ["alice.smith@example.org"]})],
        &[json!({"type": "Person", "emails": ["Alice.Smith@example.org"], "name": "Alice Smith"})],
    )?;
    assert_eq!(pairs, 1);
    Ok(())
}

#[test]
fn shared_orcid_pairs_transliterated_names() -> Result<()> {
    let (_, pairs) = compare_authors(
        &[json!({"type": "Person", "orcid": ORCID, "name": "Иван Петров"})],
        &[json!({"type": "Person", "orcid": ORCID, "name": "Ivan Petrov"})],
    )?;
    assert_eq!(pairs, 1);
    Ok(())
}

#[test]
fn short_family_name_pairs_when_given_names_are_missing() -> Result<()> {
    let affiliation = affiliation("Institute of Experimental Botany");
    for affiliation in [Some(affiliation), None] {
        let person = |given: &[&str]| {
            json!({
                "type": "Person", "givenNames": given, "familyNames": ["Li"],
                "affiliations": affiliation
            })
        };
        let (_, pairs) = compare_authors(&[person(&[])], &[person(&["X"])])?;
        assert_eq!(pairs, 1);
    }
    Ok(())
}

#[test]
fn conflicting_initials_do_not_pair_despite_shared_affiliations() -> Result<()> {
    let person = |given| {
        json!({
            "type": "Person", "givenNames": [given], "familyNames": ["Li"],
            "affiliations": affiliation("Institute of Experimental Botany")
        })
    };
    let (_, pairs) = compare_authors(&[person("X")], &[person("Y")])?;
    assert_eq!(pairs, 0);
    Ok(())
}

#[test]
fn initials_and_diacritics_do_not_prevent_pairing() -> Result<()> {
    let person = |given: Value, family| json!({"type": "Person", "givenNames": given, "familyNames": [family]});
    for (left, right) in [
        (
            person(json!(["S.S"]), "Rao"),
            person(json!(["S", "S"]), "Rao"),
        ),
        (
            person(json!(["Jiří"]), "Friml"),
            person(json!(["Jiri"]), "Friml"),
        ),
        (
            person(json!(["J"]), "Hajný"),
            person(json!(["Jakub"]), "Hajny"),
        ),
        (
            person(json!(["Łukasz"]), "Kowalski"),
            person(json!(["Lukasz"]), "Kowalski"),
        ),
    ] {
        let (_, pairs) = compare_authors(&[left], &[right])?;
        assert_eq!(pairs, 1);
    }
    Ok(())
}

#[test]
fn equivalent_name_representations_pair() -> Result<()> {
    let names = [
        (json!({"name": "XU LI"}), json!({"name": "Xu Li"})),
        (
            json!({"givenNames": ["XU"], "familyNames": ["LI"]}),
            json!({"name": "Xu Li"}),
        ),
        (
            json!({"name": "Xu Li"}),
            json!({"givenNames": ["Xu"], "familyNames": ["Li"]}),
        ),
        (
            json!({"name": "Li Xu"}),
            json!({"givenNames": ["Xu"], "familyNames": ["Li"]}),
        ),
    ];
    let affiliation = affiliation("Institute of Experimental Botany");
    for affiliation in [Some(&affiliation), None] {
        for (left, right) in &names {
            let (_, pairs) =
                compare_authors(&[person(left, affiliation)], &[person(right, affiliation)])?;
            ensure!(pairs == 1, "Expected {left} and {right} to pair");
        }
    }
    Ok(())
}

#[test]
fn shared_family_name_does_not_pair_different_full_names() -> Result<()> {
    let left = json!({"name": "Alexandra Li", "familyNames": ["Li"]});
    let affiliation = affiliation("Institute of Experimental Botany");
    for right in [
        json!({"name": "Benjamin Li", "familyNames": ["Li"]}),
        json!({"givenNames": ["Benjamin"], "familyNames": ["Li"]}),
    ] {
        for affiliation in [Some(&affiliation), None] {
            let (_, pairs) = compare_authors(
                &[person(&left, affiliation)],
                &[person(&right, affiliation)],
            )?;
            ensure!(pairs == 0, "Expected {left} and {right} not to pair");
        }
    }
    Ok(())
}

#[test]
fn missing_first_given_name_does_not_conflict() -> Result<()> {
    let affiliation = affiliation("Institute of Experimental Botany");
    let (_, pairs) = compare_authors(
        &[person(
            &json!({"givenNames": ["Mary", "Ann"], "familyNames": ["Smith"]}),
            Some(&affiliation),
        )],
        &[person(
            &json!({"givenNames": ["Ann"], "familyNames": ["Smith"]}),
            Some(&affiliation),
        )],
    )?;
    assert_eq!(pairs, 1);
    Ok(())
}

#[test]
fn author_roles_pair_by_their_people_despite_missing_affiliations() -> Result<()> {
    let names = json!({"givenNames": ["Jakub"], "familyNames": ["Hajný"]});
    let role =
        |person| json!({"type": "AuthorRole", "author": person, "roleName": "Conceptualization"});
    let (comparison, pairs) = compare_authors(
        &[role(person(&names, Some(&affiliation(LONG_AFFILIATION))))],
        &[role(person(&names, None))],
    )?;
    assert_eq!(pairs_of(&comparison, NodeType::AuthorRole), 1);
    assert_eq!(pairs, 1);
    assert!(comparison.differences().iter().any(|difference| matches!(
        difference,
        Difference::PropertyPresenceChanged { .. }
            if difference.property() == Some(NodeProperty::Affiliations)
    )));
    Ok(())
}
