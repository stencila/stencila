use crate::{prelude::*, PromptContext};

use common_dev::pretty_assertions::assert_eq;
use kernel_quickjs::kernel::common::tokio;
use schema::Node;

use super::{
    variable::{Hint, Variable},
    Kernel, Kernels, Package,
};

#[tokio::test]
async fn info() -> Result<()> {
    let context = PromptContext {
        kernels: Some(Kernels::new(vec![
            Kernel::new("Python", "3.12.1"),
            Kernel::new("R", "3.1.0"),
        ])),
        ..Default::default()
    };

    let mut kernel = context.into_kernel().await?;
    kernel.execute("const ks = kernels").await?;

    let (output, messages) = kernel.evaluate("ks.count").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Integer(2));

    let (output, messages) = kernel.evaluate("ks.all.length").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Integer(2));

    let (output, messages) = kernel.evaluate("ks.all").await?;
    assert_eq!(messages, []);
    assert_eq!(
        serde_json::to_string_pretty(&output)?,
        r#"[
  {
    "name": "Python",
    "version": "3.12.1",
    "packages": [],
    "variables": []
  },
  {
    "name": "R",
    "version": "3.1.0",
    "packages": [],
    "variables": []
  }
]"#
    );

    let (output, messages) = kernel.evaluate("ks.all[0].name").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("Python".into()));

    let (output, messages) = kernel.evaluate("ks.all.pop().name").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("R".into()));

    let (output, messages) = kernel.evaluate("ks.find('python').version").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("3.12.1".into()));

    Ok(())
}

#[tokio::test]
async fn packages() -> Result<()> {
    let context = PromptContext {
        kernels: Some(Kernels::new(vec![Kernel {
            packages: vec![
                Package::new("pandas", "1.2.3"),
                Package::new("pytorch", "4.5.6"),
            ],
            ..Default::default()
        }])),
        ..Default::default()
    };

    let mut kernel = context.into_kernel().await?;
    kernel.execute("const ps = kernels.first.packages").await?;

    let (output, messages) = kernel.evaluate("ps.length").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Integer(2));

    let (output, messages) = kernel
        .evaluate("ps.map(p => `${p.name}@${p.version}`).join(', ')")
        .await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("pandas@1.2.3, pytorch@4.5.6".into()));

    let (output, messages) = kernel.evaluate("ps").await?;
    assert_eq!(messages, []);
    assert_eq!(
        serde_json::to_string_pretty(&output)?,
        r#"[
  {
    "name": "pandas",
    "version": "1.2.3"
  },
  {
    "name": "pytorch",
    "version": "4.5.6"
  }
]"#
    );

    Ok(())
}

#[tokio::test]
async fn variable() -> Result<()> {
    let context = PromptContext {
        kernels: Some(Kernels::new(vec![Kernel {
            variables: vec![
                Variable {
                    name: "var1".into(),
                    r#type: Some("Integer".into()),
                    hint: Some(Hint::Integer(42)),
                    ..Default::default()
                },
                Variable {
                    name: "var2".into(),
                    r#type: Some("Array".into()),
                    hint: Some(Hint::Array {
                        length: 123,
                        types: None,
                        minimum: Some(1.),
                        maximum: Some(100.),
                        nulls: Some(23),
                    }),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }])),
        ..Default::default()
    };

    let mut kernel = context.into_kernel().await?;
    kernel
        .execute("const vars = kernels.first.variables")
        .await?;

    let (output, messages) = kernel.evaluate("vars.length").await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::Integer(2));

    let (output, messages) = kernel
        .evaluate("vars.map(v => `${v.name} ${v.type} ${v.hint?.length}`).join(', ')")
        .await?;
    assert_eq!(messages, []);
    assert_eq!(
        output,
        Node::String("var1 Integer 1, var2 Array 123".into())
    );

    let (output, messages) = kernel
        .evaluate(
            "vars.filter(v => v.name === 'var2').map(v => `${v.hint.minimum} ${v.hint.maximum} ${v.hint.nulls}`)[0]",
        )
        .await?;
    assert_eq!(messages, []);
    assert_eq!(output, Node::String("1 100 23".into()));

    let (output, messages) = kernel.evaluate("vars").await?;
    assert_eq!(messages, []);
    assert_eq!(
        serde_json::to_string_pretty(&output)?,
        r#"[
  {
    "name": "var1",
    "type": "Integer"
  },
  {
    "name": "var2",
    "type": "Array"
  }
]"#
    );

    Ok(())
}
