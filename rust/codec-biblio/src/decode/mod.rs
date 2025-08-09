use hayagriva::io::{from_biblatex_str, from_yaml_str};

use codec::{
    common::{
        eyre::{Result, bail},
        itertools::Itertools,
    },
    schema::Reference,
};

use crate::conversion::entry_to_reference;

mod apa;
mod authors;
mod chicago;
mod date;
mod doi;
mod ieee;
mod mla;
mod pages;
mod references;
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
pub fn text(mut text: &str) -> Result<Vec<Reference>> {
    Ok(references::references(&mut text).unwrap_or_default())
}
