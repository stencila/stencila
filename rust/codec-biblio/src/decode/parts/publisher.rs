use winnow::{
    Parser, Result,
    ascii::{multispace0, multispace1},
    combinator::{alt, opt, preceded, separated, terminated},
    token::take_while,
};

use codec::schema::{
    Organization, OrganizationOptions, PersonOrOrganization, PostalAddressOrString,
};

/// Parse place and publisher in Vancouver and IEEE format
///
/// Parses "Place: Publisher" or just "Publisher" format.
/// Avoids matching against URLs and DOIs
pub fn place_publisher(input: &mut &str) -> Result<PersonOrOrganization> {
    (
        opt(terminated(
            take_while(2.., |c: char| c != ':' && c != '.'),
            ":",
        )),
        take_while(2.., |c: char| c != '.' && c != ',' && c != ';' && c != ')'),
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
                .and_then(|place| place.split_whitespace().next_back())
            {
                // Place should contain some alphabetic chars
                if !place.chars().any(|c: char| c.is_alphabetic()) {
                    return false;
                }

                // Last word in place should not be one of these...
                if matches!(
                    place.to_lowercase().as_str(),
                    "http" | "https" | "url" | "doi"
                ) {
                    return false;
                }
            }

            // Name should contain some alphabetic chars
            if !name.chars().any(|c: char| c.is_alphabetic()) {
                return false;
            }

            // Name should not start with two slashes
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

/// Parse publisher and place in comma separated format
///
/// e.g. Springer, Berlin
pub fn publisher_place(input: &mut &str) -> Result<PersonOrOrganization> {
    (
        take_while(2.., |c: char| c != ',' && c != '.' && !c.is_numeric()),
        opt(preceded(
            opt((multispace0, alt((",", ":")), multispace0)),
            separated(
                1..,
                take_while(1.., |c: char| !c.is_whitespace()).verify(|part: &str| {
                    part.chars()
                        .next()
                        .map(|c| c.is_alphabetic())
                        .unwrap_or_default()
                }),
                multispace1,
            ),
        )),
    )
        .verify(|(name, _place): &(&str, Option<Vec<&str>>)| {
            !(name.ends_with("http")
                || name.ends_with("https")
                || name.ends_with("doi")
                || name.ends_with("url"))
        })
        .map(|(name, place): (&str, Option<Vec<&str>>)| {
            let address = place
                .map(|place| {
                    place
                        .join(" ")
                        .trim()
                        .trim_end_matches([',', '.'])
                        .trim()
                        .to_string()
                })
                .filter(|place| !place.is_empty())
                .map(PostalAddressOrString::String);

            PersonOrOrganization::Organization(Organization {
                name: Some(name.to_string()),
                options: Box::new(OrganizationOptions {
                    address,
                    ..Default::default()
                }),
                ..Default::default()
            })
        })
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_place_publisher() -> Result<()> {
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
        assert!(place_publisher(&mut "DOI: 10.1234/xyz").is_err());
        assert!(place_publisher(&mut "Some content before doi:10.1234/xyz").is_err());

        // Test volume:pages avoidance
        assert!(place_publisher(&mut "10:123-456").is_err());
        assert!(place_publisher(&mut "10 (4) :123 - 456").is_err());

        Ok(())
    }

    #[test]
    fn test_publisher_place() -> Result<()> {
        // Place and publisher with colon
        let publisher = publisher_place(&mut "Tech Press, New York, USA")?;
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
        let publisher = publisher_place(&mut "Academic Press")?;
        if let PersonOrOrganization::Organization(org) = publisher {
            assert_eq!(org.name, Some("Academic Press".to_string()));
            assert_eq!(org.options.address, None);
        } else {
            unreachable!("expected organization")
        }

        // With extra whitespace
        let publisher = publisher_place(&mut "University Press,  Boston")?;
        if let PersonOrOrganization::Organization(org) = publisher {
            assert_eq!(org.name, Some("University Press".to_string()));
            assert_eq!(
                org.options.address,
                Some(PostalAddressOrString::String("Boston".to_string()))
            );
        } else {
            unreachable!("expected organization")
        }

        Ok(())
    }
}
