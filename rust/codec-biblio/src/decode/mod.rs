use hayagriva::io::{from_biblatex_str, from_yaml_str};
use winnow::Parser;

use codec::{
    common::{
        eyre::{Result, bail},
        itertools::Itertools,
    },
    schema::{Inline, Reference, shortcuts::t},
};

mod acs;
mod apa;
mod apj;
mod chicago;
mod citations;
mod fallback;
mod ieee;
mod lncs;
mod mla;
mod parts;
mod reference;
mod references;
mod vancouver;

use crate::{
    conversion::entry_to_reference,
    decode::{citations::author_year_and_text, references::reference},
};

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

/// Decode plain text into a Stencila [`Reference`] node
pub fn text_to_reference(text: &str) -> Reference {
    reference(text)
}

/// Decode plain text into a vector of Stencila [`Reference`] nodes
pub fn text_to_references(text: &str) -> Vec<Reference> {
    text.split("\n\n")
        .map(|text: &str| reference(text))
        .collect()
}

/// Decode plain text into a vector of author-year [Inline::Citation] or
/// [Inline::CitationGroup] and interspersed [Inline::Text]
pub fn text_with_author_year_citations(text: &str) -> Vec<Inline> {
    match author_year_and_text.parse(text) {
        Ok(result) => result,
        Err(_) => vec![t(text)],
    }
}
