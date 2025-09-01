use kernel_micro::{
    Kernel, KernelAvailability, KernelInstance, KernelInterrupt, KernelKill, KernelProvider,
    KernelTerminate, Microkernel, eyre::Result, format::Format, schema::ExecutionBounds,
};

/// A kernel for executing Bash code locally
#[derive(Default)]
pub struct BashKernel;

const NAME: &str = "bash";

impl Kernel for BashKernel {
    fn name(&self) -> String {
        NAME.to_string()
    }

    fn provider(&self) -> KernelProvider {
        KernelProvider::Environment
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

    fn supported_bounds(&self) -> Vec<ExecutionBounds> {
        vec![ExecutionBounds::Main]
    }

    fn create_instance(&self, bounds: ExecutionBounds) -> Result<Box<dyn KernelInstance>> {
        self.microkernel_create_instance(NAME, bounds)
    }
}

impl Microkernel for BashKernel {
    fn executable_name(&self) -> String {
        "bash".to_string()
    }

    fn microkernel_script(&self) -> (String, String) {
        ("kernel.bash".into(), include_str!("kernel.bash").into())
    }
}

#[cfg(test)]
#[allow(clippy::print_stderr)]
mod tests {
    use common_dev::{ntest::timeout, pretty_assertions::assert_eq};
    use kernel_micro::{
        eyre::bail,
        schema::{MessageLevel, Node, Null, Variable},
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
            eprintln!("Skipping flakey test on CI");
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
                // Aliases defined in one chunk are available in the next
                (
                    "
alias greet='echo Hello'",
                    vec![],
                    vec![],
                ),
                (
                    "
greet World",
                    vec![Node::String("Hello World\n".to_string())],
                    vec![],
                ),
                // Functions defined in one chunk are available in the next
                (
                    "
greet_func() {
    echo \"Hello from function: $1\"
}
add_numbers() {
    echo $(($1 + $2))
}",
                    vec![],
                    vec![],
                ),
                (
                    "
greet_func World
add_numbers 5 3",
                    vec![Node::String("Hello from function: World\n8\n".to_string())],
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
            eprintln!("Skipping flakey test on CI");
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
            eprintln!("Skipping flakey test on CI");
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
    #[allow(clippy::print_stderr)]
    async fn messages() -> Result<()> {
        if std::env::var("CI").is_ok() {
            eprintln!("Skipping flakey test on CI");
            return Ok(());
        }

        let Some(mut instance) = start_instance::<BashKernel>().await? else {
            return Ok(());
        };

        let (outputs, messages) = instance.execute("if").await?;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].message, "syntax error: unexpected end of file");
        assert!(messages[0].code_location.is_some());
        let loc = messages[0].code_location.as_ref().expect("should be some");
        assert_eq!(loc.start_line, Some(0));
        assert_eq!(outputs, vec![]);

        let (outputs, messages) = instance.execute("foo").await?;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].message, "foo: command not found");
        assert_eq!(messages[0].level, MessageLevel::Error);

        // Check code location
        assert!(messages[0].code_location.is_some());
        let loc = messages[0].code_location.as_ref().expect("should be some");
        assert_eq!(loc.start_line, Some(0));
        assert_eq!(outputs, vec![]);

