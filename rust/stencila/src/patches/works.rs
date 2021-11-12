use super::prelude::*;
use stencila_schema::{Article, CreativeWorkContent};

patchable_struct!(Article, content);

patchable_variants!(
    CreativeWorkContent,
    CreativeWorkContent::VecNode,
    CreativeWorkContent::String
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patches::diff;
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
