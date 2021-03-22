use stencila_schema::{Article, VecInlineContentString};

#[test]
fn create_article() {
    let article: Article = Default::default();
    assert!(article.title.is_none());

    let article = Article {
        title: Some(VecInlineContentString::String(
            "The article title".to_string(),
        )),
        ..Default::default()
    };
    assert!(!article.title.is_none());
}
