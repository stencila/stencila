//! Parsers that parse a Stencila [`Author`] from a string

use winnow::{
    Parser, Result,
    ascii::{multispace0, multispace1},
    combinator::{alt, preceded, separated, terminated},
    token::{take_till, take_while},
};

use codec::schema::{Author, Organization, Person};

/// Parse multiple authors separated by various delimiters
pub fn authors(input: &mut &str) -> Result<Vec<Author>> {
    separated(
        1..,
        author,
        (multispace0, alt(("&", "and", ",", ", &")), multispace0),
    )
    .parse_next(input)
}

/// Parse a single author in various formats
pub fn author(input: &mut &str) -> Result<Author> {
    alt((person_family_initials, organization)).parse_next(input)
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
    alt((person_family_initials,))
        .map(|author| match author {
            Author::Person(person) => person,
            _ => Person::default(),
        })
        .parse_next(input)
}

/// Parse person in "Family, F. M." format and deviations
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
fn person_family_initials(input: &mut &str) -> Result<Author> {
    (
        terminated(take_till(2.., |c: char| c == ','), ","),
        preceded(
            multispace0,
            separated(
                1..,
                take_while(1.., |c: char| (c.is_alphabetic() || c == '.') && c != ' '),
                multispace1,
            ),
        ),
    )
        .map(|(family_name, given_names): (&str, Vec<&str>)| {
            Author::Person(Person {
                family_names: Some(vec![family_name.to_string()]),
                given_names: Some(given_names.into_iter().map(String::from).collect()),
                ..Default::default()
            })
        })
        .parse_next(input)
}

/// Parse an organization
///
/// Generally, used as a fallback if the string does not match expected format
/// for a [`Person`].
fn organization(input: &mut &str) -> Result<Author> {
    take_while(2.., |c: char| !c.is_ascii_punctuation())
        .map(|name: &str| {
            Author::Organization(Organization {
                name: Some(name.to_string()),
                ..Default::default()
            })
        })
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use common_dev::pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_authors() -> Result<()> {
        // Single person
        let items = authors(&mut "Author, A. B.")?;
        assert_eq!(items.len(), 1);

        // Two people with ampersand
        let items = authors(&mut "Author, A. B., & Author, B. C.")?;
        assert_eq!(items.len(), 2);

        Ok(())
    }
    // In the following tests we parse using `authors` as a test of differentiating between different
    // authors types and that the sub-parsers to not conflict

    #[test]
    fn test_person_family_initials() -> Result<()> {
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
            assert_eq!(given_names, Some(vec!["J".to_string(), "A.".to_string()]));
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
            assert_eq!(family_names, Some(vec!["One Two".to_string()]));
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
            assert_eq!(given_names, Some(vec!["R".to_string()]));
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
            assert_eq!(family_names, Some(vec!["Van Der Berg".to_string()]));
            assert_eq!(given_names, Some(vec!["P.".to_string(), "Q.".to_string()]));
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
