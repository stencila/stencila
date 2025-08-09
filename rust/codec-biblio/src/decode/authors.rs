//! Parsers that parse a Stencila [`Author`] from a string

use winnow::{
    Parser, Result,
    ascii::{multispace0, multispace1},
    combinator::{alt, not, opt, peek, preceded, repeat, separated, terminated},
    token::{take, take_while},
};

use codec::schema::{Author, Organization, Person};

/// Parse multiple authors separated by various delimiters
pub fn authors(input: &mut &str) -> Result<Vec<Author>> {
    separated(
        1..,
        author,
        (
            multispace0,
            alt((", &", ", and", "&", "and", ",")),
            multispace0,
        ),
    )
    .map(|authors: Vec<Author>| {
        authors
            .into_iter()
            .filter(|author| match author {
                Author::Organization(org) => {
                    org.name != Some("et al".to_string()) && org.name != Some("...".to_string())
                }
                _ => true,
            })
            .collect()
    })
    .parse_next(input)
}

/// Parse a single author in various formats
pub fn author(input: &mut &str) -> Result<Author> {
    alt((
        person_family_initial_periods,
        person_family_initials,
        person_family_given,
        person_given_family,
        organization,
        ellipses,
        etal,
    ))
    .parse_next(input)
}

/// Parse multiple persons separated by various delimiters
pub fn persons(input: &mut &str) -> Result<Vec<Person>> {
    separated(
        1..,
        person,
        (multispace0, alt(("&", "and", ",", ", &")), multispace0),
    )
    .parse_next(input)
}

/// Parse a single person in various formats
pub fn person(input: &mut &str) -> Result<Person> {
    alt((person_family_given, person_given_family))
        .map(|author| match author {
            Author::Person(person) => person,
            _ => Person::default(),
        })
        .parse_next(input)
}

/// Parse person in "Family FM" format
///
/// Used in Vancouver style.
pub fn person_family_initials(input: &mut &str) -> Result<Author> {
    (
        // Family names
        terminated(separated(1.., name, multispace1), multispace1),
        // Initials (not separated by anything)
        repeat(1.., initial_letter.map(|letter| [letter, "."].concat())),
    )
        .map(|(family_names, given_names): (Vec<String>, Vec<String>)| {
            Author::Person(Person {
                family_names: Some(family_names),
                given_names: Some(given_names),
                ..Default::default()
            })
        })
        .parse_next(input)
}

/// Parse person in "Family, F. M." format
///
/// Used where it is necessary to be strick about ending periods to avoid
/// consuming the start of the title.
///
/// See `person_family_given` for a more lenient parser which allows for
/// deviations such as missing periods and complete given names.
pub fn person_family_initial_periods(input: &mut &str) -> Result<Author> {
    (
        // Family names
        terminated(separated(1.., name, multispace1), ","),
        // Initials with period
        preceded(
            multispace0,
            separated(
                1..,
                (initial_letter, ".").take().map(String::from),
                multispace1,
            ),
        ),
    )
        .map(|(family_names, given_names): (Vec<String>, Vec<String>)| {
            Author::Person(Person {
                family_names: Some(family_names),
                given_names: Some(given_names),
                ..Default::default()
            })
        })
        .parse_next(input)
}

/// Parse person in "Family, F. M." format and deviations
///
/// As used for all authors in APA and first author in MLA.
///
/// Handles deviations:
///
/// - missing period after initials
/// - given names rather than initials
///
/// Does not handle missing comma after family name because that would parse
/// incorrect multiple family names incorrectly.
///
/// Note that the terminating period is intentionally included in the given
/// names to indicate it is an initial, not a complete given name.
pub fn person_family_given(input: &mut &str) -> Result<Author> {
    (
        // Family names
        terminated(separated(1.., name, multispace1), ","),
        // Given names or initials
        preceded(
            multispace0,
            separated(1.., alt((initial, name)), multispace1),
        ),
    )
        .map(|(family_names, given_names): (Vec<String>, Vec<String>)| {
            Author::Person(Person {
                family_names: Some(family_names),
                given_names: Some(given_names),
                ..Default::default()
            })
        })
        .parse_next(input)
}

