//! The identity of a person, as distinct from metadata about them
//!
//! Generic subtree text lets long metadata, such as affiliations, outweigh a person's
//! own names: an author whose affiliation one source omits would be left unmatched
//! despite identical names, and two different authors at the same institution could be
//! paired. People are therefore matched by stable identifiers and by their own names.
//! An ORCID iD identifies one person, but an email address may be shared, by
//! co-authors writing from one lab address for example, so it only corroborates names.
//! Subtree text only ranks people whose names agree, and stands in for people without
//! names.
//!
//! A role, such as an author's CRediT role, is identified by the person who plays it.
//!
//! This selects matching evidence only: fingerprints, ancestor text, and property
//! comparisons remain complete.

use stencila_node_type::{NodeProperty, NodeType};
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

use crate::{
    alignment::AlignmentCost,
    error::CompareResult,
    projection::{Item, OccurrenceId, Projection},
    scalar::ScalarValue,
    text::{self, Grams},
};

/// The share of the similarity of people with agreeing names left to their subtrees
///
/// A tenth, so that metadata such as affiliations ranks otherwise equal candidates but
/// cannot, even when entirely different, make agreeing names cheaper to leave unmatched.
const SUBTREE_SHARE: i64 = 10;

/// What identifies a person, independently of metadata such as affiliations
#[derive(Debug, Clone, Default)]
pub struct PersonIdentity {
    /// Normalized ORCID iDs, sorted
    orcids: Vec<String>,

    /// Normalized email addresses, sorted
    emails: Vec<String>,

    /// Folded family names, split into words
    family_names: Vec<String>,

    /// Folded given names, split into names and initials
    ///
    /// Taken from the full name, less the family names, when there are no structured
    /// given names, so that people who share only a family name do not agree.
    given_names: Vec<String>,

    /// The n-grams of the person's own folded names, in sorted order so that neither
    /// case nor name order, nor whether names are structured, affects them
    names: Grams,
}

impl PersonIdentity {
    /// The identity of an occurrence, when it is a person or the role of a person
    pub fn new(projection: &Projection, id: OccurrenceId) -> CompareResult<Option<Self>> {
        let occurrence = projection.occurrence(id)?;
        match occurrence.node_type {
            NodeType::Person => {}
            NodeType::AuthorRole => {
                let author = occurrence
                    .properties
                    .iter()
                    .filter(|property| property.decl.property == NodeProperty::Author)
                    .flat_map(|property| property.items.iter())
                    .find_map(|item| match item {
                        Item::Structured(id) => Some(*id),
                        Item::Scalar(..) => None,
                    });
                return match author {
                    Some(author) => Self::new(projection, author),
                    None => Ok(None),
                };
            }
            _ => return Ok(None),
        }

        let (mut orcids, mut emails) = (Vec::new(), Vec::new());
        let (mut name, mut given_names, mut family_names) = (Vec::new(), Vec::new(), Vec::new());
        for property in &occurrence.properties {
            let values = property.items.iter().filter_map(|item| match item {
                Item::Scalar(ScalarValue::String { value }) => Some(value.as_str()),
                _ => None,
            });
            match property.decl.property {
                NodeProperty::Orcid => orcids.extend(values.filter_map(orcid)),
                NodeProperty::Emails => emails.extend(values.filter_map(email)),
                NodeProperty::Name => name.extend(values.flat_map(words)),
                NodeProperty::GivenNames => given_names.extend(values.flat_map(words)),
                NodeProperty::FamilyNames => family_names.extend(values.flat_map(words)),
                _ => {}
            }
        }
        for identifiers in [&mut orcids, &mut emails] {
            identifiers.sort();
            identifiers.dedup();
        }

        let mut own = if given_names.is_empty() && family_names.is_empty() {
            name
        } else {
            if given_names.is_empty() {
                given_names = without(name, &family_names);
            }
            given_names.iter().chain(&family_names).cloned().collect()
        };
        own.sort();

        Ok(Some(Self {
            orcids,
            emails,
            family_names,
            given_names,
            names: Grams::new(&own.join(" ")),
        }))
    }
}

