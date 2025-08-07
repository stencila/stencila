//! Parsers that parse a Stencila [`Author`] from a string

use winnow::{
    Parser, Result,
    ascii::{multispace0, multispace1},
    combinator::{alt, preceded, separated, terminated},
    token::{take_till, take_while},
};

use codec::schema::{Author, Person};

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
    alt((person_family_initials,)).parse_next(input)
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

#[cfg(test)]
mod tests {
    use common_dev::pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_authors() -> Result<()> {
        // Mainly a test of separators. Use specific tests below for testing author variations
        let items = authors(&mut "Author, A. B.")?;
        assert_eq!(items.len(), 1);

        let items = authors(&mut "Author, A. B., & Author, B. C.")?;
        assert_eq!(items.len(), 2);

        Ok(())
    }

    #[test]
    fn test_person_family_initials() -> Result<()> {
        // Standard format with periods after initials
        if let Author::Person(Person {
            family_names,
            given_names,
            ..
        }) = person_family_initials(&mut "Smith, J. A.")?
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
        }) = person_family_initials(&mut "Smith, J A.")?
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
        }) = person_family_initials(&mut "One Two, John A.")?
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
        }) = person_family_initials(&mut "Johnson, M.")?
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
        }) = person_family_initials(&mut "Wilson, R")?
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
        }) = person_family_initials(&mut "Brown, A. B. C.")?
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
        }) = person_family_initials(&mut "Garcia, Maria J.")?
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
        }) = person_family_initials(&mut "Smith-Jones, K. L.")?
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
        }) = person_family_initials(&mut "Williams, Mary Elizabeth")?
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
        }) = person_family_initials(&mut "Van Der Berg, P. Q.")?
        {
            assert_eq!(family_names, Some(vec!["Van Der Berg".to_string()]));
            assert_eq!(given_names, Some(vec!["P.".to_string(), "Q.".to_string()]));
        } else {
            unreachable!("expected person")
        }

        Ok(())
    }
}