/// Parse person in "First M. Family" or "F. M. Family" format and deviations
///
/// Handles deviations:
///
/// - missing period after initials
/// - given names rather than initials
///
/// To avoid matching "John Smith" etc to an organization, this requires that
/// first name and family names are (a) not all caps, (b) contains a word
/// generally only used by organizations and not people (e.g. "Institute",
/// "Association")
///
/// As used for second and subsequent authors in MLA.
pub fn person_given_family(input: &mut &str) -> Result<Author> {
    (
        // First given name or initial
        terminated(alt((initial, name)), multispace1),
        // Other initials
        opt(terminated(
            separated(1.., initial, multispace1),
            multispace1,
        )),
        // Family names
        separated(1.., name, multispace1),
    )
        .verify(
            |(first, _initials, family_names): &(String, Option<Vec<String>>, Vec<String>)| {
                !is_likely_organization(first, family_names)
            },
        )
        .map(
            |(first, initials, family_names): (String, Option<Vec<String>>, Vec<String>)| {
                let mut given_names = vec![first.to_string()];
                if let Some(initials) = initials {
                    given_names.append(&mut initials.into_iter().collect());
                }

                Author::Person(Person {
                    given_names: Some(given_names),
                    family_names: Some(family_names.into_iter().collect()),
                    ..Default::default()
                })
            },
        )
        .parse_next(input)
}

/// Parse an organization
///
/// Parses a list of whitespace separated names and them joins them to avoid
/// trailing whitespace being consumed.
///
/// Generally, used as a fallback if the string does not match expected format
/// for a [`Person`].
pub fn organization(input: &mut &str) -> Result<Author> {
    separated(
        1..,
        take_while(2.., |c: char| {
            c.is_alphabetic() || c.is_numeric() || is_hyphen(c)
        }),
        multispace1,
    )
    .map(|name: Vec<&str>| {
        Author::Organization(Organization {
            name: Some(name.join(" ")),
            ..Default::default()
        })
    })
    .parse_next(input)
}

/// Parse "et al" (an variations as an author)
pub fn etal(input: &mut &str) -> Result<Author> {
    alt(("et. al.", "et al.", "et al"))
        .map(|_| {
            Author::Organization(Organization {
                name: Some("et al".into()),
                ..Default::default()
            })
        })
        .parse_next(input)
}

/// Parse "..." (ellipses as an author)
pub fn ellipses(input: &mut &str) -> Result<Author> {
    alt(("...", "\u{2026}"))
        .map(|_| {
            Author::Organization(Organization {
                name: Some("...".into()),
                ..Default::default()
            })
        })
        .parse_next(input)
}

/// Parse a proper name: starts with uppercase letter, followed by alphabetic characters or hyphens
///
/// This parser matches personal names, family names, and place names that follow standard
/// capitalization conventions. It handles:
///
/// - Simple names: "Smith", "John", "Mary"
/// - Hyphenated names: "Smith-Jones", "Jean-Pierre", "St-Pierre"
/// - Names with various Unicode hyphens: "García-López", "O'Connor"
/// - Multi-part names when used in sequences: "Van Der Berg"
///
/// The parser requires the first character to be uppercase and alphabetic, followed by
/// one or more characters that are either alphabetic or hyphen variants. This ensures
/// proper name capitalization while allowing for hyphenated compound names.
///
/// Used in both family name and given name parsing contexts.
fn name(input: &mut &str) -> Result<String> {
    (
        take_while(1..=1, |c: char| c.is_uppercase() && c.is_alphabetic()),
        take_while(1..=1, |c: char| {
            (c.is_lowercase() && c.is_alphabetic()) || is_apostrophe(c)
        }),
        take_while(0.., |c: char| {
            c.is_alphabetic() || is_hyphen(c) || is_apostrophe(c)
        }),
    )
        .take()
        .map(|s: &str| s.to_string())
        .parse_next(input)
}

/// Parse a single uppercase letter for an initial
fn initial_letter<'s>(input: &mut &'s str) -> Result<&'s str> {
    take(1usize)
        .verify(|s: &str| {
            let chars: Vec<char> = s.chars().collect();
            chars.len() == 1 && chars[0].is_uppercase() && chars[0].is_alphabetic()
        })
        .parse_next(input)
}

/// Parse an initial: single uppercase alphabetic character, optionally with a period
///
/// This parser matches patterns like "A", "B.", "M", "J." and takes the period if present.
/// The result includes the period to indicate it's an initial rather than a full name.
fn initial(input: &mut &str) -> Result<String> {
    (
        initial_letter,
        opt("."),
        // Ensure no more alphabetic characters follow (to distinguish from names)
        peek(not(take_while(1.., |c: char| c.is_alphabetic()))),
    )
        .take()
        .map(|initial: &str| {
            if initial.ends_with(".") {
                initial.to_string()
            } else {
                [initial, "."].concat()
            }
        })
        .parse_next(input)
}

