use stencila_codec::stencila_schema::{Author, Date};

use crate::decode::parts::authors::extract_name;

/// Generate a consistent reference ID from authors and year
///
/// Creates standardized identifiers for references based on their authors and
/// publication year. This ensures that references parsed from bibliography text
/// get the same ids that would be generated from in-text author-year citations.
pub fn generate_id(authors: &[Author], date: &Option<(Date, Option<String>)>) -> String {
    let authors = if authors.is_empty() {
        "unknown".to_string()
    } else if authors.len() == 1 {
        extract_name(&authors[0])
    } else if authors.len() == 2 {
        format!(
            "{}-and-{}",
            extract_name(&authors[0]),
            extract_name(&authors[1])
        )
    } else {
        format!("{}-et-al", extract_name(&authors[0]))
    };

    let year = date
        .as_ref()
        .and_then(|(date, ..)| date.year())
        .map_or_else(|| "unknown".to_string(), |year| year.to_string());

    let suffix = date
        .as_ref()
        .and_then(|(.., suffix)| suffix.as_deref())
        .unwrap_or("");

    format!("{authors}-{year}{suffix}",)
}

#[cfg(test)]
mod tests {
    use stencila_codec::stencila_schema::{Organization, Person};

    use super::*;

    #[test]
    fn test_generate_id() {
        // Single author with year
        let authors = vec![Author::Person(Person {
            family_names: Some(vec!["Smith".to_string()]),
            given_names: Some(vec!["John".to_string()]),
            ..Default::default()
        })];
        let date = Some((
            Date {
                value: "2023".to_string(),
                ..Default::default()
            },
            None,
        ));
        assert_eq!(generate_id(&authors, &date), "smith-2023");

        // Two authors
        let authors = vec![
            Author::Person(Person {
                family_names: Some(vec!["Smith".to_string()]),
                ..Default::default()
            }),
            Author::Person(Person {
                family_names: Some(vec!["Jones".to_string()]),
                ..Default::default()
            }),
        ];
        let date = Some((
            Date {
                value: "2022".to_string(),
                ..Default::default()
            },
            None,
        ));
        assert_eq!(generate_id(&authors, &date), "smith-and-jones-2022");

        // Multiple authors (et al case)
        let authors = vec![
            Author::Person(Person {
                family_names: Some(vec!["Brown".to_string()]),
                ..Default::default()
            }),
            Author::Person(Person {
                family_names: Some(vec!["Wilson".to_string()]),
                ..Default::default()
            }),
            Author::Person(Person {
                family_names: Some(vec!["Davis".to_string()]),
                ..Default::default()
            }),
        ];
        let date = Some((
            Date {
                value: "2021".to_string(),
                ..Default::default()
            },
            None,
        ));
        assert_eq!(generate_id(&authors, &date), "brown-et-al-2021");

        // With year suffix
        let authors = vec![Author::Person(Person {
            family_names: Some(vec!["Taylor".to_string()]),
            ..Default::default()
        })];
        let date = Some((
            Date {
                value: "2020".to_string(),
                ..Default::default()
            },
            Some("a".to_string()),
        ));
        assert_eq!(generate_id(&authors, &date), "taylor-2020a");

        // Organization author
        let authors = vec![Author::Organization(Organization {
            name: Some("World Health Organization".to_string()),
            ..Default::default()
        })];
        let date = Some((
            Date {
                value: "2019".to_string(),
                ..Default::default()
            },
            None,
        ));
        assert_eq!(
            generate_id(&authors, &date),
            "world-health-organization-2019"
        );

        // No authors, no date
        let authors = vec![];
        let date = None;
        assert_eq!(generate_id(&authors, &date), "unknown-unknown");
    }
}
