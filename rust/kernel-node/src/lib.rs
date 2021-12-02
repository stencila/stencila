use kernel::eyre::Result;
use kernel_micro::{include_file, MicroKernel};

pub async fn new() -> Result<MicroKernel> {
    MicroKernel::new(
        "javascript",
        ("node", "*", &["{{script}}"]),
        include_file!("node-kernel.js"),
        &[include_file!("node-codec.js")],
        "{{name}} = decodeValue('{{json}}')",
        "{{name}}",
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::{stencila_schema::Node, KernelTrait};
    use test_utils::{assert_json_eq, print_logs};

    #[tokio::test]
    async fn all() -> Result<()> {
        print_logs();

        let mut kernel = new().await?;
        kernel.start().await?;

        // exec

        let (outputs, messages) = kernel.exec("const a = 1\na").await?;
        assert_json_eq!(outputs, [1]);
        assert!(messages.is_empty());

        let (outputs, messages) = kernel.exec("console.log(a*2)\na*3").await?;
        assert_json_eq!(outputs, [2, 3]);
        assert!(messages.is_empty());

        let (outputs, messages) = kernel.exec("bad ^ # syntax").await?;
        assert!(outputs.is_empty());
        assert_json_eq!(messages[0].error_type, "SyntaxError");
        assert_json_eq!(messages[0].error_message, "Invalid or unexpected token");

        let (outputs, messages) = kernel.exec("foo").await?;
        assert!(outputs.is_empty());
        assert_json_eq!(messages[0].error_type, "ReferenceError");
        assert_json_eq!(messages[0].error_message, "foo is not defined");

        // set & get

        kernel.set("b", Node::Integer(2)).await?;

        let b = kernel.get("b").await?;
        assert_json_eq!(b, 2);

        let (outputs, messages) = kernel.exec("a*b").await?;
        assert_json_eq!(outputs, [2]);
        assert!(messages.is_empty());

        Ok(())
    }
}
