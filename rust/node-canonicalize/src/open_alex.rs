use common::{eyre::Result, reqwest, serde::Deserialize};

#[derive(Deserialize)]
#[serde(crate = "common::serde")]
struct Author {
    id: String,
    orcid: Option<String>,
}

#[derive(Deserialize)]
#[serde(crate = "common::serde")]
struct AuthorsSearchResponse {
    results: Vec<Author>,
}

/// Get the ORCID for an author from OpenAlex based on names
/// 
/// This function should only be called as a fallback if an ORCID can
/// not be derived from authorship of a [`Reference`]`. That is because it
/// searches only by name and a such does not take advantage of the
/// OpenAlex's disambiguation.
/// 
/// See https://help.openalex.org/hc/en-us/articles/24347048891543-Author-disambiguation
pub(super) async fn orcid(
    family_names: &Option<Vec<String>>,
    given_names: &Option<Vec<String>>,
) -> Result<Option<String>> {
    let Some(family_names) = &family_names else {
        return Ok(None);
    };
    if family_names.is_empty() {
        return Ok(None);
    }

    let mut search = family_names.join(" ");
    if let Some(given_names) = &given_names {
        search = format!("{} {}", given_names.join(" "), search);
    };

    let response: AuthorsSearchResponse =
        reqwest::get(format!("https://api.openalex.org/authors?search={search}"))
            .await?
            .json()
            .await?;

    let Some(author) = response.results.first() else {
        return Ok(None);
    };

    // If author has an ORCID, return it (with URL prefix stripped)
    if let Some(orcid) = &author.orcid {
        let orcid = orcid
            .strip_prefix("https://orcid.org/")
            .unwrap_or(&orcid)
            .into();
        return Ok(Some(orcid));
    }

    // Generate a pseudo-ORCID from the OpenAlex ID
    // Uses 'A' as the first letter to indicate that it is a pseudo-ORCID based on OpenAlex ID
    // (and which OpenAlex author IDs have anyway)
    let int: u64 = author
        .id
        .strip_prefix("https://openalex.org/")
        .unwrap_or(&author.id)
        .trim_start_matches('A')
        .parse()?;
    let digits = format!("{:015}", int % 1_000_000_000_000_000);
    let pseudo_orcid = format!(
        "A{}-{}-{}-{}",
        &digits[0..3],
        &digits[3..7],
        &digits[7..11],
        &digits[11..15],
    );

    Ok(Some(pseudo_orcid))
}
