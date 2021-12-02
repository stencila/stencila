use kernel::eyre::Result;
use kernel_micro::{include_file, MicroKernel};

pub async fn new() -> Result<MicroKernel> {
    MicroKernel::new(
        "bash",
        ("bash", "*", &["{{script}}"]),
        include_file!("bash-kernel.sh"),
        &[],
        "{{name}}=\"{{json}}\"",
        "echo ${{name}}",
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::KernelTrait;
    use test_utils::{assert_json_eq, print_logs};

    #[tokio::test]
    async fn basic() -> Result<()> {
        print_logs();

        let mut kernel = new().await?;
        kernel.start().await?;

        let result = kernel.exec("echo foo").await?;
        assert_json_eq!(result.0, ["foo\n"]);

        Ok(())
    }
}
