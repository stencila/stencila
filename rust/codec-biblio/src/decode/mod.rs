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
    decode::{
        citations::{
            author_year, author_year_and_text, bracketed_numeric, bracketed_numeric_and_text,
            parenthetic_numeric, parenthetic_numeric_and_text, superscripted_numeric,
            superscripted_numeric_and_text,
        },
        references::reference,
    },
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

/// Parse author-year citations like "Smith (2020)" and "(Smith & Jones, 2021)" in text
pub fn text_with_author_year_citations(text: &str) -> Vec<Inline> {
    match author_year_and_text.parse(text) {
        Ok(result) => result,
        Err(_) => vec![t(text)],
    }
}

/// Parse an author-year citations like "Smith (2020)" or "(Smith & Jones, 2021)" from text
pub fn author_year_citation(text: &str) -> Option<Inline> {
    author_year.parse(text).ok()
}

/// Parse square bracket numeric citations like "[1]", "[1-3]", and "[1,2,3]" in text
pub fn text_with_bracketed_numeric_citations(text: &str) -> Vec<Inline> {
    match bracketed_numeric_and_text.parse(text) {
        Ok(result) => result,
        Err(_) => vec![t(text)],
    }
}

/// Parse a square bracket numeric citation like "[1]", "[1-3]", or "[1,2,3]" from text
pub fn bracketed_numeric_citation(text: &str) -> Option<Inline> {
    bracketed_numeric.parse(text).ok()
}

/// Parse parenthetic numeric citations like "(1)", "(1-3)", and "(1,2,3)" in text
pub fn text_with_parenthetic_numeric_citations(text: &str) -> Vec<Inline> {
    match parenthetic_numeric_and_text.parse(text) {
        Ok(result) => result,
        Err(_) => vec![t(text)],
    }
}

/// Parse a parenthetic numeric citation like "(1)", "(1-3)", or "(1,2,3)" from text
pub fn parenthetic_numeric_citation(text: &str) -> Option<Inline> {
    parenthetic_numeric.parse(text).ok()
}

/// Parse LaTeX superscript citations like "{}^{1}", "{}^{1-3}", and "{}^{1,2,3}" in math
pub fn text_with_superscripted_numeric_citations(text: &str) -> Vec<Inline> {
    match superscripted_numeric_and_text.parse(text) {
        Ok(result) => result,
        Err(_) => vec![t(text)],
    }
}

/// Parse a superscripted numeric citation like "{}^{1}", "{}^{1-3}", and "{}^{1,2,3}" from math
pub fn superscripted_numeric_citation(text: &str) -> Option<Inline> {
    superscripted_numeric.parse(text).ok()
}
