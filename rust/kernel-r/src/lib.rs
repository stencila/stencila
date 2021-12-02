use kernel::eyre::Result;
use kernel_micro::{include_file, MicroKernel};

pub async fn new() -> Result<MicroKernel> {
    MicroKernel::new(
        "r",
        ("Rscript", "*", &["{{script}}"]),
        include_file!("r-kernel.r"),
        &[],
        "{{name}} <- \"{{json}}\"",
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

        let (outputs, messages) = kernel.exec("6 * 7").await?;
        assert_json_eq!(outputs, [42]);
        assert!(messages.is_empty());

        Ok(())
    }
}
