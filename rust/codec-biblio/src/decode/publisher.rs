use winnow::{
    Parser, Result,
    combinator::{opt, terminated},
    token::take_while,
};

use codec::schema::{
    Organization, OrganizationOptions, PersonOrOrganization, PostalAddressOrString,
};

/// Parse place and publisher in Vancouver and IEEE format
///
/// Parses "Place: Publisher" or just "Publisher" format.
/// Avoids matching against
pub fn place_publisher(input: &mut &str) -> Result<PersonOrOrganization> {
    (
        opt(terminated(
            take_while(2.., |c: char| c != ':' && c != '.'),
            ":",
        )),
        take_while(2.., |c: char| c != '.' && c != ',' && c != ';'),
    )
        .map(|(place, name): (Option<&str>, &str)| {
            (
                place.map(|place| place.trim().to_string()),
                name.trim().to_string(),
            )
        })
        .verify(|(place, name)| {
            if let Some(place) = place
                .as_ref()
                .and_then(|place| place.split_whitespace().rev().next())
            {
                if matches!(place, "http" | "https" | "url" | "doi") {
                    return false;
                }
            }
            !name.starts_with("//")
        })
        .map(|(place, name)| {
            PersonOrOrganization::Organization(Organization {
                name: Some(name),
                options: Box::new(OrganizationOptions {
                    address: place.map(PostalAddressOrString::String),
                    ..Default::default()
                }),
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
    fn test_publisher() -> Result<()> {
        // Place and publisher with colon
        let publisher = place_publisher(&mut "New York, USA:Tech Press")?;
        if let PersonOrOrganization::Organization(org) = publisher {
            assert_eq!(org.name, Some("Tech Press".to_string()));
            assert_eq!(
                org.options.address,
                Some(PostalAddressOrString::String("New York, USA".to_string()))
            );
        } else {
            unreachable!("expected organization")
        }

        // Just publisher (no place)
        let publisher = place_publisher(&mut "Academic Press")?;
        if let PersonOrOrganization::Organization(org) = publisher {
            assert_eq!(org.name, Some("Academic Press".to_string()));
            assert_eq!(org.options.address, None);
        } else {
            unreachable!("expected organization")
        }

        // With extra whitespace
        let publisher = place_publisher(&mut "  Boston  :  University Press  ")?;
        if let PersonOrOrganization::Organization(org) = publisher {
            assert_eq!(org.name, Some("University Press".to_string()));
            assert_eq!(
                org.options.address,
                Some(PostalAddressOrString::String("Boston".to_string()))
            );
        } else {
            unreachable!("expected organization")
        }

        // Test URL, DOI etc avoidance
        assert!(place_publisher(&mut "https://example.com").is_err());
        assert!(place_publisher(&mut "doi:10.1234/xyz").is_err());
        assert!(place_publisher(&mut "Some content before doi:10.1234/xyz").is_err());

        Ok(())
    }
}