fn is_hyphen(c: char) -> bool {
    // Hyphen-minus, En dash, Hyphen, Figure dash, Em dash, Horizontal bar, Minus sign
    matches!(c, '-' | '–' | '‐' | '‒' | '—' | '―' | '−')
}

fn is_apostrophe(c: char) -> bool {
    // Apostrophe, right single quotation mark
    matches!(c, '\'' | '\u{2019}')
}

/// Check if a parsed name is likely an organization rather than a person
///
/// Returns true if:
/// - All words are in ALL CAPS
/// - Contains organizational keywords like "Institute", "Association", etc.
fn is_likely_organization(first: &String, family_names: &[String]) -> bool {
    let all_words: Vec<&String> = std::iter::once(first).chain(family_names.iter()).collect();

    // Check if all words are in ALL CAPS (indicating likely organization)
    // But exclude single letters or short initials with periods
    if all_words.len() >= 2
        && all_words.iter().all(|word| {
            let clean_word = word.trim_end_matches('.');
            // Must be longer than 2 chars to be considered "all caps organization"
            clean_word.len() > 2
                && clean_word
                    .chars()
                    .all(|c| !c.is_alphabetic() || c.is_uppercase())
        })
    {
        return true;
    }

    // Check for organizational keywords
    let org_keywords = [
        "Institute",
        "Institution",
        "Association",
        "Organization",
        "Organisation",
        "Foundation",
        "Society",
        "Academy",
        "University",
        "College",
        "School",
        "Department",
        "Ministry",
        "Agency",
        "Bureau",
        "Office",
        "Center",
        "Centre",
        "Council",
        "Committee",
        "Board",
        "Group",
        "Corporation",
        "Company",
        "Ltd",
        "Inc",
        "LLC",
        "Trust",
        "Fund",
        "Bank",
        "Union",
        "Federation",
        "Alliance",
        "Consortium",
        "Partnership",
        "Network",
        "Authority",
        "Commission",
        "Service",
    ];
    all_words.iter().any(|word| {
        let clean_word = word.trim_end_matches('.');
        org_keywords
            .iter()
            .any(|keyword| clean_word.eq_ignore_ascii_case(keyword))
    })
}

#[cfg(test)]
mod tests {
    use common_dev::pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_name() -> Result<()> {
        // Simple names
        assert_eq!(name(&mut "Smith"), Ok("Smith".to_string()));
        assert_eq!(name(&mut "John"), Ok("John".to_string()));
        assert_eq!(name(&mut "Mary"), Ok("Mary".to_string()));

        // Hyphenated names with standard hyphen
        assert_eq!(name(&mut "Smith-Jones"), Ok("Smith-Jones".to_string()));
        assert_eq!(name(&mut "Jean-Pierre"), Ok("Jean-Pierre".to_string()));

        // Names with Unicode hyphens
        assert_eq!(name(&mut "García-López"), Ok("García-López".to_string()));
        assert_eq!(name(&mut "O'Connor"), Ok("O'Connor".to_string()));

        // Multi-character names (minimum 2 chars to avoid clash with initials)
        assert_eq!(name(&mut "St"), Ok("St".to_string()));
        assert_eq!(name(&mut "Du"), Ok("Du".to_string()));

        // Should not match single letters (reserved for initials)
        assert!(name(&mut "A").is_err());
        assert!(name(&mut "B").is_err());

        // Should not match lowercase start
        assert!(name(&mut "smith").is_err());
        assert!(name(&mut "john").is_err());

        // Should not match numbers or special chars at start
        assert!(name(&mut "3Smith").is_err());
        assert!(name(&mut "@John").is_err());

        Ok(())
    }

    #[test]
    fn test_initial() -> Result<()> {
        // Standard initials with periods
        assert_eq!(initial(&mut "A."), Ok("A.".to_string()));
        assert_eq!(initial(&mut "B."), Ok("B.".to_string()));
        assert_eq!(initial(&mut "Z."), Ok("Z.".to_string()));

        // Initials without periods
        assert_eq!(initial(&mut "A"), Ok("A.".to_string()));
        assert_eq!(initial(&mut "M"), Ok("M.".to_string()));

        // Should not match lowercase
        assert!(initial(&mut "a.").is_err());
        assert!(initial(&mut "m").is_err());

        // Should not match multiple characters
        assert!(initial(&mut "AB").is_err());
        assert!(initial(&mut "John").is_err());

        // Should not match non-alphabetic
        assert!(initial(&mut "1.").is_err());
        assert!(initial(&mut "@").is_err());

        Ok(())
    }

