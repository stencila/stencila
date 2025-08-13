//! Parsers that parse a Stencila [`Reference`] from a string

use winnow::{
    Parser, Result,
    ascii::{digit1, multispace0, multispace1},
    combinator::{alt, opt, preceded},
};

use codec::{common::tracing, schema::Reference};

use crate::decode::{acs, apa, apj, chicago, fallback::fallback, ieee, lncs, mla, vancouver};

/// Parse a Stencila [`Reference`] from a string
///
/// The aim of this function is to extract as much bibliographic information as
/// possible from plain text. It attempts to parse reference using several
/// popular formats falling back to just trying to extract a DOI (since that
/// is the most valuable bibliographic information).
pub fn reference(input: &mut &str) -> Result<Reference> {
    let mut parser = preceded(
        // Ignore any numbering prefix for the reference
        opt((
            multispace0,
            alt((
                ("[", digit1, "]").take(),
                (digit1, ".").take(),
                (digit1, multispace1).take(),
            )),
            multispace0,
        )),
        // Order parsers is roughly from most strict to most flexible with some
        // re-ordering based on making the following tests pass. It is necessary
        // to list individual types like this so that, for example, an APA
        // article is not parsed prematurely as a Vancouver book.
        alt((
            // Chapter or conference paper
            alt((
                lncs::conference,
                vancouver::chapter,
                acs::chapter,
                ieee::chapter,
                apa::chapter,
                chicago::chapter,
                mla::chapter,
                lncs::chapter,
                apj::chapter,
            )),
            // Article
            alt((
                vancouver::article,
                acs::article,
                ieee::article,
                apa::article,
                chicago::article,
                mla::article,
                lncs::article,
                apj::article,
            )),
            // Web
            alt((
                vancouver::web,
                ieee::web,
                apa::web,
                chicago::web,
                mla::web,
                lncs::web,
                apj::web,
            )),
            // Book
            alt((
                apa::book,
                vancouver::book,
                acs::article,
                ieee::book,
                chicago::book,
                mla::book,
                lncs::book,
                apj::book,
            )),
        )),
    );

    // Drive the parser with the expectation that it will be able to parse all input into a reference
    match parser.parse(input) {
        Ok(reference) => Ok(reference),
        Err(error) => {
            // The parse could not consume all input so decide, based on how far it got whether to use
            // the result, or to use fallback parser. Do this base on proportion of alphanumeric chars
            // parsed (crude measure of proportion of information captured).
            let use_partial = if error.offset() >= input.len() {
                // Parsing got all the way to the end without success
                false
            } else {
                // Parsing matched partially but there was some unmatched content at the end
                let span = error.char_span();
                let remaining = input[span.start..]
                    .chars()
                    .filter(|c| c.is_alphanumeric())
                    .count();
                let total = input.chars().filter(|c| c.is_alphanumeric()).count();

                tracing::debug!("Unmatched content: {}", &input[span.start..]);

                remaining < 3 || (total > 0 && (remaining * 100 / total) < 10)
            };

            if use_partial {
                parser.parse_next(input).or_else(|_| fallback(input))
            } else {
                fallback(input)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use codec::schema::{CreativeWorkType, IntegerOrString, Organization, PersonOrOrganization};
    use codec_text_trait::to_text;
    use common_dev::pretty_assertions::assert_eq;

    use super::*;

    /// Test of use of fallback parser when large proportion of records are not match
    #[test]
    fn fallback() -> Result<()> {
        let r = reference(&mut "Plain text with no structure, DOI or URL")?;
        assert_eq!(r.work_type, None);
        assert_eq!(
            r.text,
            Some("Plain text with no structure, DOI or URL".into())
        );
        assert_eq!(r.doi, None);
        assert_eq!(r.url, None);

        let r = reference(&mut "Plain text with a doi 10.12345/xyz")?;
        assert_eq!(r.work_type, None);
        assert_eq!(r.text, Some("Plain text with a".into()));
        assert_eq!(r.doi, Some("10.12345/xyz".into()));
        assert_eq!(r.url, None);

        let r = reference(&mut "Plain text with a url https://example.org")?;
        assert_eq!(r.work_type, Some(CreativeWorkType::WebPage));
        assert_eq!(r.text, Some("Plain text with a".into()));
        assert_eq!(r.doi, None);
        assert_eq!(r.url, Some("https://example.org".into()));

        Ok(())
    }

    /// Plain text references in https://zenodo.org/api/records/15308198
    ///
    /// These tests are principally to check for routing to the correct parser
    /// so only check the work type and the last property.
    #[test]
    fn zenodo_15308198() -> Result<()> {
        let r = reference(
            &mut "1.\tBirla, N. (2019). Vehicle Dataset from CarDekho. Retrieved from: https://www.kaggle.com/datasets/nehalbirla/vehicle-dataset-from-cardekho",
        )?;
        assert_eq!(r.work_type, Some(CreativeWorkType::WebPage));
        assert_eq!(
            r.url,
            Some("https://www.kaggle.com/datasets/nehalbirla/vehicle-dataset-from-cardekho".into())
        );

        let r = reference(
            &mut "2.\tPedregosa, F., Varoquaux, G., Gramfort, A., Michel, V., Thirion, B., Grisel, O., ... & Duchesnay, E. (2011). Scikit-learn: Machine learning in Python. Journal of Machine Learning Research, 12, 2825–2830.",
        )?;
        assert_eq!(r.work_type, Some(CreativeWorkType::Article));
        assert_eq!(
            r.title.map(|title| to_text(&title)),
            Some("Scikit-learn: Machine learning in Python".to_string())
        );
        assert_eq!(
            r.is_part_of
                .and_then(|journal| journal.title)
                .map(|title| to_text(&title)),
            Some("Journal of Machine Learning Research".to_string())
        );
        assert_eq!(r.page_end, Some(IntegerOrString::Integer(2830)));

        let r = reference(
            &mut "3.\tMcKinney, W. (2010). Data structures for statistical computing in Python. In Proceedings of the 9th Python in Science Conference (pp. 51–56).",
        )?;
        assert_eq!(r.work_type, Some(CreativeWorkType::Chapter));
        assert_eq!(
            r.is_part_of
                .and_then(|book| book.title)
                .map(|title| to_text(&title)),
            Some("Proceedings of the 9th Python in Science Conference".to_string())
        );

        let r = reference(
            &mut "4.\tHunter, J. D. (2007). Matplotlib: A 2D graphics environment. Computing in Science & Engineering, 9(3), 90–95.",
        )?;
        assert_eq!(r.work_type, Some(CreativeWorkType::Article));
        assert_eq!(r.page_end, Some(IntegerOrString::Integer(95)));

        let r = reference(
            &mut "5.\tWaskom, M. (2021). Seaborn: statistical data visualization. Journal of Open Source Software, 6(60), 3021.",
        )?;
        assert_eq!(r.work_type, Some(CreativeWorkType::Article));
        assert_eq!(r.page_start, Some(IntegerOrString::Integer(3021)));

        let r = reference(
            &mut "6.\tJames, G., Witten, D., Hastie, T., & Tibshirani, R. (2013). An Introduction to Statistical Learning: with Applications in R. Springer.",
        )?;
        assert_eq!(r.work_type, Some(CreativeWorkType::Book));
        assert_eq!(
            r.publisher,
            Some(PersonOrOrganization::Organization(Organization {
                name: Some("Springer".to_string()),
                ..Default::default()
            }))
        );

        let r = reference(
            &mut "7.\tTibshirani, R. (1996). Regression shrinkage and selection via the Lasso. Journal of the Royal Statistical Society: Series B (Methodological), 58(1), 267-288.",
        )?;
        assert_eq!(r.work_type, Some(CreativeWorkType::Article));
        assert_eq!(r.page_end, Some(IntegerOrString::Integer(288)));

        let r = reference(
            &mut "8.\tRaschka, S., & Mirjalili, V. (2017). Python Machine Learning: Machine Learning and Deep Learning with Python, scikit-learn, and TensorFlow 2. Packt Publishing Ltd.",
        )?;
        assert_eq!(r.work_type, Some(CreativeWorkType::Book));
        assert_eq!(
            r.publisher,
            Some(PersonOrOrganization::Organization(Organization {
                name: Some("Packt Publishing Ltd".to_string()),
                ..Default::default()
            }))
        );

        Ok(())
    }

    // References extracted from arXiv 2507.09057v1 HTML as plain text that had issues
    #[test]
    fn arxiv_2507_09057v1() -> Result<()> {
        let r = reference(
            &mut "American Dental Association (2022). Eruption Charts. https://www.mouthhealthy.org/all-topics-a-z/eruption-charts.",
        )?;
        assert_eq!(r.work_type, Some(CreativeWorkType::WebPage));
        assert_eq!(
            r.url,
            Some("https://www.mouthhealthy.org/all-topics-a-z/eruption-charts".into())
        );

        let r = reference(
            &mut "Anyaso-Samuel, S., Bandyopadhyay, D., and Datta, S. (2023). Pseudo-value regression of clustered multistate current status data with informative cluster sizes. Statistical Methods in Medical Research, 32(8):1494–1510.",
        )?;
        assert_eq!(r.work_type, Some(CreativeWorkType::Article));
        assert_eq!(r.page_end, Some(IntegerOrString::Integer(1510)));

        let r = reference(
            &mut "Anyaso-Samuel, S. and Datta, S. (2024). Nonparametric estimation of a future entry time distribution given the knowledge of a past state occupation in a progressive multistate model with current status data. arXiv preprint arXiv:2405.05781.",
        )?;
        assert_eq!(r.work_type, Some(CreativeWorkType::Article));
        assert_eq!(r.pagination, Some("2405.05781".into()));

        let r = reference(
            &mut "Bietti, A., Bruna, J., Sanford, C., and Song, M. J. (2022). Learning single-index models with shallow neural networks. In Advances in Neural Information Processing Systems, volume 35, pages 9768–9783. Curran Associates, Inc.",
        )?;
        assert_eq!(r.work_type, Some(CreativeWorkType::Chapter));
        assert_eq!(
            r.is_part_of
                .and_then(|book| book.title)
                .map(|title| to_text(&title)),
            Some("Advances in Neural Information Processing Systems, volume 35, pages 9768–9783. Curran Associates, Inc.".to_string())
        );
        assert_eq!(r.page_end, None);

        let r = reference(
            &mut "Escobar, M. D. and West, M. (1995). Bayesian density estimation and inference using mixtures. Journal of the American Statistical Association, 90(430):577–588.",
        )?;
        assert_eq!(r.work_type, Some(CreativeWorkType::Article));
        assert_eq!(r.page_end, Some(IntegerOrString::Integer(588)));

        let r = reference(
            &mut "Mdala, I., Olsen, I., Haffajee, A. D., Socransky, S. S., Thoresen, M., and de Blasio, B. F. (2014). Comparing clinical attachment level and pocket depth for predicting periodontal disease progression in healthy sites of patients with chronic periodontitis using multi-state markov models. Journal of Clinical Periodontology, 41(9):837–845.",
        )?;
        assert_eq!(r.work_type, Some(CreativeWorkType::Article));
        assert_eq!(r.page_end, Some(IntegerOrString::Integer(845)));

        Ok(())
    }

    // References extracted from arXiv 2507.09057v1 HTML as plain text that had issues
    #[test]
    fn arxiv_2507_11127v1() -> Result<()> {
        let r = reference(
            &mut "Kareem Ahmed, Stefano Teso, Kai-Wei Chang, Guy Van den Broeck, and Antonio Vergari. Semantic probabilistic layers for neuro-symbolic learning. Advances in Neural Information Processing Systems, 35:29944–29959, 2022.",
        )?;
        assert_eq!(r.work_type, Some(CreativeWorkType::Article));
        assert_eq!(r.page_end, Some(IntegerOrString::Integer(29959)));
        assert!(r.date.is_some());

        let r = reference(
            &mut "Vaishak Belle, Andrea Passerini, Guy Van den Broeck, et al. Probabilistic inference in hybrid domains by weighted model integration. In Proceedings of 24th International Joint Conference on Artificial Intelligence (IJCAI), pages 2770–2776. AAAI Press/International Joint Conferences on Artificial Intelligence, 2015.",
        )?;
        assert_eq!(r.work_type, Some(CreativeWorkType::Article));
        assert_eq!(
            r.title.map(|title| to_text(&title)),
            Some(
                "Probabilistic inference in hybrid domains by weighted model integration"
                    .to_string()
            )
        );
        assert_eq!(r.is_part_of.clone().and_then(|book| book.editors), None);
        assert_eq!(
            r.is_part_of
                .and_then(|book| book.title)
                .map(|title| to_text(&title)),
            Some("Proceedings of 24th International Joint Conference on Artificial Intelligence (IJCAI)".to_string())
        );
        assert_eq!(r.page_end, Some(IntegerOrString::Integer(2776)));
        assert!(r.date.is_some());

        Ok(())
    }

    // References extracted from arXiv 2507.13317v1 HTML as plain text that had issues
    #[test]
    fn arxiv_2507_13317v1() -> Result<()> {
        let r = reference(
            &mut "Abadi M. G., Navarro J. F., Fardal M., Babul A., Steinmetz M., 2010, MNRAS, 407, 435",
        )?;
        assert_eq!(r.work_type, Some(CreativeWorkType::Article));
        assert_eq!(r.page_start, Some(IntegerOrString::Integer(435)));
        assert!(r.date.is_some());

        Ok(())
    }

    // References extracted from arXiv 2507.11353v1 HTML as plain text that had issues
    #[test]
    fn arxiv_2507_11353v1() -> Result<()> {
        let r = reference(
            &mut "Acharya, V. V., Berner, R., Engle, R., Jung, H., Stroebel, J., Zeng, X. and Zhao, Y. (2023), ‘Climate stress testing’, Annual Review of Financial Economics 15(1), 291–326.",
        )?;
        assert_eq!(r.work_type, Some(CreativeWorkType::Article));
        assert_eq!(
            r.is_part_of
                .and_then(|journal| journal.title)
                .map(|title| to_text(&title)),
            Some("Annual Review of Financial Economics".to_string())
        );
        assert_eq!(r.page_end, Some(IntegerOrString::Integer(326)));
        assert!(r.date.is_some());

        Ok(())
    }

    // References extracted from biRxiv 2024.12.10.627863v2 PDF as plain text that had issues
    #[test]
    fn biorxiv_2024_12_10_627863v2() -> Result<()> {
        // "et al" without a preceding comma was unrecognized
        let r = reference(
            &mut r#"S. M. Paul et al., "How to improve R&D productivity: The pharmaceutical industry's grand challenge," Nat Rev Drug Discov, vol. 9, no. 3, pp. 203-214, 2010."#,
        )?;
        assert_eq!(r.work_type, Some(CreativeWorkType::Article));
        assert_eq!(
            r.is_part_of
                .clone()
                .and_then(|journal| journal.title)
                .map(|title| to_text(&title)),
            Some("Nat Rev Drug Discov".to_string())
        );
        assert_eq!(
            r.is_part_of
                .and_then(|journal| journal.issue_number.clone()),
            Some(IntegerOrString::Integer(3))
        );
        assert_eq!(r.page_end, Some(IntegerOrString::Integer(214)));
        assert!(r.date.is_some());

        // Non-numeric issue and pages
        let r = reference(
            &mut r#"A. P. Davis, T. C. Wiegers, R. J. Johnson, D. Sciaky, J. Wiegers, and C. J. Mattingly, "Comparative Toxicogenomics database (CTD): Update 2023," Nucleic Acids Res, vol. 51, no. D1, pp. D1257-D1262, 2023"#,
        )?;
        assert_eq!(r.work_type, Some(CreativeWorkType::Article));
        assert_eq!(
            r.is_part_of
                .and_then(|journal| journal.issue_number.clone()),
            Some(IntegerOrString::String("D1".into()))
        );
        assert_eq!(r.page_end, Some(IntegerOrString::String("D1262".into())));
        assert!(r.date.is_some());

        Ok(())
    }

    // References extracted from biRxiv 2025.05.12.25325848v3 PDF as plain text that had issues
    #[test]
    fn biorxiv_2025_05_12_25325848v3() -> Result<()> {
        // No space between initials, and abbreviated journal name
        let r = reference(
            &mut r#"Pfohl, S.R., Kim, R.B., Coan, G.S., and Mitchell, C.S. (2018). Unraveling the complexity of amyotrophic lateral sclerosis survival prediction. Front. Neuroinform. 12"#,
        )?;
        assert_eq!(r.work_type, Some(CreativeWorkType::Article));
        assert!(r.date.is_some());
        assert_eq!(
            r.is_part_of
                .clone()
                .and_then(|journal| journal.title)
                .map(|title| to_text(&title)),
            Some("Front. Neuroinform.".to_string())
        );
        assert_eq!(
            r.is_part_of
                .and_then(|journal| journal.volume_number.clone()),
            Some(IntegerOrString::Integer(12))
        );

        // Hyphen before second initial
        let r = reference(
            &mut r#"Beghi, E., Chiò, A., Couratier, P., Esteban, J., Hardiman, O., Logroscino, G., Millul, A., Mitchell, D., Preux, P.-M., Pupillo, E., et al. (2011). The epidemiology and treatment of ALS: focus on the heterogeneity of the disease and critical appraisal of therapeutic trials. Amyotroph. Lateral Scler. 12, 1-10"#,
        )?;
        assert_eq!(r.work_type, Some(CreativeWorkType::Article));
        assert!(r.date.is_some());
        assert_eq!(
            r.is_part_of
                .clone()
                .and_then(|journal| journal.title)
                .map(|title| to_text(&title)),
            Some("Amyotroph. Lateral Scler.".to_string())
        );
        assert_eq!(
            r.is_part_of
                .and_then(|journal| journal.volume_number.clone()),
            Some(IntegerOrString::Integer(12))
        );
        assert_eq!(r.page_end, Some(IntegerOrString::Integer(10)));

        Ok(())
    }
}
