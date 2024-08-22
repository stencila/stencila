use crate::prelude::*;

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
            title: article.title.as_ref().map(to_markdown),
            genre: article.genre.as_ref().map(|genre| genre.join(", ")),
            keywords: article
                .keywords
                .as_ref()
                .map(|keywords| keywords.join(", ")),
        }
    }
}

#[rquickjs::methods]
impl Metadata {
    #[qjs(rename = PredefinedAtom::ToJSON)]
    fn to_json<'js>(&self, ctx: Ctx<'js>) -> Result<Object<'js>, Error> {
        let obj = Object::new(ctx)?;
        obj.set("title", self.title.clone())?;
        obj.set("genre", self.genre.clone())?;
        obj.set("keywords", self.keywords.clone())?;
        Ok(obj)
    }
}
