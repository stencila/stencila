use stencila_schema::{
    Article, ArticleAuthors, ArticleTitle, BlockContent, CodeExpression, InlineContent, Paragraph,
    Person,
};

#[test]
fn article() {
    let article: Article = Default::default();
    assert!(article.title.is_none());

    let article = Article {
        title: Some(ArticleTitle::String("The article title".into())),
        authors: Some(vec![ArticleAuthors::Person({
            Person {
                given_names: Some(vec!["Jane".into()]),
                family_names: Some(vec!["Jones".into()]),
                ..Default::default()
            }
        })]),
        content: Some(vec![BlockContent::Paragraph(Paragraph {
            content: vec![
                InlineContent::String("A paragraph with a".into()),
                InlineContent::CodeExpression(CodeExpression {
                    programming_language: Some("r".into()),
                    text: "2^2".into(),
                    ..Default::default()
                }),
            ],
            ..Default::default()
        })]),
        ..Default::default()
    };
    assert!(!article.title.is_none());
    assert!(!article.authors.is_none());
}