    #[test]
    fn test_authors() -> Result<()> {
        // Single person
        let items = authors(&mut "Author, A. B.")?;
        assert_eq!(items.len(), 1);

        // Two people with ampersand
        let items = authors(&mut "Author, A. B., & Author, B. C.")?;
        assert_eq!(items.len(), 2);

        // Two people with and
        let items = authors(&mut "Author, A. B., and B. C. Author")?;
        assert_eq!(items.len(), 2);

        // With et al
        let items = authors(&mut "L. Chen, S. Martinez, R. Johnson, et al.")?;
        assert_eq!(items.len(), 3);

        Ok(())
    }
    // In the following tests we parse using `authors` as a test of differentiating between different
    // authors types and that the sub-parsers to not conflict

    #[test]
    fn test_person_family_given() -> Result<()> {
        // Standard format with periods after initials
        if let Author::Person(Person {
            family_names,
            given_names,
            ..
        }) = author(&mut "Smith, J. A.")?
        {
            assert_eq!(family_names, Some(vec!["Smith".to_string()]));
            assert_eq!(given_names, Some(vec!["J.".to_string(), "A.".to_string()]));
        } else {
            unreachable!("expected person")
        }

        // Missing periods after some initials
        if let Author::Person(Person {
            family_names,
            given_names,
            ..
        }) = author(&mut "Smith, J A.")?
        {
            assert_eq!(family_names, Some(vec!["Smith".to_string()]));
            assert_eq!(given_names, Some(vec!["J.".to_string(), "A.".to_string()]));
        } else {
            unreachable!("expected person")
        }

        // Compound family name with full first name and initial
        if let Author::Person(Person {
            family_names,
            given_names,
            ..
        }) = author(&mut "One Two, John A.")?
        {
            assert_eq!(
                family_names,
                Some(vec!["One".to_string(), "Two".to_string()])
            );
            assert_eq!(
                given_names,
                Some(vec!["John".to_string(), "A.".to_string()])
            );
        } else {
            unreachable!("expected person")
        }

        // Single initial
        if let Author::Person(Person {
            family_names,
            given_names,
            ..
        }) = author(&mut "Johnson, M.")?
        {
            assert_eq!(family_names, Some(vec!["Johnson".to_string()]));
            assert_eq!(given_names, Some(vec!["M.".to_string()]));
        } else {
            unreachable!("expected person")
        }

        // Single initial without period
        if let Author::Person(Person {
            family_names,
            given_names,
            ..
        }) = author(&mut "Wilson, R")?
        {
            assert_eq!(family_names, Some(vec!["Wilson".to_string()]));
            assert_eq!(given_names, Some(vec!["R.".to_string()]));
        } else {
            unreachable!("expected person")
        }

        // Multiple initials, all with periods
        if let Author::Person(Person {
            family_names,
            given_names,
            ..
        }) = author(&mut "Brown, A. B. C.")?
        {
            assert_eq!(family_names, Some(vec!["Brown".to_string()]));
            assert_eq!(
                given_names,
                Some(vec!["A.".to_string(), "B.".to_string(), "C.".to_string()])
            );
        } else {
            unreachable!("expected person")
        }

        // Mixed initials and full names
        if let Author::Person(Person {
            family_names,
            given_names,
            ..
        }) = author(&mut "Garcia, Maria J.")?
        {
            assert_eq!(family_names, Some(vec!["Garcia".to_string()]));
            assert_eq!(
                given_names,
                Some(vec!["Maria".to_string(), "J.".to_string()])
            );
        } else {
            unreachable!("expected person")
        }

        // Hyphenated family name
        if let Author::Person(Person {
            family_names,
            given_names,
            ..
        }) = author(&mut "Smith-Jones, K. L.")?
        {
            assert_eq!(family_names, Some(vec!["Smith-Jones".to_string()]));
            assert_eq!(given_names, Some(vec!["K.".to_string(), "L.".to_string()]));
        } else {
            unreachable!("expected person")
        }

        // Full first and middle names
        if let Author::Person(Person {
            family_names,
            given_names,
            ..
        }) = author(&mut "Williams, Mary Elizabeth")?
        {
            assert_eq!(family_names, Some(vec!["Williams".to_string()]));
            assert_eq!(
                given_names,
                Some(vec!["Mary".to_string(), "Elizabeth".to_string()])
            );
        } else {
            unreachable!("expected person")
        }

        // Three-part family name
        if let Author::Person(Person {
            family_names,
            given_names,
            ..
        }) = author(&mut "Van Der Berg, P. Q.")?
        {
            assert_eq!(
                family_names,
                Some(vec![
                    "Van".to_string(),
                    "Der".to_string(),
                    "Berg".to_string()
                ])
            );
            assert_eq!(given_names, Some(vec!["P.".to_string(), "Q.".to_string()]));
        } else {
            unreachable!("expected person")
        }

        Ok(())
    }

