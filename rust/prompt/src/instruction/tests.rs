use crate::{prelude::*, PromptContext};

use common_dev::pretty_assertions::assert_eq;
use kernel_quickjs::kernel::common::tokio;
use schema::Node;

use super::Instruction;

#[tokio::test]
async fn instruction() -> Result<()> {
    let context = PromptContext {
        instruction: Some(Instruction {
            r#type: "New".into(),
            message: Some("paragraph".into()),
            content: Some("content".into()),
        }),
        ..Default::default()
    };

    let mut kernel = context.into_kernel().await?;
    kernel.execute("const ins = instruction").await?;

    let (output, messages) = kernel.evaluate("instruction").await?;
    assert_eq!(messages, []);
    assert_eq!(
        serde_json::to_string_pretty(&output)?,
        r#"{
  "type": "New",
  "message": "paragraph",
  "content": "content"
}"#
    );

    let (output, messages) = kernel
        .evaluate("`${ins.type} ${ins.message} ${ins.content}`")
        .await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("New paragraph content".into()));

    Ok(())
}
