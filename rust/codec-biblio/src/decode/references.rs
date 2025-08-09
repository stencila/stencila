//! Parsers that parse a Stencila [`Reference`] from a string

use winnow::{
    Parser, Result,
    ascii::{digit1, multispace0, multispace1, newline},
    combinator::{alt, opt, preceded, repeat, separated},
};

use codec::schema::Reference;

use super::apa;
use super::chicago;
use super::fallback::fallback;
use super::ieee;
use super::mla;
use super::vancouver;

/// Parse a list of Stencila [`Reference`]s from a string
pub fn references(input: &mut &str) -> Result<Vec<Reference>> {
    separated(0.., reference, repeat::<_, _, (), _, _>(1.., newline)).parse_next(input)
}

/// Parse a Stencila [`Reference`] from a string
///
/// The aim of this function is to extract as much bibliographic information as
/// possible from plain text. It attempts to parse reference using several
/// popular formats falling back to just trying to extract a DOI (since that
/// is the most valuable bibliographic information).
pub fn reference(input: &mut &str) -> Result<Reference> {
    preceded(
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
            // Chapter
            vancouver::chapter,
            ieee::chapter,
            apa::chapter,
            chicago::chapter,
            mla::chapter,
            // Article
            vancouver::article,
            ieee::article,
            apa::article,
            chicago::article,
            mla::article,
            // Web
            vancouver::web,
            ieee::web,
            apa::web,
            chicago::web,
            mla::web,
            // Book
            apa::book,
            vancouver::book,
            ieee::book,
            chicago::book,
            mla::book,
            // Fallback
            fallback,
        )),
    )
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use codec::schema::{CreativeWorkType, IntegerOrString, Organization, PersonOrOrganization};
    use codec_text_trait::to_text;
    use common_dev::pretty_assertions::assert_eq;

    use super::*;

    /// Plain text reference in https://zenodo.org/api/records/15308198
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
}
