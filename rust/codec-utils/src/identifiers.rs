//! Normalization of scholarly identifiers and names shared by the codecs for
//! formats of scholarly articles (e.g. JATS and TEI)

/// Remove any resolver prefix from a DOI
pub fn strip_doi_prefix(doi: &str) -> String {
    doi.trim()
        .trim_start_matches("https://doi.org/")
        .trim_start_matches("http://doi.org/")
        .trim_start_matches("https://dx.doi.org/")
        .trim_start_matches("http://dx.doi.org/")
        .to_string()
}

/// Remove any resolver prefix from a ROR id
pub fn strip_ror_prefix(ror: &str) -> String {
    ror.trim()
        .trim_start_matches("https://ror.org/")
        .trim_start_matches("http://ror.org/")
        .trim_end_matches('/')
        .to_string()
}

/// Split a string of author given names
///
/// In addition to splitting by whitespace, removes any trailing dot
/// (often on initials) and splits all caps (initials without separator).
pub fn split_given_names(name: &str) -> Vec<String> {
    name.split_ascii_whitespace()
        .flat_map(|n| {
            let trimmed = n.trim_end_matches('.');
            if trimmed.chars().all(|c| c.is_ascii_uppercase()) && trimmed.chars().count() > 1 {
                trimmed
                    .chars()
                    .map(|c| c.to_string())
                    .collect::<Vec<String>>()
            } else {
                vec![trimmed.to_string()]
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doi_prefixes() {
        for doi in [
            "10.1/x",
            " 10.1/x ",
            "https://doi.org/10.1/x",
            "http://doi.org/10.1/x",
            "https://dx.doi.org/10.1/x",
            "http://dx.doi.org/10.1/x",
        ] {
            assert_eq!(strip_doi_prefix(doi), "10.1/x");
        }
    }

    #[test]
    fn ror_prefixes() {
        for ror in [
            "02mhbdp94",
            "https://ror.org/02mhbdp94",
            "http://ror.org/02mhbdp94",
            "https://ror.org/02mhbdp94/",
        ] {
            assert_eq!(strip_ror_prefix(ror), "02mhbdp94");
        }
    }

    #[test]
    fn given_names() {
        assert_eq!(split_given_names("Jane A."), vec!["Jane", "A"]);
        assert_eq!(split_given_names("JA"), vec!["J", "A"]);
        assert_eq!(split_given_names("Jean-Luc"), vec!["Jean-Luc"]);
    }
}