    #[test]
    fn test_person_family_initials() -> Result<()> {
        // Single initial
        if let Author::Person(Person {
            family_names,
            given_names,
            ..
        }) = author(&mut "Johnson M")?
        {
            assert_eq!(family_names, Some(vec!["Johnson".to_string()]));
            assert_eq!(given_names, Some(vec!["M.".to_string()]));
        } else {
            unreachable!("expected person")
        }

        // Multiple initials
        if let Author::Person(Person {
            family_names,
            given_names,
            ..
        }) = author(&mut "Brown ABC")?
        {
            assert_eq!(family_names, Some(vec!["Brown".to_string()]));
            assert_eq!(
                given_names,
                Some(vec!["A.".to_string(), "B.".to_string(), "C.".to_string()])
            );
        } else {
            unreachable!("expected person")
        }

        // Compound family name
        if let Author::Person(Person {
            family_names,
            given_names,
            ..
        }) = author(&mut "Van Der Berg PQ")?
        {
            assert_eq!(
                family_names,
                Some(vec![
                    "Van".to_string(),
                    "Der".to_string(),
                    "Berg".to_string()
                ])
            );
            assert_eq!(given_names, Some(vec!["P.".to_string(), "Q.".to_string()]));
        } else {
            unreachable!("expected person")
        }

        // Hyphenated family name
        if let Author::Person(Person {
            family_names,
            given_names,
            ..
        }) = author(&mut "Smith-Jones KL")?
        {
            assert_eq!(family_names, Some(vec!["Smith-Jones".to_string()]));
            assert_eq!(given_names, Some(vec!["K.".to_string(), "L.".to_string()]));
        } else {
            unreachable!("expected person")
        }

        Ok(())
    }

    #[test]
    fn test_person_family_initial_periods() -> Result<()> {
        // Standard format with periods after initials
        if let Author::Person(Person {
            family_names,
            given_names,
            ..
        }) = author(&mut "Smith, J. A.")?
        {
            assert_eq!(family_names, Some(vec!["Smith".to_string()]));
            assert_eq!(given_names, Some(vec!["J.".to_string(), "A.".to_string()]));
        } else {
            unreachable!("expected person")
        }

        // Single initial with period
        if let Author::Person(Person {
            family_names,
            given_names,
            ..
        }) = author(&mut "Johnson, M.")?
        {
            assert_eq!(family_names, Some(vec!["Johnson".to_string()]));
            assert_eq!(given_names, Some(vec!["M.".to_string()]));
        } else {
            unreachable!("expected person")
        }

        // Multiple initials, all with periods
        if let Author::Person(Person {
            family_names,
            given_names,
            ..
        }) = author(&mut "Brown, A. B. C.")?
        {
            assert_eq!(family_names, Some(vec!["Brown".to_string()]));
            assert_eq!(
                given_names,
                Some(vec!["A.".to_string(), "B.".to_string(), "C.".to_string()])
            );
        } else {
            unreachable!("expected person")
        }

        // Compound family name
        if let Author::Person(Person {
            family_names,
            given_names,
            ..
        }) = author(&mut "Van Der Berg, P. Q.")?
        {
            assert_eq!(
                family_names,
                Some(vec![
                    "Van".to_string(),
                    "Der".to_string(),
                    "Berg".to_string()
                ])
            );
            assert_eq!(given_names, Some(vec!["P.".to_string(), "Q.".to_string()]));
        } else {
            unreachable!("expected person")
        }

        // Hyphenated family name
        if let Author::Person(Person {
            family_names,
            given_names,
            ..
        }) = author(&mut "Smith-Jones, K. L.")?
        {
            assert_eq!(family_names, Some(vec!["Smith-Jones".to_string()]));
            assert_eq!(given_names, Some(vec!["K.".to_string(), "L.".to_string()]));
        } else {
            unreachable!("expected person")
        }

        Ok(())
    }

