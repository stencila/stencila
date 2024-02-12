use std::sync::atomic::{AtomicU64, Ordering};

use kernel_micro::{
    common::eyre::Result, format::Format, Kernel, KernelAvailability, KernelForks, KernelInstance,
    KernelInterrupt, KernelKill, KernelTerminate, Microkernel,
};

/// A kernel for executing Bash code locally
#[derive(Default)]
pub struct BashKernel {
    /// A counter of instances of this microkernel
    instances: AtomicU64,
}

impl Kernel for BashKernel {
    fn id(&self) -> String {
        "bash".to_string()
    }

    fn availability(&self) -> KernelAvailability {
        self.microkernel_availability()
    }

    fn supports_languages(&self) -> Vec<Format> {
        vec![Format::Bash, Format::Shell]
    }

    fn supports_interrupt(&self) -> KernelInterrupt {
        self.microkernel_supports_interrupt()
    }

    fn supports_terminate(&self) -> KernelTerminate {
        self.microkernel_supports_terminate()
    }

    fn supports_kill(&self) -> KernelKill {
        self.microkernel_supports_kill()
    }

    fn supports_forks(&self) -> KernelForks {
        // Supported on all platforms where `bash` is present because uses background
        // process rather than Unix `fork`.
        KernelForks::Yes
    }

    fn create_instance(&self) -> Result<Box<dyn KernelInstance>> {
        self.microkernel_create_instance(self.instances.fetch_add(1, Ordering::SeqCst))
    }
}

impl Microkernel for BashKernel {
    fn executable_name(&self) -> String {
        "bash".to_string()
    }

    fn microkernel_script(&self) -> String {
        include_str!("kernel.bash").to_string()
    }
}

#[cfg(test)]
mod tests {
    use common_dev::{ntest::timeout, pretty_assertions::assert_eq};
    use kernel_micro::{
        common::{eyre::bail, tokio},
        schema::{Node, Null, Variable},
        tests::{create_instance, start_instance},
    };

    use super::*;

    // Pro-tip! Use get logs for these tests use:
    //
    // ```sh
    // RUST_LOG=trace cargo test -p kernel-bash -- --nocapture
    // ```

    // TODO: Remove skips of tests when flakiness is fixed
    // https://github.com/stencila/stencila/issues/2021

    /// Standard kernel test for execution of code
    #[test_log::test(tokio::test)]
    async fn execution() -> Result<()> {
        if std::env::var("CI").is_ok() {
            println!("Skipping flakey test on CI");
            return Ok(());
        }

        let Some(instance) = create_instance::<BashKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::execution(
            instance,
            vec![
                // Empty code: no outputs
                ("", vec![], vec![]),
                (" ", vec![], vec![]),
                ("\n\n", vec![], vec![]),
                // Prints: multiple, separate outputs
                (
                    "
print 1
print 2 3
echo 4",
                    vec![
                        Node::Integer(1),
                        Node::Integer(2),
                        Node::Integer(3),
                        Node::Integer(4),
                    ],
                    vec![],
                ),
                // Variables set in one chunk are available in the next
                (
                    "
declare a=1
declare b=2
export c=3",
                    vec![],
                    vec![],
                ),
                (
                    "
print $a $b $c",
                    vec![Node::Integer(1), Node::Integer(2), Node::Integer(3)],
                    vec![],
                ),
                // Comments are ignored
                (
                    "
# Comment
value=4 # Comment at line end
# Another comment
echo $value",
                    vec![Node::Integer(4)],
                    vec![],
                ),
            ],
        )
        .await
    }

    /// Standard kernel test for evaluation of expressions
    #[test_log::test(tokio::test)]
    async fn evaluation() -> Result<()> {
        if std::env::var("CI").is_ok() {
            println!("Skipping flakey test on CI");
            return Ok(());
        }

        let Some(instance) = create_instance::<BashKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::evaluation(
            instance,
            vec![
                // Bash kernel only supports simple integer expressions...
                ("1 + 1", Node::Integer(2), None),
                ("2 * 2", Node::Integer(4), None),
                ("16 / 2", Node::Integer(8), None),
                // ...and will return null on other expressions
                ("'a' + 'b'", Node::Null(Null), None),
            ],
        )
        .await
    }

    /// Standard kernel test for printing nodes
    #[test_log::test(tokio::test)]
    async fn printing() -> Result<()> {
        if std::env::var("CI").is_ok() {
            println!("Skipping flakey test on CI");
            return Ok(());
        }

        let Some(instance) = create_instance::<BashKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::printing(
            instance,
            r#"print str"#,
            r#"print str1 str2"#,
            r#"print null true 1 2.3 str '[1, 2.3, "str"]' '{"a":1, "b":2.3, "c":"str"}'"#,
            r#"print '{"type":"Paragraph", "content":[]}'"#,
        )
        .await
    }

