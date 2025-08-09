use hayagriva::io::{from_biblatex_str, from_yaml_str};

use codec::{
    common::{
        eyre::{Result, bail},
        itertools::Itertools,
    },
    schema::Reference,
};

use crate::{conversion::entry_to_reference, decode::references::reference};

mod apa;
mod authors;
mod chicago;
mod date;
mod doi;
mod fallback;
mod ieee;
mod mla;
mod pages;
mod preprints;
mod publisher;
mod references;
mod terminator;
mod url;
mod vancouver;

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

/// Decode plain text into a set of Stencila [`Reference`] nodes
pub fn text(text: &str) -> Result<Vec<Reference>> {
    Ok(text
        .split("\n\n")
        .filter_map(|mut text| reference(&mut text).ok())
        .collect())
}
