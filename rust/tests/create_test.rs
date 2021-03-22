use stencila_schema::{Article, ArticleTitle};

#[test]
fn create_article() {
    let article: Article = Default::default();
    assert!(article.title.is_none());

    let article = Article {
        title: Some(ArticleTitle::String(
            "The article title".into(),
        )),
        ..Default::default()
    };
    assert!(!article.title.is_none());
}
