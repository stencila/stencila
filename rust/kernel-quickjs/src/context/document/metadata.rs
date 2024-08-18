use rquickjs::class::Trace;

use codec_markdown_trait::to_markdown;
use kernel::schema;

/// The metadata of the current document
#[derive(Default, Clone, Trace)]
#[rquickjs::class]
pub struct Metadata {
    /// The document's title as a Markdown string
    #[qjs(get)]
    pub title: Option<String>,

    /// The genre of the article
    #[qjs(get)]
    pub genre: Option<String>,

    /// The keywords of the article as comma separated strings
    #[qjs(get)]
    pub keywords: Option<String>,
}

impl From<&schema::Article> for Metadata {
    fn from(article: &schema::Article) -> Self {
        Self {
            title: article.title.as_ref().map(|title| to_markdown(title)),
            genre: article.genre.as_ref().map(|genre| genre.join(", ")),
            keywords: article
                .keywords
                .as_ref()
                .map(|keywords| keywords.join(", ")),
        }
    }
}
