use hayagriva::io::{from_biblatex_str, from_yaml_str};

use codec::{
    common::{
        eyre::{Result, bail},
        itertools::Itertools,
    },
    schema::Reference,
};

use crate::conversion::entry_to_reference;

/// Decode Hayagriva YAML to a set of Stencila [`Reference`] nodes
pub fn yaml(yaml: &str) -> Result<Vec<Reference>> {
    let library = from_yaml_str(yaml)?;

    let references = library.iter().map(entry_to_reference).try_collect()?;

    Ok(references)
}

/// Decode BibLaTeX to a set of Stencila [`Reference`] nodes
pub fn bibtex(bibtex: &str) -> Result<Vec<Reference>> {
    let library = match from_biblatex_str(bibtex) {
        Ok(lib) => lib,
        Err(errors) => {
            let error_msg = errors
                .iter()
                .map(|error| format!("{error:?}"))
                .collect::<Vec<_>>()
                .join(", ");
            bail!("Failed to parse BibTeX: {error_msg}");
        }
    };

    let references = library.iter().map(entry_to_reference).try_collect()?;

    Ok(references)
}

/// Decode Plain text into a set of Stencila [`Reference`] nodes
pub fn text(_text: &str) -> Result<Vec<Reference>> {
    // TODO: Implement a winnow parser for plain text citations
    todo!()
}
