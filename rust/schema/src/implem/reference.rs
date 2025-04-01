use codec_text_trait::to_text;

use crate::{prelude::*, Article, Reference};

impl From<&Node> for Reference {
    fn from(node: &Node) -> Self {
        match node {
            Node::Article(article) => Reference::from(article),
            _ => Reference::default(),
        }
    }
}

impl From<&Article> for Reference {
    fn from(article: &Article) -> Self {
        Self {
            doi: article.doi(),
            title: article.title.as_ref().map(to_text),
            date: article
                .date_published
                .as_ref()
                .or(article.date_modified.as_ref())
                .cloned(),
            authors: article
                .authors
                .as_ref()
                .map(|authors| authors.iter().map(|author| author.name()).collect()),
            ..Default::default()
        }
    }
}
