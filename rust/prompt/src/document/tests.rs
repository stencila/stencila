use crate::{prelude::*, PromptContext};

use common_dev::pretty_assertions::assert_eq;
use kernel_quickjs::kernel::common::tokio;
use schema::{shortcuts::t, Node, Null, SectionType};

use super::{
    code_chunks::{CodeChunk, CodeChunks},
    headings::{Heading, Headings},
    node::Node as ContextNode,
    paragraphs::{Paragraph, Paragraphs},
    sections::Sections,
    Document, Metadata,
};

#[tokio::test]
async fn metadata() -> Result<()> {
    let context = PromptContext {
        document: Some(Document {
            metadata: Metadata {
                title: Some("The title".into()),
                description: Some("A description".into()),
                genre: Some("The genre".into()),
                keywords: Some("some, key, words".into()),
            },
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut kernel = context.into_kernel().await?;
    kernel.execute("const md = document.metadata").await?;

    let (output, messages) = kernel.evaluate("md").await?;
    assert_eq!(messages, []);
    assert_eq!(
        serde_json::to_string_pretty(&output)?,
        r#"{
  "title": "The title",
  "description": "A description",
  "genre": "The genre",
  "keywords": "some, key, words"
}"#
    );

    let (output, messages) = kernel.evaluate("md.properties.join()").await?;
    assert_eq!(messages, []);
    assert_eq!(
        output,
        Node::String("title,description,genre,keywords".into())
    );

    let (output, messages) = kernel.evaluate("md.title").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("The title".into()));

    let (output, messages) = kernel.evaluate("md.description").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("A description".into()));

    let (output, messages) = kernel.evaluate("md.genre").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("The genre".into()));

    let (output, messages) = kernel.evaluate("md.keywords").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("some, key, words".into()));

    Ok(())
}

#[tokio::test]
async fn sections() -> Result<()> {
    let context = PromptContext {
        document: Some(Document {
            sections: Sections::new(vec![
                SectionType::Introduction.to_string(),
                SectionType::Methods.to_string(),
                SectionType::Results.to_string(),
                SectionType::Discussion.to_string(),
            ]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut kernel = context.into_kernel().await?;
    kernel.execute("const secs = document.sections").await?;

    let (output, messages) = kernel.evaluate("secs.count").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Integer(4));

    let (output, messages) = kernel.evaluate("secs.all.length").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Integer(4));

    let (output, messages) = kernel.evaluate("secs.first").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("Introduction".into()));

    let (output, messages) = kernel.evaluate("secs.last").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("Discussion".into()));

    {
        kernel.execute("secs._enter()").await?;

        let (output, ..) = kernel.evaluate("secs.previous").await?;
        assert_eq!(output, Node::Null(Null));

        let (output, ..) = kernel.evaluate("secs.current").await?;
        assert_eq!(output, Node::String("Introduction".into()));

        let (output, ..) = kernel.evaluate("secs.next").await?;
        assert_eq!(output, Node::String("Methods".into()));

        kernel.execute("secs._exit()").await?;
    }

    assert_eq!(kernel.evaluate("secs.current").await?.0, Node::Null(Null));

    {
        kernel.execute("secs._enter()").await?;

        let (output, ..) = kernel.evaluate("secs.previous").await?;
        assert_eq!(output, Node::String("Introduction".into()));

        let (output, ..) = kernel.evaluate("secs.current").await?;
        assert_eq!(output, Node::String("Methods".into()));

        let (output, ..) = kernel.evaluate("secs.next").await?;
        assert_eq!(output, Node::String("Results".into()));

        kernel.execute("secs._exit()").await?;
    }

    assert_eq!(kernel.evaluate("secs.current").await?.0, Node::Null(Null));

    {
        kernel.execute("secs._enter()").await?;

        let (output, ..) = kernel.evaluate("secs.previous").await?;
        assert_eq!(output, Node::String("Methods".into()));

        let (output, ..) = kernel.evaluate("secs.current").await?;
        assert_eq!(output, Node::String("Results".into()));

        let (output, ..) = kernel.evaluate("secs.next").await?;
        assert_eq!(output, Node::String("Discussion".into()));

        kernel.execute("secs._exit()").await?;
    }

    Ok(())
}