    /// Custom test for execution messages
    #[test_log::test(tokio::test)]
    async fn messages() -> Result<()> {
        if std::env::var("CI").is_ok() {
            println!("Skipping flakey test on CI");
            return Ok(());
        }

        let Some(mut instance) = start_instance::<BashKernel>().await? else {
            return Ok(());
        };

        let (outputs, messages) = instance.execute("if").await?;
        assert_eq!(messages.len(), 1);
        assert!(messages[0]
            .message
            .ends_with("syntax error: unexpected end of file\n"));
        assert_eq!(outputs, vec![]);

        let (outputs, messages) = instance.execute("foo").await?;
        assert_eq!(messages.len(), 1);
        assert!(messages[0].message.ends_with("foo: command not found\n"));
        assert_eq!(outputs, vec![]);

        Ok(())
    }

    /// Standard kernel test for variable listing
    #[test_log::test(tokio::test)]
    async fn var_listing() -> Result<()> {
        if std::env::var("CI").is_ok() {
            println!("Skipping flakey test on CI");
            return Ok(());
        }

        let Some(instance) = create_instance::<BashKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::var_listing(
            instance,
            r#"
declare str="str"
declare -i int=1
declare -a arr=(1 2 3)
declare -A obj=(["key1"]="value1" ["key2"]="value2")
"#,
            vec![
                Variable {
                    name: "str".to_string(),
                    native_type: Some("string".to_string()),
                    node_type: Some("String".to_string()),
                    programming_language: Some("Bash".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "int".to_string(),
                    native_type: Some("integer".to_string()),
                    node_type: Some("Integer".to_string()),
                    programming_language: Some("Bash".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "arr".to_string(),
                    native_type: Some("array".to_string()),
                    node_type: Some("Array".to_string()),
                    programming_language: Some("Bash".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "obj".to_string(),
                    native_type: Some("associative array".to_string()),
                    node_type: Some("Object".to_string()),
                    programming_language: Some("Bash".to_string()),
                    ..Default::default()
                },
            ],
        )
        .await
    }

    /// Standard kernel test for variable management
    #[test_log::test(tokio::test)]
    async fn var_management() -> Result<()> {
        if std::env::var("CI").is_ok() {
            println!("Skipping flakey test on CI");
            return Ok(());
        }

        let Some(instance) = create_instance::<BashKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::var_management(instance).await
    }

    /// Standard kernel test for forking
    #[test_log::test(tokio::test)]
    async fn forking() -> Result<()> {
        if std::env::var("CI").is_ok() {
            println!("Skipping flakey test on CI");
            return Ok(());
        }

        let Some(instance) = create_instance::<BashKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::forking(instance).await
    }

    /// Standard kernel test for signals
    #[test_log::test(tokio::test)]
    #[timeout(5000)]
    async fn signals() -> Result<()> {
        if std::env::var("CI").is_ok() {
            println!("Skipping flakey test on CI");
            return Ok(());
        }

        let Some(instance) = create_instance::<BashKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::signals(
            instance,
            "
# Setup step
sleep 0.1
value=1
echo $value",
            Some(
                "
# Interrupt step (can't attempt to assign because that will cause
# this to run in main, uninterruptible bash process)
sleep 100",
            ),
            None,
            None,
        )
        .await
    }

    /// Standard kernel test for stopping
    #[test_log::test(tokio::test)]
    async fn stop() -> Result<()> {
        if std::env::var("CI").is_ok() {
            println!("Skipping flakey test on CI");
            return Ok(());
        }

        let Some(instance) = create_instance::<BashKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::stop(instance).await
    }

    /// `BashKernel` specific test of execution tasks that may involve additional escaping
    #[tokio::test]
    async fn escaping() -> Result<()> {
        if std::env::var("CI").is_ok() {
            println!("Skipping flakey test on CI");
            return Ok(());
        }

        let Some(mut kernel) = start_instance::<BashKernel>().await? else {
            return Ok(())
        };

        // Test escaping of percent signs in commands
        let (outputs, messages) = kernel.execute("date +%s").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs.len(), 1);

        match outputs.first() {
            Some(Node::Integer(timestamp)) => assert!(*timestamp > 1600000000),
            _ => bail!("Expected an integer output"),
        }

        Ok(())
    }
}
