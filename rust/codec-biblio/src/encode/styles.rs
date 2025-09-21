use cached::proc_macro::cached;
use hayagriva::{
    archive::ArchivedStyle,
    citationberg::{IndependentStyle, Style},
};
use reqwest::StatusCode;
use tokio::fs::{read_to_string, write};

use stencila_codec::eyre::{Result, bail};
use stencila_dirs::{DirType, get_app_dir};

/// Get a citation style by name
pub(super) async fn get_style(name: &str) -> Result<IndependentStyle> {
    let normalized_name = normalize_name(name).to_string();

    let style = match get_archived(normalized_name.clone()) {
        Some(style) => style,
        _ => get_csl(normalized_name).await?,
    };

    match style {
        Style::Independent(style) => Ok(style),
        _ => bail!("Only independent citation styles are supported"),
    }
}

/// Get a Hayagriva's archived style
///
/// Memoized to avoid repeated deserialization of the CBOR representation of the
/// style.
#[cached]
fn get_archived(name: String) -> Option<Style> {
    ArchivedStyle::by_name(&name).map(|style| style.get())
}

/// Get a style by downloading the CSL XML from GitHub and constructing a new
/// style
///
/// CSL files are cached in the "csl" subdirectory of the Stencila app cache.
///
/// Memoized to avoid repeated deserialization of the CBOR representation of the
/// style.
#[tracing::instrument]
#[cached(result = true)]
async fn get_csl(name: String) -> Result<Style> {
    let cache_dir = get_app_dir(DirType::Csl, true)?;
    let cache_path = cache_dir.join(format!("{name}.csl"));

    // Attempt to read from the cache, failing if no cached file, or if file can not
    // be parsed as style
    if let Some(style) = read_to_string(&cache_path)
        .await
        .ok()
        .and_then(|xml| Style::from_xml(xml.as_str()).ok())
    {
        return Ok(style);
    }

    // Fetch the CSL
    let url = format!(
        "https://raw.githubusercontent.com/citation-style-language/styles/refs/heads/master/{name}.csl"
    );
    let response = reqwest::get(url).await?;

    // Return early on error
    if response.status() == StatusCode::NOT_FOUND {
        bail!("Unknown citation style name `{name}`");
    }
    response.error_for_status_ref()?;

    // Parse the CSL into a style
    let xml = response.text().await?;
    let style = Style::from_xml(&xml)?;

    // Cache the CSL is the parsing was successful
    if let Err(error) = write(&cache_path, xml).await {
        tracing::debug!("Unable to write to {}: {error}", cache_path.display());
    }

    Ok(style)
}