        // Test multi-line code with error on specific line
        let (outputs, messages) = instance.execute("echo line1\nFOO\necho line3").await?;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].message, "FOO: command not found");
        assert!(messages[0].code_location.is_some());
        let loc = messages[0].code_location.as_ref().expect("should be some");
        assert_eq!(loc.start_line, Some(1));
        // Should have output from echo commands
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0], Node::String("line1\nline3\n".to_string()));

        // Test error on last line of multi-line code
        let (outputs, messages) = instance.execute("echo start\necho middle\nBADCMD").await?;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].message, "BADCMD: command not found");
        assert!(messages[0].code_location.is_some());
        let loc = messages[0].code_location.as_ref().expect("should be some");
        assert_eq!(loc.start_line, Some(2));
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0], Node::String("start\nmiddle\n".to_string()));

        // Test multiple errors (bash stops on first error in non-background execution)
        let (outputs, messages) = instance
            .execute("BADCMD1 || echo recovered\necho hello")
            .await?;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].message, "BADCMD1: command not found");
        assert!(messages[0].code_location.is_some());
        let loc = messages[0].code_location.as_ref().expect("should be some");
        assert_eq!(loc.start_line, Some(0));
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0], Node::String("recovered\nhello\n".to_string()));

        // Test error in function or alias execution should also be clean
        let (outputs, messages) = instance.execute("alias badtest='NOTFOUND'").await?;
        assert_eq!(outputs, vec![]);
        assert_eq!(messages, vec![]);

        let (_outputs, messages) = instance.execute("badtest").await?;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].message, "NOTFOUND: command not found");
        // Note: errors from aliases might not have line numbers since they're expanded inline

        // Test stderr output with zero exit code should be Info level
        let (_outputs, messages) = instance.execute("echo 'a message' >&2").await?;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].message, "a message");
        assert_eq!(messages[0].level, MessageLevel::Info);

        // Test stderr output with non-zero exit code should be Error level
        let (_outputs, messages) = instance.execute("echo 'error message' >&2; exit 1").await?;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].message, "error message");
        assert_eq!(messages[0].level, MessageLevel::Error);

        // Test stderr output with successful command that produces stderr should be Info
        let (_outputs, messages) = instance
            .execute("ls /nonexistent 2>&1 || echo 'handled'")
            .await?;
        // This should succeed (exit 0) because of the || echo, so stderr should be Info
        if !messages.is_empty() {
            assert_eq!(messages[0].level, MessageLevel::Info);
        }

        // Test multiline stderr output
        let (_outputs, messages) = instance
            .execute("echo -e 'line 1\\nline 2\\nline 3' >&2")
            .await?;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].message, "line 1\nline 2\nline 3");
        assert_eq!(messages[0].level, MessageLevel::Info);

        // Test JSON escaping in stderr messages - quotes
        let (_outputs, messages) = instance
            .execute(r#"echo 'Test "quoted" message' >&2"#)
            .await?;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].message, r#"Test "quoted" message"#);

        // Test JSON escaping in stderr messages - backslashes
        let (_outputs, messages) = instance
            .execute(r#"echo 'Test \backslash\ message' >&2"#)
            .await?;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].message, r#"Test \backslash\ message"#);

        // Test complex multiline stderr with quotes and newlines
        let (_outputs, messages) = instance
            .execute(
                r#"echo -e 'Error: "something"\nDetails:\n  - item 1\n  - item 2' >&2; exit 1"#,
            )
            .await?;
        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0].message,
            "Error: \"something\"\nDetails:\n  - item 1\n  - item 2"
        );
        assert_eq!(messages[0].level, MessageLevel::Error);

        Ok(())
    }

    /// Standard kernel test for getting runtime information
    #[test_log::test(tokio::test)]
    async fn info() -> Result<()> {
        if std::env::var("CI").is_ok() {
            eprintln!("Skipping flakey test on CI");
            return Ok(());
        }

        let Some(instance) = create_instance::<BashKernel>().await? else {
            return Ok(());
        };

        let sw = kernel_micro::tests::info(instance).await?;
        assert_eq!(sw.name, "Bash");
        assert!(sw.options.software_version.is_some());
        assert!(sw.options.operating_system.is_some());

        Ok(())
    }

    /// Standard kernel test for listing installed packages
    #[test_log::test(tokio::test)]
    async fn packages() -> Result<()> {
        if std::env::var("CI").is_ok() {
            eprintln!("Skipping flakey test on CI");
            return Ok(());
        }

        let Some(instance) = start_instance::<BashKernel>().await? else {
            return Ok(());
        };

        let pkgs = kernel_micro::tests::packages(instance).await?;
        assert!(pkgs.is_empty());

        Ok(())
    }

    /// Standard kernel test for variable listing
    #[test_log::test(tokio::test)]
    async fn var_listing() -> Result<()> {
        if std::env::var("CI").is_ok() {
            eprintln!("Skipping flakey test on CI");
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
            eprintln!("Skipping flakey test on CI");
            return Ok(());
        }

        let Some(instance) = create_instance::<BashKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::var_management(instance).await
    }

    /// Standard kernel test for signals
    #[ignore = "unclear if test is failing due to set setup"]
    #[test_log::test(tokio::test)]
    #[timeout(5000)]
    async fn signals() -> Result<()> {
        if std::env::var("CI").is_ok() {
            eprintln!("Skipping flakey test on CI");
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
            eprintln!("Skipping flakey test on CI");
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
            eprintln!("Skipping flakey test on CI");
            return Ok(());
        }

        let Some(mut kernel) = start_instance::<BashKernel>().await? else {
            return Ok(());
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