/// The similarity of two people, as a cost between zero and one
///
/// - A shared ORCID iD is the same person, whatever their names.
/// - Different ORCID iDs are different people, whatever else they share.
/// - Agreeing names are the same person; the subtree similarity only ranks candidates.
/// - Conflicting names are different people, however much metadata they share.
/// - Otherwise names are compared as text, tolerating spelling and transcription
///   differences, without metadata.
/// - When either person has no names, the subtree similarity is used.
/// - A shared email address is the same person, unless their names are no more than
///   half similar.
pub fn similarity(
    left: &PersonIdentity,
    right: &PersonIdentity,
    subtree: AlignmentCost,
) -> AlignmentCost {
    if !left.orcids.is_empty() && !right.orcids.is_empty() {
        return if shares(&left.orcids, &right.orcids) {
            AlignmentCost::ONE
        } else {
            AlignmentCost::ZERO
        };
    }

    let shared_email = shares(&left.emails, &right.emails);
    if left.names.is_empty() || right.names.is_empty() {
        return if shared_email {
            AlignmentCost::ONE
        } else {
            subtree
        };
    }

    let similarity = match agreement(left, right) {
        Agreement::Compatible => {
            let scale = AlignmentCost::SCALE;
            AlignmentCost::from_units(scale - (scale - subtree.units()) / SUBTREE_SHARE)
        }
        Agreement::Conflicting => AlignmentCost::ZERO,
        Agreement::Undetermined => text::similarity(&left.names, &right.names),
    };
    if shared_email && similarity.units() * 2 > AlignmentCost::SCALE {
        AlignmentCost::ONE
    } else {
        similarity
    }
}

/// Whether two sorted lists of identifiers have one in common
fn shares(left: &[String], right: &[String]) -> bool {
    left.iter()
        .any(|identifier| right.binary_search(identifier).is_ok())
}

/// Whether two people's structured names agree
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Agreement {
    /// The same family names, and given names that are equal, initials of one another,
    /// or present on only one side
    Compatible,

    /// The same family names, and the same number of given names, but corresponding
    /// given names with different initials
    Conflicting,

    /// Anything else, including people without structured family names
    Undetermined,
}

/// The agreement of two people's structured names
fn agreement(left: &PersonIdentity, right: &PersonIdentity) -> Agreement {
    if left.family_names.is_empty() || left.family_names != right.family_names {
        return Agreement::Undetermined;
    }

    // Given names that do not line up, such as a first name present on only one side,
    // cannot show that two people differ
    let aligned = left.given_names.len() == right.given_names.len();
    let mut agreement = Agreement::Compatible;
    for (left, right) in left.given_names.iter().zip(&right.given_names) {
        if left == right || is_initial_of(left, right) || is_initial_of(right, left) {
            continue;
        }
        if aligned && left.chars().next() != right.chars().next() {
            return Agreement::Conflicting;
        }
        agreement = Agreement::Undetermined;
    }

    agreement
}

/// Whether a name is a single-letter initial of another
fn is_initial_of(initial: &str, name: &str) -> bool {
    initial.chars().count() == 1 && name.starts_with(initial)
}

/// Words with the first occurrence of each of another list's words removed
fn without(mut words: Vec<String>, removed: &[String]) -> Vec<String> {
    for removed in removed {
        if let Some(index) = words.iter().position(|word| word == removed) {
            words.remove(index);
        }
    }
    words
}

