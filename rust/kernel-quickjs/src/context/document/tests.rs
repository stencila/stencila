use common_dev::pretty_assertions::assert_eq;
use kernel::{
    common::{eyre::Result, serde_json, tokio},
    schema::{Node, Null},
    KernelInstance,
};

use crate::{context::Context, QuickJsKernelInstance};

use super::{
    code_chunks::{CodeChunk, CodeChunks},
    headings::{Heading, Headings},
    node::Node as ContextNode,
    paragraphs::{Paragraph, Paragraphs},
    Document, Metadata,
};

#[tokio::test]
async fn metadata() -> Result<()> {
    let context = Context {
        document: Document {
            metadata: Metadata {
                title: Some("The title".into()),
                genre: Some("The genre".into()),
                keywords: Some("some, key, words".into()),
            },
            ..Default::default()
        },
        ..Default::default()
    };

    let mut kernel = QuickJsKernelInstance::new("test".to_string());
    kernel.start_here().await?;
    kernel.set_context(context).await?;
    kernel.execute("const md = document.metadata").await?;

    let (output, messages) = kernel.evaluate("md.title").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("The title".into()));

    let (output, messages) = kernel.evaluate("md.genre").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("The genre".into()));

    let (output, messages) = kernel.evaluate("md.keywords").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("some, key, words".into()));

    Ok(())
}

#[tokio::test]
async fn headings() -> Result<()> {
    let context = Context {
        document: Document {
            headings: Headings::new(vec![
                Heading::new(1, "H1"),
                Heading::new(2, "H1.1"),
                Heading::new(3, "H1.1.1"),
                Heading::new(1, "H2"),
            ]),
            ..Default::default()
        },
        ..Default::default()
    };

    let mut kernel = QuickJsKernelInstance::new("test".to_string());
    kernel.start_here().await?;
    kernel.set_context(context).await?;
    kernel.execute("const hs = document.headings").await?;

    let (output, messages) = kernel.evaluate("hs.count").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Integer(4));

    let (output, messages) = kernel.evaluate("hs.all.length").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Integer(4));

    let (output, messages) = kernel.evaluate("hs.first.content").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("H1".into()));

    let (output, messages) = kernel.evaluate("hs.last.content").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("H2".into()));

    let (output, messages) = kernel.evaluate("hs.hierarchy.length").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Integer(0));

    let (.., messages) = kernel.execute("hs._forward()").await?;
    assert_eq!(messages, []);

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

    let (.., messages) = kernel.execute("hs._forward()").await?;
    assert_eq!(messages, []);

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

    let (.., messages) = kernel.execute("hs._forward()").await?;
    assert_eq!(messages, []);

    let (output, messages) = kernel
        .evaluate("hs.hierarchy.map(h => h.content).join()")
        .await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("H1,H1.1,H1.1.1".into()));

    Ok(())
}

#[tokio::test]
async fn paragraphs() -> Result<()> {
    let context = Context {
        document: Document {
            paragraphs: Paragraphs::new(vec![
                Paragraph::new("Para one."),
                Paragraph::new("Para two."),
            ]),
            ..Default::default()
        },
        ..Default::default()
    };

    let mut kernel = QuickJsKernelInstance::new("test".to_string());
    kernel.start_here().await?;
    kernel.set_context(context).await?;
    kernel.execute("const ps = document.paragraphs").await?;

    let (output, messages) = kernel.evaluate("ps.count").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Integer(2));

    let (output, messages) = kernel.evaluate("ps.all.length").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Integer(2));

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

    let (.., messages) = kernel.execute("ps._forward()").await?;
    assert_eq!(messages, []);

    let (output, messages) = kernel.evaluate("ps.previous").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Null(Null));

    let (output, messages) = kernel.evaluate("ps.current.content").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("Para one.".into()));

    let (output, messages) = kernel.evaluate("ps.next.content").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("Para two.".into()));

    let (.., messages) = kernel.execute("ps._forward()").await?;
    assert_eq!(messages, []);

    let (output, messages) = kernel.evaluate("ps.previous.content").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("Para one.".into()));

    let (output, messages) = kernel.evaluate("ps.current.content").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("Para two.".into()));

    let (output, messages) = kernel.evaluate("ps.next").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Null(Null));

    Ok(())
}

#[tokio::test]
async fn code_chunks() -> Result<()> {
    let context = Context {
        document: Document {
            code_chunks: CodeChunks::new(vec![
                CodeChunk::new(
                    "python",
                    "1 + 2",
                    Some(vec![ContextNode::from(&Node::Integer(3))]),
                ),
                CodeChunk::new("", "code 2", None),
            ]),
            ..Default::default()
        },
        ..Default::default()
    };

    let mut kernel = QuickJsKernelInstance::new("test".to_string());
    kernel.start_here().await?;
    kernel.set_context(context).await?;
    kernel.execute("const cc = document.codeChunks").await?;

    let (output, messages) = kernel.evaluate("cc.count").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Integer(2));

    let (output, messages) = kernel.evaluate("cc.all.length").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Integer(2));

    let (output, messages) = kernel
        .evaluate("({language: cc.first.language, code: cc.first.code})")
        .await?;
    assert_eq!(messages, []);
    assert_eq!(
        serde_json::to_string(&output)?,
        r#"{"language":"python","code":"1 + 2"}"#
    );

    let (output, messages) = kernel.evaluate("cc.first.outputs[0].value").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Integer(3));

    let (output, messages) = kernel.evaluate("cc.last.code").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("code 2".into()));

    let (output, messages) = kernel.evaluate("cc.previous").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Null(Null));

    let (output, messages) = kernel.evaluate("cc.current").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Null(Null));

    let (output, messages) = kernel.evaluate("cc.next.code").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("1 + 2".into()));

    let (.., messages) = kernel.execute("cc._forward()").await?;
    assert_eq!(messages, []);

    let (output, messages) = kernel.evaluate("cc.previous").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Null(Null));

    let (output, messages) = kernel.evaluate("cc.current.code").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("1 + 2".into()));

    let (output, messages) = kernel.evaluate("cc.next.code").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("code 2".into()));

    let (.., messages) = kernel.execute("cc._forward()").await?;
    assert_eq!(messages, []);

    let (output, messages) = kernel.evaluate("cc.previous.code").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("1 + 2".into()));

    let (output, messages) = kernel.evaluate("cc.current.code").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("code 2".into()));

    let (output, messages) = kernel.evaluate("cc.next").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Null(Null));

    Ok(())
}