    #[test]
    fn test_person_given_family() -> Result<()> {
        // Standard format with periods after initials
        if let Author::Person(Person {
            family_names,
            given_names,
            ..
        }) = author(&mut "J. A. Smith")?
        {
            assert_eq!(family_names, Some(vec!["Smith".to_string()]));
            assert_eq!(given_names, Some(vec!["J.".to_string(), "A.".to_string()]));
        } else {
            unreachable!("expected person")
        }

        // Missing periods after some initials
        if let Author::Person(Person {
            family_names,
            given_names,
            ..
        }) = author(&mut "J A Smith")?
        {
            assert_eq!(family_names, Some(vec!["Smith".to_string()]));
            assert_eq!(given_names, Some(vec!["J.".to_string(), "A.".to_string()]));
        } else {
            unreachable!("expected person")
        }

        // Full first name and initial
        if let Author::Person(Person {
            family_names,
            given_names,
            ..
        }) = person_given_family(&mut "John A. Smith")?
        {
            assert_eq!(
                given_names,
                Some(vec!["John".to_string(), "A.".to_string()])
            );
            assert_eq!(family_names, Some(vec!["Smith".to_string()]));
        } else {
            unreachable!("expected person")
        }

        // Single initial
        if let Author::Person(Person {
            family_names,
            given_names,
            ..
        }) = author(&mut "M. Johnson")?
        {
            assert_eq!(family_names, Some(vec!["Johnson".to_string()]));
            assert_eq!(given_names, Some(vec!["M.".to_string()]));
        } else {
            unreachable!("expected person")
        }

        // Multiple initials, all with periods
        if let Author::Person(Person {
            family_names,
            given_names,
            ..
        }) = author(&mut "A. B. C. Brown")?
        {
            assert_eq!(family_names, Some(vec!["Brown".to_string()]));
            assert_eq!(
                given_names,
                Some(vec!["A.".to_string(), "B.".to_string(), "C.".to_string()])
            );
        } else {
            unreachable!("expected person")
        }

        // Hyphenated family name
        if let Author::Person(Person {
            family_names,
            given_names,
            ..
        }) = author(&mut "K. L. Smith-Jones")?
        {
            assert_eq!(given_names, Some(vec!["K.".to_string(), "L.".to_string()]));
            assert_eq!(family_names, Some(vec!["Smith-Jones".to_string()]));
        } else {
            unreachable!("expected person")
        }

        // Compound family name and additional spacing
        if let Author::Person(Person {
            family_names,
            given_names,
            ..
        }) = author(&mut "S  I. Sanchez   Gomez")?
        {
            assert_eq!(given_names, Some(vec!["S.".to_string(), "I.".to_string()]));
            assert_eq!(
                family_names,
                Some(vec!["Sanchez".to_string(), "Gomez".to_string()])
            );
        } else {
            unreachable!("expected person")
        }

        Ok(())
    }

    #[test]
    fn test_organization() -> Result<()> {
        // Simple organization
        if let Author::Organization(org) = author(&mut "World Health Organization")? {
            assert_eq!(org.name, Some("World Health Organization".to_string()));
        } else {
            unreachable!("expected organization")
        }

        // Organization with numbers
        if let Author::Organization(org) = author(&mut "Group of 20")? {
            assert_eq!(org.name, Some("Group of 20".to_string()));
        } else {
            unreachable!("expected organization")
        }

        // University
        if let Author::Organization(org) = author(&mut "University of California")? {
            assert_eq!(org.name, Some("University of California".to_string()));
        } else {
            unreachable!("expected organization")
        }

        // Government agency
        if let Author::Organization(org) = author(&mut "Environmental Protection Agency")? {
            assert_eq!(
                org.name,
                Some("Environmental Protection Agency".to_string())
            );
        } else {
            unreachable!("expected organization")
        }

        // International organization
        if let Author::Organization(org) = author(&mut "UNESCO")? {
            assert_eq!(org.name, Some("UNESCO".to_string()));
        } else {
            unreachable!("expected organization")
        }

        // Research institute
        if let Author::Organization(org) = author(&mut "Max Planck Institute for Biology")? {
            assert_eq!(
                org.name,
                Some("Max Planck Institute for Biology".to_string())
            );
        } else {
            unreachable!("expected organization")
        }

        Ok(())
    }
}
