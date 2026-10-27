//! Author identity should survive differences in affiliation metadata, without losing
//! stronger identity signals such as ORCID iDs and email addresses.

use eyre::{Result, ensure};
use serde_json::{Value, json};
use stencila_node_compare::{
    AlignmentSignal, CompareOptions, Comparison, Difference, DifferenceFilter, ScalarValue,
    ValueState, compare, compare_with_options,
};
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

fn role(author: &Value, name: &str) -> Value {
    json!({"type": "AuthorRole", "author": author, "roleName": name})
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
fn shared_email_does_not_pair_different_names() -> Result<()> {
    let person = |given: &str, family: &str| {
        json!({
            "type": "Person", "givenNames": [given], "familyNames": [family],
            "emails": ["lab@uni.edu"]
        })
    };
    let (_, pairs) = compare_authors(&[person("Alice", "Smith")], &[person("Bob", "Jones")])?;
    assert_eq!(pairs, 0);
    Ok(())
}

#[test]
fn shared_email_pairs_corroborating_names() -> Result<()> {
    let (_, pairs) = compare_authors(
        &[json!({"type": "Person", "name": "A. Smith", "emails": ["lab@uni.edu"]})],
        &[json!({"type": "Person", "name": "Alice Smith", "emails": ["lab@uni.edu"]})],
    )?;
    assert_eq!(pairs, 1);
    Ok(())
}

#[test]
fn different_orcids_are_different_people_despite_a_shared_email() -> Result<()> {
    let person = |orcid: &str| {
        json!({
            "type": "Person", "orcid": orcid, "name": "Alice Smith",
            "emails": ["lab@uni.edu"]
        })
    };
    let (_, pairs) = compare_authors(&[person(ORCID)], &[person("0000-0001-5109-3700")])?;
    assert_eq!(pairs, 0);
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
    assert_eq!(pairs_of(&comparison, NodeType::AuthorRole), 0);
    assert_eq!(pairs, 1);
    assert!(comparison.differences().iter().any(|difference| matches!(
        difference,
        Difference::PropertyPresenceChanged { .. }
            if difference.property() == Some(NodeProperty::Affiliations)
    )));
    Ok(())
}

/// The JATS decoder represents each CRediT role separately; TEI has plain people.
#[test]
fn credit_roles_pair_with_plain_people() -> Result<()> {
    let people = [
        json!({"type": "Person", "givenNames": ["Giorgio"], "familyNames": ["Dall'Olmo"]}),
        json!({"type": "Person", "givenNames": ["Francesco"], "familyNames": ["Nencioli"]}),
        json!({"type": "Person", "givenNames": ["Thomas"], "familyNames": ["Jackson"]}),
    ];
    let roles = [
        vec!["Conceptualization", "Software", "WritingOriginalDraft"],
        vec!["Software"],
        vec!["Software", "Validation"],
    ];
    let authors: Vec<_> = people
        .iter()
        .zip(roles)
        .flat_map(|(person, roles)| {
            roles
                .into_iter()
                .map(move |role| json!({"type": "AuthorRole", "author": person, "roleName": role}))
        })
        .collect();
    let (left, right) = (article(&authors)?, article(&people)?);
    let comparison = compare(&left, &right)?;
    assert_eq!(person_pairs(&comparison), 3);
    assert!(!comparison.alignment().has_one_sided());
    assert_eq!(comparison.differences().len(), 3);
    assert!(comparison.differences().iter().all(|difference| matches!(
        difference,
        Difference::ValueChanged { location, .. }
            if location.property == Some(NodeProperty::RoleName)
    )));
    assert_eq!(comparison, compare(&right, &left)?.invert());
    comparison.alignment().validate(&left, &right)?;

    let paths: Vec<_> = comparison
        .alignment()
        .pairs()
        .filter(|(left, ..)| left.node_type == NodeType::Person)
        .map(|(left, right, ..)| (left.path.to_string(), right.path.to_string()))
        .collect();
    assert_eq!(
        paths,
        [
            ("authors/0/author".into(), "authors/0".into()),
            ("authors/3/author".into(), "authors/1".into()),
            ("authors/4/author".into(), "authors/2".into()),
        ]
    );
    Ok(())
}

#[test]
fn role_names_compare_once_as_a_set() -> Result<()> {
    let person = json!({"type": "Person", "name": "Ada Lovelace"});
    let left = article(&[role(&person, "Software"), role(&person, "Validation")])?;
    let reordered = article(&[
        role(&person, "Validation"),
        role(&person, "Software"),
        role(&person, "Software"),
    ])?;
    assert!(compare(&left, &reordered)?.is_equal());

    let right = article(&[role(&person, "Software")])?;
    let comparison = compare(&left, &right)?;
    assert_eq!(comparison, compare(&right, &left)?.invert());
    assert_eq!(person_pairs(&comparison), 1);
    assert!(!comparison.alignment().has_one_sided());
    let [
        Difference::ValueChanged {
            location,
            left,
            right,
            ..
        },
    ] = comparison.differences()
    else {
        eyre::bail!("Expected exactly one role-name change");
    };
    assert_eq!(location.property, Some(NodeProperty::RoleName));
    let value = |variant: &str| ScalarValue::Enum {
        schema_type: "AuthorRoleName".into(),
        variant: variant.into(),
    };
    assert_eq!(
        left,
        &ValueState::Many {
            values: vec![value("Software"), value("Validation")]
        }
    );
    assert_eq!(
        right,
        &ValueState::Many {
            values: vec![value("Software")]
        }
    );
    Ok(())
}

#[test]
fn organizations_pair_through_roles_and_roles_can_be_filtered() -> Result<()> {
    for author in [
        json!({"type": "Person", "name": "Ada Lovelace"}),
        json!({"type": "Organization", "name": "Analytical Engine Society"}),
    ] {
        let left = article(&[role(&author, "Software"), role(&author, "Validation")])?;
        let right = article(std::slice::from_ref(&author))?;
        let comparison = compare(&left, &right)?;
        assert_eq!(comparison, compare(&right, &left)?.invert());
        assert!(!comparison.alignment().has_one_sided());
        assert_eq!(comparison.differences().len(), 1);
        let selector = format!(
            "{}.roleName",
            author["type"]
                .as_str()
                .ok_or_else(|| eyre::eyre!("Missing type"))?
        );
        for selector in ["roleName", selector.as_str()] {
            let options = CompareOptions {
                filter: DifferenceFilter {
                    exclude: vec![selector.parse()?],
                    ..Default::default()
                },
                ..Default::default()
            };
            let filtered = compare_with_options(&left, &right, &options)?;
            assert!(filtered.is_equal());
            assert_eq!(filtered.alignment(), comparison.alignment());
        }
    }
    Ok(())
}

#[test]
fn grouping_does_not_hide_affiliation_changes() -> Result<()> {
    let person = json!({"type": "Person", "name": "Ada Lovelace"});
    let mut affiliated = person.clone();
    affiliated["affiliations"] = affiliation("Analytical Engine Society");
    let left = article(&[
        role(&affiliated, "Software"),
        role(&affiliated, "Validation"),
    ])?;
    let right = article(&[person])?;
    let comparison = compare(&left, &right)?;
    assert_eq!(person_pairs(&comparison), 1);
    assert_eq!(comparison, compare(&right, &left)?.invert());
    assert!(
        comparison
            .differences()
            .iter()
            .any(|diff| diff.property() == Some(NodeProperty::Affiliations))
    );
    assert!(
        comparison
            .differences()
            .iter()
            .any(|diff| diff.property() == Some(NodeProperty::RoleName))
    );
    assert_eq!(
        comparison
            .alignment()
            .correspondences()
            .iter()
            .filter_map(|c| c.left())
            .filter(|node| node.node_type == NodeType::Organization)
            .count(),
        1
    );
    Ok(())
}

#[test]
fn grouping_requires_consecutive_roles_with_identical_authors() -> Result<()> {
    let person = json!({"type": "Person", "name": "Ada Lovelace"});
    let mut affiliated = person.clone();
    affiliated["affiliations"] = affiliation("Analytical Engine Society");
    for (authors, count) in [
        (
            vec![role(&person, "Software"), role(&affiliated, "Validation")],
            2,
        ),
        (
            vec![
                role(&person, "Software"),
                json!({"type": "Person", "name": "Charles Babbage"}),
                role(&person, "Validation"),
            ],
            3,
        ),
        (vec![person.clone(), person.clone()], 2),
        (vec![person.clone(), role(&person, "Software")], 2),
    ] {
        let node = article(&authors)?;
        let comparison = compare(&node, &node)?;
        assert!(comparison.is_equal());
        assert_eq!(person_pairs(&comparison), count);
        comparison.alignment().validate(&node, &node)?;
    }
    Ok(())
}

#[test]
fn richer_roles_and_other_author_types_keep_their_metadata() -> Result<()> {
    let person = json!({"type": "Person", "name": "Ada Lovelace"});
    for (property, value) in [
        ("format", json!("Python")),
        (
            "lastModified",
            json!({"type": "Timestamp", "value": 1, "timeUnit": "Second"}),
        ),
    ] {
        let mut richer = role(&person, "Software");
        richer[property] = value;
        let node = article(&[richer])?;
        let comparison = compare(&node, &node)?;
        assert!(comparison.is_equal());
        assert_eq!(pairs_of(&comparison, NodeType::AuthorRole), 1);
        comparison.alignment().validate(&node, &node)?;
    }
    let software = json!({"type": "SoftwareApplication", "name": "Assistant"});
    let node = article(&[role(&software, "Software")])?;
    let comparison = compare(&node, &node)?;
    assert!(comparison.is_equal());
    assert_eq!(pairs_of(&comparison, NodeType::AuthorRole), 1);
    Ok(())
}

/// IDs are never compared, so an ID-only edit must neither unwrap nor split a unit
#[test]
fn ids_do_not_change_author_units() -> Result<()> {
    let person = json!({"type": "Person", "name": "Ada Lovelace"});
    let mut identified = person.clone();
    identified["id"] = json!("ada");
    let mut wrapper = role(&person, "Software");
    wrapper["id"] = json!("role-1");
    let roles = article(&[role(&person, "Software"), role(&person, "Validation")])?;
    let plain = article(std::slice::from_ref(&person))?;

    for (edited, original) in [
        (article(&[wrapper])?, article(&[role(&person, "Software")])?),
        (
            article(&[role(&person, "Software"), role(&identified, "Validation")])?,
            roles.clone(),
        ),
    ] {
        let comparison = compare(&edited, &original)?;
        assert!(comparison.is_equal());
        assert_eq!(person_pairs(&comparison), 1);
        comparison.alignment().validate(&edited, &original)?;

        let comparison = compare(&edited, &plain)?;
        assert_eq!(person_pairs(&comparison), 1);
        assert_eq!(pairs_of(&comparison, NodeType::AuthorRole), 0);
        assert!(!comparison.alignment().has_one_sided());
        assert_eq!(
            comparison.differences(),
            compare(&original, &plain)?.differences()
        );
        assert_eq!(comparison, compare(&plain, &edited)?.invert());
    }
    Ok(())
}

#[test]
fn grouping_preserves_missing_authors_and_relative_author_order() -> Result<()> {
    let ada = json!({"type": "Person", "name": "Ada Lovelace", "orcid": ORCID});
    let charles = json!({"type": "Person", "name": "Charles Babbage"});
    let left = article(&[
        role(&ada, "Software"),
        role(&ada, "Validation"),
        charles.clone(),
    ])?;
    let right = article(&[charles.clone(), ada])?;
    let comparison = compare(&left, &right)?;
    assert_eq!(person_pairs(&comparison), 2);
    assert!(!comparison.alignment().has_one_sided());
    assert_eq!(comparison, compare(&right, &left)?.invert());
    assert_eq!(
        comparison
            .differences()
            .iter()
            .filter(|diff| matches!(diff, Difference::Reordered { .. }))
            .count(),
        1
    );

    let right = article(&[charles])?;
    let comparison = compare(&left, &right)?;
    assert_eq!(person_pairs(&comparison), 1);
    assert!(comparison.alignment().has_one_sided());
    assert_eq!(
        comparison
            .alignment()
            .correspondences()
            .iter()
            .filter(|c| c.right().is_none())
            .count(),
        1
    );
    assert_eq!(comparison, compare(&right, &left)?.invert());
    Ok(())
}

#[test]
fn an_author_can_still_move_to_another_property() -> Result<()> {
    let person = json!({"type": "Person", "name": "Ada Lovelace"});
    let left = article(std::slice::from_ref(&person))?;
    let right: Node =
        serde_json::from_value(json!({"type": "Article", "content": [], "editors": [person]}))?;
    let comparison = compare(&left, &right)?;
    assert_eq!(person_pairs(&comparison), 1);
    assert!(!comparison.alignment().has_one_sided());
    assert_eq!(comparison, compare(&right, &left)?.invert());
    assert!(
        comparison
            .differences()
            .iter()
            .any(|difference| matches!(difference, Difference::ParentChanged { .. }))
    );
    Ok(())
}