/// Normalize a name or abbreviation to the full kebab-cased name listed at
/// https://github.com/citation-style-language/styles
///
/// Allows users to use an abbreviation or other aliases for a citation style.
fn normalize_name(name: &str) -> &str {
    match name.to_lowercase().as_str() {
        // AMA - American Medical Association variants
        "ama" | "ama10" | "ama-10" => "american-medical-association",
        "ama-10th" => "american-medical-association-10th-edition",
        "ama-alphabetical" | "ama-alpha" => "american-medical-association-alphabetical",
        "ama-brackets" => "american-medical-association-brackets",
        "ama-no-et-al" => "american-medical-association-no-et-al",
        "ama-no-url" => "american-medical-association-no-url",

        // APA - American Psychological Association variants
        "apa" | "apa7" | "apa-7" => "apa",
        "apa5" | "apa-5" => "apa-5th-edition",
        "apa6" | "apa-6" => "apa-6th-edition",
        "apa-no-doi" => "apa-no-doi-no-issue",
        "apa-annotated" => "apa-annotated-bibliography",

        // Chicago Style variants
        "chicago" | "chicago-author-date" | "cms-author-date" => "chicago-author-date",
        "chicago16" | "chicago-16" => "chicago-author-date-16th-edition",
        "chicago17" | "chicago-17" => "chicago-author-date-17th-edition",
        "chicago-notes" | "chicago-footnotes" | "cms-notes" => "chicago-notes-bibliography",
        "chicago-notes16" | "chicago-notes-16" => "chicago-notes-bibliography-16th-edition",
        "chicago-notes17" | "chicago-notes-17" => "chicago-notes-bibliography-17th-edition",

        // MLA - Modern Language Association
        "mla" | "mla9" | "mla-9" => "modern-language-association",
        "mla8" | "mla-8" => "modern-language-association-8th-edition",
        "mla7" | "mla-7" => "modern-language-association-7th-edition",
        "mla6" | "mla-6" => "modern-language-association-6th-edition-with-url",
        "mla-note" => "modern-language-association-note",
        "mla-url" => "modern-language-association-with-url",

        // Harvard variants
        "harvard" => "harvard-cite-them-right",
        "harvard1" => "harvard1",
        "harvard2" => "harvard2",
        "harvard3" => "harvard3",
        "harvard-anglia" => "harvard-anglia-ruskin-university",
        "harvard-leeds" => "harvard-leeds-beckett-university",
        "harvard-university-west-london" => "harvard-university-of-west-london",

        // IEEE - Institute of Electrical and Electronics Engineers
        "ieee" => "ieee",
        "ieee-cite" => "ieee-with-url",
        "ieee-access" => "ieee-access",

        // Nature variants
        "nature" => "nature",
        "nature-no-et-al" => "nature-no-et-al",
        "nature-no-superscript" => "nature-no-superscript",
        "nature-no-title" => "nature-no-title",

        // Science
        "science" => "science",
        "science-without-titles" => "science-without-titles",

        // Cell
        "cell" => "cell",
        "cell-numeric" => "cell-numeric",
        "cell-research" => "cell-research",

        // Vancouver style
        "vancouver" => "vancouver",
        "vancouver-author-date" => "vancouver-author-date",
        "vancouver-brackets" => "vancouver-brackets",
        "vancouver-superscript" => "vancouver-superscript",
        "nlm" | "national-library-of-medicine" => "national-library-of-medicine",

        // Elsevier variants
        "elsevier" => "elsevier-harvard",
        "elsevier-harvard" => "elsevier-harvard",
        "elsevier-harvard2" => "elsevier-harvard2",
        "elsevier-vancouver" => "elsevier-vancouver",
        "elsevier-with-titles" => "elsevier-with-titles",

        // BMJ - British Medical Journal
        "bmj" => "bmj",

        // Lancet
        "lancet" | "the-lancet" => "the-lancet",
        "lancet-neurology" => "the-lancet-neurology",
        "lancet-oncology" => "the-lancet-oncology",

        // JAMA - Journal of the American Medical Association
        "jama" => "journal-of-the-american-medical-association",

        // Oxford variants
        "oxford" => "oxford-university-press-humsoc",
        "oxford-note" => "oxford-university-press-note",
        "oxford-numeric" => "oxford-university-press-numeric",

        // Cambridge variants
        "cambridge" => "cambridge-university-press-author-date",
        "cambridge-note" => "cambridge-university-press-note",
        "cambridge-numeric" => "cambridge-university-press-numeric",

        // ACM - Association for Computing Machinery
        "acm" => "association-for-computing-machinery",
        "acm-sig" => "acm-sig-proceedings",
        "acm-sigchi" => "acm-sigchi-proceedings",
        "acm-siggraph" => "acm-siggraph",

        // ACS - American Chemical Society
        "acs" => "american-chemical-society",
        "acs-with-titles" => "american-chemical-society-with-titles",

        // AIP - American Institute of Physics
        "aip" => "american-institute-of-physics",
        "aip-advances" => "aip-advances",

        // APS - American Physical Society
        "aps" => "american-physics-society",
        "aps-with-titles" => "american-physics-society-without-titles",

        // ASA - American Sociological Association
        "asa" => "american-sociological-association",

        // APSA - American Political Science Association
        "apsa" => "american-political-science-association",

        // CSE - Council of Science Editors
        "cse" => "council-of-science-editors",
        "cse8" | "cse-8" => "council-of-science-editors-8th-edition",
        "cse-author-date" => "council-of-science-editors-author-date",

        // ISO 690
        "iso690" | "iso-690" => "iso690-author-date-en",
        "iso690-numeric" => "iso690-numeric-en",
        "iso690-note" => "iso690-note-en",

        // Other common journals/publishers
        "plos" | "plos-one" => "plos-one",
        "pnas" => "proceedings-of-the-national-academy-of-sciences",
        "rsc" | "royal-society-chemistry" => "royal-society-of-chemistry",
        "springer" => "springer-basic-author-date",
        "springer-numeric" => "springer-basic-brackets",
        "springer-vancouver" => "springer-vancouver",
        "wiley" => "wiley",
        "taylor-francis" | "taylor-and-francis" => "taylor-and-francis-chicago-author-date",
        "sage" => "sage-harvard",
        "sage-vancouver" => "sage-vancouver",

        // GOST (Russian/Eastern European)
        "gost" => "gost-r-7-0-5-2008",
        "gost-numeric" => "gost-r-7-0-5-2008-numeric",

        // Turabian
        "turabian" | "turabian-author-date" => "turabian-author-date",
        "turabian8" | "turabian-8" => "turabian-author-date-8th-edition",
        "turabian-notes" | "turabian-fullnote" => "turabian-fullnote-bibliography",
        "turabian-notes8" | "turabian-fullnote-8" => "turabian-fullnote-bibliography-8th-edition",

        // Bluebook legal citation
        "bluebook" => "bluebook-law-review",
        "bluebook-inline" => "bluebook-inline",

        // OSCOLA legal citation
        "oscola" => "oscola",
        "oscola-no-ibid" => "oscola-no-ibid",

        // Australian Guide to Legal Citation
        "aglc" => "australian-guide-to-legal-citation",
        "aglc3" | "aglc-3" => "australian-guide-to-legal-citation-3rd-edition",

        // Frontiers journals
        "frontiers" => "frontiers",

        // BibTeX
        "bibtex" => "bibtex",

        // Annual Reviews
        "annual-reviews" => "annual-reviews",

        // Biomed Central
        "bmc" | "biomed-central" => "biomed-central",

        // Default (return the input as-is)
        _ => name,
    }
}
