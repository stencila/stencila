use crate::{SectionType, prelude::*};

impl SectionType {
    pub fn from_text(text: &str) -> Result<Self> {
        use SectionType::*;

        let lower = text.to_lowercase().split_whitespace().join(" ");
        let trimmed = lower.trim();

        if trimmed.starts_with("appendix") {
            return Ok(Appendix);
        }

        Ok(match trimmed {
            // --- Front matter ---
            "abstract" => Abstract,

            "summary" | "author summary" | "executive summary" => Summary,

            "nontechnical summary"
            | "non-technical summary"
            | "plain language summary"
            | "lay summary"
            | "significance statement"
            | "public summary" => NonTechnicalSummary,

            "introduction" | "intro" => Introduction,

            "background"
            | "theoretical background"
            | "theoretical foundation"
            | "motivation and background" => Background,

            "related work" | "prior work" | "literature review" | "state of the art" => RelatedWork,

            // --- Methods & setup ---
            "materials" | "materials and methods" => Materials,

            "methods" | "methodology" | "experimental methods" | "methods and materials" => Methods,

            "experimental design"
            | "experimental setup"
            | "experimental set-up"
            | "experimental procedure"
            | "experimental procedures"
            | "design and methods"
            | "study design"
            | "experimental apparatus" => ExperimentalDesign,

            "statistical analysis"
            | "statistics"
            | "statistical methods"
            | "data analysis"
            | "analytical methods"
            | "power analysis"
            | "power calculation"
            | "sample size calculation" => StatisticalAnalysis,

            "cases" | "case report" | "case reports" | "case series" => Cases,

            // --- Results & interpretation ---
            "results" | "findings" => Results,

            "discussion" | "analysis" => Discussion,

            "limitations" | "study limitations" | "threats to validity" => Limitations,

            "conclusions" | "conclusion" | "concluding remarks" => Conclusions,

            "future work" | "outlook" | "perspectives" | "future directions" | "next steps" => {
                FutureWork
            }

            // --- References & post-text ---
            "references"
            | "bibliography"
            | "works cited"
            | "literature cited"
            | "citations"
            | "sources"
            | "reference list"
            | "further reading"
            | "additional sources"
            | "for further information" => References,

            "acknowledgements" | "acknowledgments" => Acknowledgements,

            "funding"
            | "funding statement"
            | "financial support"
            | "sources of funding"
            | "funding information" => Funding,

            "competing interests"
            | "competing interest"
            | "conflicts of interest"
            | "conflict of interest"
            | "declaration of interest"
            | "declarations of interest"
            | "declaration of interests"
            | "conflict of interests" => CompetingInterests,

            "ethics"
            | "ethics statement"
            | "ethical approval"
            | "ethics approval"
            | "irb statement"
            | "institutional review board statement"
            | "animal ethics"
            | "human subjects" => Ethics,

            "consent statements"
            | "informed consent"
            | "consent"
            | "consent statement"
            | "consent to participate"
            | "consent for publication"
            | "patient consent" => ConsentStatements,

            "data availability"
            | "availability of data"
            | "availability of data and materials"
            | "data and materials availability"
            | "data availability statement" => DataAvailability,

            "code availability"
            | "software availability"
            | "code and data availability"
            | "availability of code"
            | "analysis code"
            | "computational notebooks"
            | "notebooks"
            | "jupyter notebooks"
            | "colab notebooks" => CodeAvailability,

            "reproducibility"
            | "reproducible research"
            | "transparency statement"
            | "open science statement"
            | "availability and reproducibility" => Reproducibility,

            // --- Additional scholarly sections ---
            "author contributions"
            | "contributions"
            | "contributors"
            | "contribution statement" => AuthorContributions,

            "abbreviations" | "list of abbreviations" => Abbreviations,

            "nomenclature" | "glossary" | "list of symbols" | "list of notation" | "notation" => {
                Nomenclature
            }
            "preregistration" | "pre-registration" | "registered report" | "study registration"
            | "trial registration" | "registration" => Preregistration,

            "highlights" | "key points" | "key findings" | "key messages" => Highlights,

            // --- Back matter / structural ---
            "supplementary materials"
            | "supplementary material"
            | "supporting information"
            | "supporting material"
            | "supplementary data"
            | "supplementary files" => SupplementaryMaterials,

            "main" | "main text" => Main,
            "header" => Header,
            "footer" => Footer,
            "iteration" => Iteration,

            _ => bail!("Unrecognized section type: {text}"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_text() -> Result<()> {
        use SectionType::*;

        let f = SectionType::from_text;

        assert_eq!(f("Abstract")?, Abstract);
        assert_eq!(f("ABSTRACT")?, Abstract);
        assert_eq!(f("abstract")?, Abstract);

        assert_eq!(f("Summary")?, Summary);
        assert_eq!(f("Author summary")?, Summary);
        assert_eq!(f("executive   summary")?, Summary);

        assert_eq!(f("Introduction")?, Introduction);
        assert_eq!(f("intro")?, Introduction);

        assert_eq!(f("Methods")?, Methods);
        assert_eq!(f("Methodology")?, Methods);
        assert_eq!(f("Experimental Methods")?, Methods);
        assert_eq!(f("methods   and materials")?, Methods);

        assert_eq!(f("Materials")?, Materials);
        assert_eq!(f("materials and\tmethods")?, Materials);

        assert_eq!(f("Results")?, Results);
        assert_eq!(f("findings")?, Results);

        assert_eq!(f("Discussion")?, Discussion);
        assert_eq!(f("analysis")?, Discussion);

        assert_eq!(f("Conclusions")?, Conclusions);
        assert_eq!(f("conclusion")?, Conclusions);
        assert_eq!(f("Concluding   remarks")?, Conclusions);

        assert_eq!(f("Acknowledgements")?, Acknowledgements);

        assert_eq!(f("Supplementary Materials")?, SupplementaryMaterials);
        assert_eq!(f("supplementary material")?, SupplementaryMaterials);
        assert_eq!(f("supporting information")?, SupplementaryMaterials);

        // Test non-matching cases
        assert!(f("Random Section").is_err());
        assert!(f("Custom Heading").is_err());

        Ok(())
    }
}
