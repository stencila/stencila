use super::prelude::*;
use stencila_schema::Article;

patchable_struct!(Article, content);

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