/// The folded words of a name, so that "S.S", "S. S." and "s s" agree
///
/// Folding lowercases and removes diacritics, so that "Jiří" and "Jiri" agree, and
/// replaces letters that have no decomposition, so that "Łukasz" and "Lukasz" agree.
fn words(name: &str) -> Vec<String> {
    let folded = name
        .nfd()
        .filter(|character| !is_combining_mark(*character))
        .flat_map(char::to_lowercase)
        .fold(String::new(), |mut folded, character| {
            match fold_letter(character) {
                Some(replacement) => folded.push_str(replacement),
                None => folded.push(character),
            }
            folded
        });
    folded
        .split(|character: char| character.is_whitespace() || matches!(character, '.' | '-'))
        .filter(|word| !word.is_empty())
        .map(String::from)
        .collect()
}

/// The usual ASCII replacement of a lowercase Latin letter that Unicode does not
/// decompose into a base letter and diacritics
fn fold_letter(character: char) -> Option<&'static str> {
    Some(match character {
        'ł' => "l",
        'ø' => "o",
        'đ' | 'ð' => "d",
        'ħ' => "h",
        'ı' => "i",
        'ŧ' => "t",
        'ß' => "ss",
        'æ' => "ae",
        'œ' => "oe",
        'þ' => "th",
        _ => return None,
    })
}

/// A normalized ORCID iD, without any URL prefix
fn orcid(value: &str) -> Option<String> {
    let value = value.trim();
    let value = ["https://orcid.org/", "http://orcid.org/", "orcid.org/"]
        .iter()
        .find_map(|prefix| value.strip_prefix(prefix))
        .unwrap_or(value)
        .to_uppercase();
    (!value.is_empty()).then_some(value)
}

/// A normalized email address, without any `mailto:` prefix
fn email(value: &str) -> Option<String> {
    let value = value.trim();
    let value = value
        .strip_prefix("mailto:")
        .unwrap_or(value)
        .to_lowercase();
    (!value.is_empty()).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_fold_case_diacritics_and_initials() {
        assert_eq!(words("S.S"), ["s", "s"]);
        assert_eq!(words(" S. S. "), ["s", "s"]);
        assert_eq!(words("Jiří"), ["jiri"]);
        assert_eq!(words("Łukasz Søren"), ["lukasz", "soren"]);
        assert_eq!(words("Strauß"), ["strauss"]);
        assert_eq!(words("Jean-Paul"), ["jean", "paul"]);
        assert_eq!(words("van der Berg"), ["van", "der", "berg"]);
    }

    #[test]
    fn full_names_stand_in_for_missing_given_names() {
        assert_eq!(without(words("Alexandra Li"), &words("Li")), ["alexandra"]);
        assert_eq!(without(words("Li Li"), &words("Li")), ["li"]);
    }

    #[test]
    fn identifiers_are_normalized() {
        assert_eq!(
            orcid("https://orcid.org/0000-0002-1825-009x"),
            Some("0000-0002-1825-009X".to_string())
        );
        assert_eq!(orcid("  "), None);
        assert_eq!(
            email("mailto:Alice.Smith@Example.org"),
            Some("alice.smith@example.org".to_string())
        );
    }

    #[test]
    fn given_names_agree_through_initials() {
        let person = |family: &str, given: &str| PersonIdentity {
            family_names: words(family),
            given_names: words(given),
            ..PersonIdentity::default()
        };
        let agree = |left, right| agreement(&person("Li", left), &person("Li", right));

        assert_eq!(agree("", "X"), Agreement::Compatible);
        assert_eq!(agree("X", "Xiang"), Agreement::Compatible);
        assert_eq!(agree("S.S", "S S"), Agreement::Compatible);
        assert_eq!(agree("X", "Y"), Agreement::Conflicting);
        assert_eq!(agree("Xiang", "Xiao"), Agreement::Undetermined);
        assert_eq!(agree("Mary Ann", "Ann"), Agreement::Undetermined);
        assert_eq!(agree("Mary Ann", "Mary Beth"), Agreement::Conflicting);
        assert_eq!(
            agreement(&person("Li", "X"), &person("Lee", "X")),
            Agreement::Undetermined
        );
    }
}
