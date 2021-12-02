use kernel::eyre::Result;
use kernel_micro::{include_file, MicroKernel};

pub async fn new() -> Result<MicroKernel> {
    MicroKernel::new(
        "python",
        ("python3", "*", &["{{script}}"]),
        include_file!("python-kernel.py"),
        &[],
        "{{name}} = \"{{json}}\"",
        "{{name}}",
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

        let result = kernel.exec("6 * 7").await?;
        assert_json_eq!(result.0, [42]);

        Ok(())
    }
}
