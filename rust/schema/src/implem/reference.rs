use crate::{prelude::*, replicate, Article, CreativeWorkType, Reference};

impl From<&Node> for Reference {
    fn from(node: &Node) -> Self {
        match node {
            Node::Article(article) => Reference::from(article),
            _ => Reference::default(),
        }
    }
}

impl From<&CreativeWorkType> for Reference {
    fn from(work: &CreativeWorkType) -> Self {
        match work {
            CreativeWorkType::Article(article) => Reference::from(article),
            _ => Reference {
                title: work.title(),
                ..Default::default()
            },
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

impl MarkdownCodec for Reference {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        let mut content = false;

        if let Some(authors) = &self.authors {
            context.push_prop_fn(NodeProperty::Authors, |context| {
                for (index, author) in authors.iter().enumerate() {
                    if index > 0 {
                        context.push_str(", ");
                    }
                    context.push_str(&author.name());
                }
            });
            content = true;
        };

        if let Some(year) = self.date.as_ref().and_then(|date| date.year()) {
            if content {
                context.push_str(" ");
            }
            context
                .push_str("(")
                .push_prop_str(NodeProperty::Date, &year.to_string())
                .push_str(")");
            content = true;
        }

        if let Some(title) = &self.title {
            if content {
                context.push_str(" ");
            }
            context.push_prop_fn(NodeProperty::Title, |context| title.to_markdown(context));
            if !context.content.ends_with('.') {
                context.push_str(".");
            }
            content = true;
        }

        if let Some(doi) = &self.doi {
            if content {
                context.push_str(" ");
            }
            context
                .push_str("https://doi.org/")
                .push_prop_str(NodeProperty::Doi, doi)
                // Trailing space prevents syntect highlighting when output on the CLI from bleeding to next
                // reference, but should ideally not need to be here
                .push_str(" ");
        }

        context.newline().exit_node().newline();
    }
}
