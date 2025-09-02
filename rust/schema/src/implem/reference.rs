use crate::{
    Article, CreativeWork, CreativeWorkType, CreativeWorkVariant, Reference, prelude::*, replicate,
};

impl From<&Node> for Reference {
    fn from(node: &Node) -> Self {
        match node {
            Node::Article(article) => Reference::from(article),
            _ => Reference::default(),
        }
    }
}

impl From<&CreativeWorkVariant> for Reference {
    fn from(work: &CreativeWorkVariant) -> Self {
        match work {
            CreativeWorkVariant::Article(article) => Reference::from(article),
            _ => Reference {
                work_type: Some(work.work_type()),
                doi: work.doi(),
                title: work.title(),
                ..Default::default()
            },
        }
    }
}

impl From<&CreativeWork> for Reference {
    fn from(work: &CreativeWork) -> Self {
        Self {
            work_type: work.work_type,
            doi: work.doi.clone(),
            authors: work
                .options
                .authors
                .as_ref()
                .and_then(|authors| replicate(authors).ok()),
            date: work
                .options
                .date_published
                .as_ref()
                .or(work.options.date_modified.as_ref())
                .and_then(|date| replicate(date).ok()),
            title: work
                .options
                .title
                .as_ref()
                .and_then(|title| replicate(title).ok()),
            is_part_of: work
                .options
                .is_part_of
                .as_ref()
                .map(|is_part_of| Box::new(Reference::from(is_part_of))),
            ..Default::default()
        }
    }
}

impl From<&Article> for Reference {
    fn from(article: &Article) -> Self {
        Self {
            work_type: Some(CreativeWorkType::Article),
            doi: article.doi.clone(),
            authors: article
                .authors
                .as_ref()
                .and_then(|authors| replicate(authors).ok()),
            date: article
                .date_published
                .as_ref()
                .or(article.options.date_modified.as_ref())
                .or(article.options.date_accepted.as_ref())
                .or(article.options.date_received.as_ref())
                .or(article.options.date_created.as_ref())
                .and_then(|date| replicate(date).ok()),
            title: article.title(),
            is_part_of: article.is_part_of().map(Box::new),
            page_start: article.options.page_start.clone(),
            page_end: article.options.page_end.clone(),
            pagination: article.options.pagination.clone(),
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