#[tokio::test]
async fn section_headings() -> Result<()> {
    let mut sections = Sections::default();
    sections.push_heading(&schema::Heading::new(1, vec![t("Introduction")]));
    sections.push_heading(&schema::Heading::new(1, vec![t("Methods")]));
    sections.push_heading(&schema::Heading::new(1, vec![t("Results")]));
    sections.push_heading(&schema::Heading::new(1, vec![t("Discussion")]));

    let context = PromptContext {
        document: Some(Document {
            sections,
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut kernel = context.into_kernel().await?;
    kernel.execute("const secs = document.sections").await?;

    let (output, messages) = kernel.evaluate("secs.count").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Integer(4));

    {
        kernel.execute("secs._enter()").await?;

        let (output, ..) = kernel.evaluate("secs.previous").await?;
        assert_eq!(output, Node::Null(Null));

        let (output, ..) = kernel.evaluate("secs.current").await?;
        assert_eq!(output, Node::String("Introduction".into()));

        let (output, ..) = kernel.evaluate("secs.next").await?;
        assert_eq!(output, Node::String("Methods".into()));

        kernel.execute("secs._exit()").await?;
    }

    Ok(())
}

#[tokio::test]
async fn headings() -> Result<()> {
    let context = PromptContext {
        document: Some(Document {
            headings: Headings::new(vec![
                Heading::new(1, "H1"),
                Heading::new(2, "H1.1"),
                Heading::new(3, "H1.1.1"),
                Heading::new(1, "H2"),
            ]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut kernel = context.into_kernel().await?;
    kernel.execute("const hs = document.headings").await?;

    let (output, messages) = kernel.evaluate("hs.count").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Integer(4));

    let (output, messages) = kernel.evaluate("hs.all.length").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Integer(4));

    let (output, messages) = kernel.evaluate("hs.first").await?;
    assert_eq!(messages, []);
    assert_eq!(
        serde_json::to_string_pretty(&output)?,
        r#"{
  "level": 1,
  "content": "H1"
}"#
    );

    let (output, messages) = kernel.evaluate("hs.first.content").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("H1".into()));

    let (output, messages) = kernel.evaluate("hs.last.content").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("H2".into()));

    let (output, messages) = kernel.evaluate("hs.hierarchy.length").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Integer(0));

    let (output, messages) = kernel.evaluate("hs.previous").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Null(Null));

    let (output, messages) = kernel.evaluate("hs.current").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Null(Null));

    let (output, messages) = kernel.evaluate("hs.next.content").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("H1".into()));

    {
        kernel.execute("hs._enter()").await?;

        let (output, messages) = kernel.evaluate("hs.previous").await?;
        assert_eq!(messages, []);
        assert_eq!(output, Node::Null(Null));

        let (output, messages) = kernel.evaluate("hs.current.content").await?;
        assert_eq!(messages, []);
        assert_eq!(output, Node::String("H1".into()));

        let (output, messages) = kernel.evaluate("hs.next.content").await?;
        assert_eq!(messages, []);
        assert_eq!(output, Node::String("H1.1".into()));

        let (output, messages) = kernel
            .evaluate("hs.hierarchy.map(h => h.content).join()")
            .await?;
        assert_eq!(messages, []);
        assert_eq!(output, Node::String("H1".into()));

        kernel.execute("hs._exit()").await?;
    }

    assert_eq!(kernel.evaluate("hs.current").await?.0, Node::Null(Null));

    {
        kernel.execute("hs._enter()").await?;

        let (output, messages) = kernel.evaluate("hs.previous.content").await?;
        assert_eq!(messages, []);
        assert_eq!(output, Node::String("H1".into()));

        let (output, messages) = kernel.evaluate("hs.current.content").await?;
        assert_eq!(messages, []);
        assert_eq!(output, Node::String("H1.1".into()));

        let (output, messages) = kernel.evaluate("hs.next.content").await?;
        assert_eq!(messages, []);
        assert_eq!(output, Node::String("H1.1.1".into()));

        let (output, messages) = kernel
            .evaluate("hs.hierarchy.map(h => h.content).join()")
            .await?;
        assert_eq!(messages, []);
        assert_eq!(output, Node::String("H1,H1.1".into()));

        kernel.execute("hs._exit()").await?;
    }

    assert_eq!(kernel.evaluate("hs.current").await?.0, Node::Null(Null));

    {
        kernel.execute("hs._enter()").await?;

        let (output, messages) = kernel
            .evaluate("hs.hierarchy.map(h => h.content).join()")
            .await?;
        assert_eq!(messages, []);
        assert_eq!(output, Node::String("H1,H1.1,H1.1.1".into()));

        kernel.execute("hs._exit()").await?;
    }

    assert_eq!(kernel.evaluate("hs.current").await?.0, Node::Null(Null));

    Ok(())
}

