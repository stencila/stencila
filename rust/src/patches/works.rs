use super::prelude::*;
use stencila_schema::Article;

impl Diffable for Article {
    diffable_is_same!();
    diffable_diff!();

    fn is_equal(&self, other: &Self) -> Result<()> {
        self.content.is_equal(&other.content)?;
        // TODO add other properties using macro
        Ok(())
    }

    fn diff_same(&self, differ: &mut Differ, other: &Self) {
        differ.field("content", &self.content, &other.content)
        // TODO add other properties using macro
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        assert_json,
        patches::{apply_new, diff, equal},
    };
    use stencila_schema::{BlockContent, Paragraph};

    #[test]
    fn test_article() {
        let article1 = Article {
            content: Some(vec![]),
            ..Default::default()
        };
        let article2 = Article {
            content: Some(vec![BlockContent::Paragraph(Paragraph::default())]),
            ..Default::default()
        };

        diff(&article1, &article2);
    }
}
