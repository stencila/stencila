// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The type of a `Section`.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, Copy, strum::EnumString, Eq, PartialOrd, Ord, Hash, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub enum SectionType {
    /// A concise summary of the article’s purpose, methods, key results, and conclusions.
    #[default]
    Abstract,

    /// A short, often technical summary such as an author or executive summary.
    Summary,

    /// Non-technical summary written for general audiences, explaining the research’s significance and real-world implications without specialized terminology.
    NonTechnicalSummary,

    /// Bullet-point summary of key findings or contributions (also called Key Points in some journals).
    Highlights,

    /// Establishes context, prior work, and the research question or objectives.
    Introduction,

    /// Extended context and theoretical foundation, providing deeper background than typically found in the Introduction.
    Background,

    /// Survey or comparison of closely related prior work (common in CS/ML).
    RelatedWork,

    /// Details on materials, datasets, organisms, hardware, or reagents used.
    Materials,

    /// Procedures, algorithms, and analysis methods sufficient for reproducibility.
    Methods,

    /// Detailed description of experimental design, including apparatus, parameters, conditions, and protocols.
    ExperimentalDesign,

    /// Detailed description of statistical methods, including model specifications, power/sample-size calculations, and analysis decisions.
    StatisticalAnalysis,

    /// Case reports or case-series descriptions, often in clinical research.
    Cases,

    /// Objective presentation of findings without extensive interpretation.
    Results,

    /// Interpretation of results, implications, and relation to prior work.
    Discussion,

    /// Known constraints or threats to validity affecting interpretation or generalizability.
    Limitations,

    /// Final takeaways, recommendations, and wrap-up of the study’s contribution.
    Conclusions,

    /// Suggested directions, next steps, or open problems for subsequent research.
    FutureWork,

    /// Bibliographic list of works cited in the document.
    References,

    /// Recognition of non-author contributions such as assistance, facilities, or feedback.
    Acknowledgements,

    /// Sources of financial support, grant numbers, and funding acknowledgments.
    Funding,

    /// Declarations of conflicts or competing interests by the authors.
    CompetingInterests,

    /// Ethical approvals, IRB/IEC statements, animal welfare, or ethical compliance.
    Ethics,

    /// Statements confirming informed consent was obtained from participants, patients, or for publication of identifying information.
    ConsentStatements,

    /// Statement describing where and how the underlying data can be accessed.
    DataAvailability,

    /// Statement describing where and how to access analysis code, software, or computational notebooks used in the research.
    CodeAvailability,

    /// Comprehensive statement on reproducibility and openness, covering availability of data, code, materials, and protocols.
    Reproducibility,

    /// Specific roles and contributions of each author (e.g., CRediT taxonomy).
    AuthorContributions,

    /// List of abbreviations used in the document and their definitions.
    Abbreviations,

    /// Glossary, symbols, or notation used throughout the document.
    Nomenclature,

    /// Statement or link to study preregistration, including hypotheses and analysis plans registered before data collection.
    Preregistration,

    /// Additional figures, tables, data, or methods that support the main text.
    SupplementaryMaterials,

    /// Ancillary material such as derivations, proofs, or extended details.
    Appendix,

    /// The main body of content when a document is not subdivided into standard sections.
    Main,

    /// Front matter at the top of a page or section (e.g., running titles).
    Header,

    /// Foot matter at the bottom of a page or section (e.g., footers, disclaimers).
    Footer,

    /// A section representing an iteration of a `ForBlock`.
    Iteration,
}
