use crate::{prelude::*, replicate, Article, Reference};

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
            doi: article.doi.clone(),
            authors: article
                .authors
                .as_ref()
                .and_then(|authors| replicate(authors).ok()),
            date: article
                .date_published
                .as_ref()
                .or(article.date_modified.as_ref())
                .and_then(|date| replicate(date).ok()),
            title: article
                .title
                .as_ref()
                .and_then(|title| replicate(title).ok()),
            ..Default::default()
        }
    }
}