#[tokio::test]
async fn paragraphs() -> Result<()> {
    let context = PromptContext {
        document: Some(Document {
            paragraphs: Paragraphs::new(vec![
                Paragraph::new("Para one."),
                Paragraph::new("Para two."),
            ]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut kernel = context.into_kernel().await?;
    kernel.execute("const ps = document.paragraphs").await?;

    let (output, messages) = kernel.evaluate("ps.count").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Integer(2));

    let (output, messages) = kernel.evaluate("ps.all.length").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Integer(2));

    let (output, messages) = kernel.evaluate("ps.first").await?;
    assert_eq!(messages, []);
    assert_eq!(
        serde_json::to_string_pretty(&output)?,
        r#"{
  "content": "Para one."
}"#
    );

    let (output, messages) = kernel.evaluate("ps.first.content").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("Para one.".into()));

    let (output, messages) = kernel.evaluate("ps.last.content").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("Para two.".into()));

    let (output, messages) = kernel.evaluate("ps.previous").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Null(Null));

    let (output, messages) = kernel.evaluate("ps.current").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Null(Null));

    let (output, messages) = kernel.evaluate("ps.next.content").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("Para one.".into()));

    {
        kernel.execute("ps._enter()").await?;

        let (output, messages) = kernel.evaluate("ps.previous").await?;
        assert_eq!(messages, []);
        assert_eq!(output, Node::Null(Null));

        let (output, messages) = kernel.evaluate("ps.current.content").await?;
        assert_eq!(messages, []);
        assert_eq!(output, Node::String("Para one.".into()));

        let (output, messages) = kernel.evaluate("ps.next.content").await?;
        assert_eq!(messages, []);
        assert_eq!(output, Node::String("Para two.".into()));

        kernel.execute("ps._exit()").await?;
    }

    assert_eq!(kernel.evaluate("ps.current").await?.0, Node::Null(Null));

    {
        kernel.execute("ps._enter()").await?;

        let (output, messages) = kernel.evaluate("ps.previous.content").await?;
        assert_eq!(messages, []);
        assert_eq!(output, Node::String("Para one.".into()));

        let (output, messages) = kernel.evaluate("ps.current.content").await?;
        assert_eq!(messages, []);
        assert_eq!(output, Node::String("Para two.".into()));

        let (output, messages) = kernel.evaluate("ps.next").await?;
        assert_eq!(messages, []);
        assert_eq!(output, Node::Null(Null));

        kernel.execute("ps._exit()").await?;
    }

    assert_eq!(kernel.evaluate("ps.current").await?.0, Node::Null(Null));

    Ok(())
}

#[tokio::test]
async fn code_chunks() -> Result<()> {
    let context = PromptContext {
        document: Some(Document {
            code_chunks: CodeChunks::new(vec![
                CodeChunk::new(
                    "python",
                    "1 + 2",
                    Some(vec![ContextNode::from(&Node::Integer(3))]),
                ),
                CodeChunk::new("", "3 + 4", None),
            ]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut kernel = context.into_kernel().await?;
    kernel.execute("const cc = document.codeChunks").await?;

    let (output, messages) = kernel.evaluate("cc.count").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Integer(2));

    let (output, messages) = kernel.evaluate("cc.all.length").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Integer(2));

    let (output, messages) = kernel.evaluate("cc.first").await?;
    assert_eq!(messages, []);
    assert_eq!(
        serde_json::to_string_pretty(&output)?,
        r#"{
  "language": "python",
  "code": "1 + 2",
  "outputs": [
    {
      "type": "Integer",
      "json": "3",
      "markdown": "3"
    }
  ],
  "markdown": ""
}"#
    );

    let (output, messages) = kernel.evaluate("cc.first.outputs[0].value").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Integer(3));

    let (output, messages) = kernel.evaluate("cc.last.code").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("3 + 4".into()));

    let (output, messages) = kernel.evaluate("cc.preceding.length").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Integer(0));

    let (output, messages) = kernel.evaluate("cc.previous").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Null(Null));

    let (output, messages) = kernel.evaluate("cc.next.code").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("1 + 2".into()));

    {
        kernel.execute("cc._enter()").await?;

        let (output, messages) = kernel.evaluate("cc.preceding.length").await?;
        assert_eq!(messages, []);
        assert_eq!(output, Node::Integer(0));

        let (output, messages) = kernel.evaluate("cc.previous").await?;
        assert_eq!(messages, []);
        assert_eq!(output, Node::Null(Null));

        let (output, messages) = kernel.evaluate("cc.current.code").await?;
        assert_eq!(messages, []);
        assert_eq!(output, Node::String("1 + 2".into()));

        let (output, messages) = kernel.evaluate("cc.next.code").await?;
        assert_eq!(messages, []);
        assert_eq!(output, Node::String("3 + 4".into()));

        kernel.execute("cc._exit()").await?;
    }

    assert_eq!(kernel.evaluate("cc.current").await?.0, Node::Null(Null));

    {
        kernel.execute("cc._enter()").await?;

        let (output, messages) = kernel.evaluate("cc.preceding.length").await?;
        assert_eq!(messages, []);
        assert_eq!(output, Node::Integer(1));

        let (output, messages) = kernel.evaluate("cc.previous.code").await?;
        assert_eq!(messages, []);
        assert_eq!(output, Node::String("1 + 2".into()));

        let (output, messages) = kernel.evaluate("cc.current.code").await?;
        assert_eq!(messages, []);
        assert_eq!(output, Node::String("3 + 4".into()));

        let (output, messages) = kernel.evaluate("cc.next").await?;
        assert_eq!(messages, []);
        assert_eq!(output, Node::Null(Null));

        kernel.execute("cc._exit()").await?;
    }

    assert_eq!(
        kernel.evaluate("cc.preceding.length").await?.0,
        Node::Integer(2)
    );
    assert_eq!(kernel.evaluate("cc.current").await?.0, Node::Null(Null));

    Ok(())
}
