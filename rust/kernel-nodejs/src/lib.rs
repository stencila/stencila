use std::{
    fs::{read_to_string, write},
    path::Path,
    process::{Command, Stdio},
};

use kernel_micro::{
    common::{eyre::Result, serde::Deserialize, serde_json, tempfile, tracing},
    format::Format,
    schema::{
        AuthorRole, AuthorRoleName, CodeLocation, CompilationMessage, MessageLevel,
        SoftwareApplication, Timestamp,
    },
    Kernel, KernelAvailability, KernelForks, KernelInstance, KernelInterrupt, KernelKill,
    KernelLint, KernelLinting, KernelLintingOptions, KernelLintingOutput, KernelProvider,
    KernelTerminate, Microkernel,
};

/// A kernel for executing JavaScript code in Node.js
#[derive(Default)]
pub struct NodeJsKernel;

const NAME: &str = "nodejs";

impl Kernel for NodeJsKernel {
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
        vec![Format::JavaScript]
    }

    #[allow(unreachable_code)]
    fn supports_linting(&self) -> KernelLinting {
        // Note: NodeJS linting is preliminary and can be slow so is
        // currently disabled
        return KernelLinting::No;

        let format = Command::new("npx")
            .arg("--no") // Do not install prettier if not already
            .arg("--")
            .arg("prettier")
            .arg("--version") // Smaller output than without
            .stdout(Stdio::null())
            .status()
            .map_or(false, |status| status.success());

        let fix = Command::new("npx")
            .arg("--no") // Do not install eslint if not already
            .arg("--")
            .arg("eslint")
            .arg("--version") // To prevent eslint waiting for input
            .stdout(Stdio::null())
            .status()
            .map_or(false, |status| status.success());

        KernelLinting::new(format, fix, fix)
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
        // Supported on all platforms because uses Node.js `child_process.fork`
        // rather than Unix `fork`.
        KernelForks::Yes
    }

    fn create_instance(&self) -> Result<Box<dyn KernelInstance>> {
        self.microkernel_create_instance(NAME)
    }
}

impl KernelLint for NodeJsKernel {
    #[tracing::instrument(skip(self, code))]
    async fn lint(
        &self,
        code: &str,
        _dir: &Path,
        options: KernelLintingOptions,
    ) -> Result<KernelLintingOutput> {
        tracing::trace!("Linting Node.js code");

        // It is difficult (impossible?) to get eslint to work on an out
        // of tree file (e.g. in /tmp) so this creates one next to the
        // current document.
        let temp_dir = tempfile::Builder::new()
            .prefix("stencila-lint-")
            .tempdir_in(".")?;

        // Write the code to a temporary file
        let code_path = temp_dir.path().join("code.js");
        write(&code_path, code)?;
        let code_path_str = code_path.to_string_lossy();

        let mut authors: Vec<AuthorRole> = Vec::new();

        // Format code if specified
        // Does this optimistically with no error if it fails
        if options.format {
            let result = Command::new("npx")
                .arg("--no") // Do not install prettier if not already
                .arg("--")
                .arg("prettier")
                .arg("--write")
                .arg(&*code_path_str)
                .output();

            if let Ok(output) = result {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                if !stdout.contains("unchanged") {
                    // Successfully ran Prettier, and it made changes, so add as an author
                    authors.push(
                        SoftwareApplication::new("Prettier".to_string()).into_author_role(
                            AuthorRoleName::Formatter,
                            Some(Format::JavaScript),
                            Some(Timestamp::now()),
                        ),
                    );
                }
            }
        }

        // Need a config file
        // TODO: walk up the tree to find any existing config
        let config_path = temp_dir.path().join("eslint.config.mjs");
        write(
            &config_path,
            r#"
import pluginJs from "@eslint/js";
export default [
    pluginJs.configs.recommended,
];
"#,
        )?;
        let config_path_str = config_path.to_string_lossy();

        // Run eslint to get diagnostics and output as JSON
        let mut cmd = Command::new("npx");
        cmd.arg("eslint")
            .arg("--format=json")
            .arg("--config")
            .arg(&*config_path_str)
            .arg(&*code_path_str);

        if options.fix {
            cmd.arg("--fix");
        }

        // Run ESLint with JSON output to parse into messages
        let messages = if let Ok(output) = cmd.output() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();

            // Successfully ran ESLint so add as an author (regardless of whether it made any fixes)
            authors.push(
                SoftwareApplication::new("ESLint".to_string()).into_author_role(
                    AuthorRoleName::Linter,
                    Some(Format::JavaScript),
                    Some(Timestamp::now()),
                ),
            );

            // A diagnostic report from ESlint
            #[derive(Deserialize)]
            #[serde(crate = "kernel_micro::common::serde")]
            struct EslintReport {
                messages: Vec<EslintMessage>,
            }
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase", crate = "kernel_micro::common::serde")]
            struct EslintMessage {
                severity: u8,
                rule_id: Option<String>,
                message: String,
                line: u64,
                column: u64,
                end_line: Option<u64>,
                end_column: Option<u64>,
            }

            let mut eslint_reports = serde_json::from_str::<Vec<EslintReport>>(&stdout)?;
            if eslint_reports.is_empty() {
                None
            } else {
                let messages = eslint_reports
                    .swap_remove(0)
                    .messages
                    .into_iter()
                    .map(|msg| CompilationMessage {
                        error_type: Some("Linting".into()),
                        level: match msg.severity {
                            1 => MessageLevel::Warning,
                            _ => MessageLevel::Error,
                        },
                        message: format!(
                            "{}{}",
                            msg.message,
                            msg.rule_id
                                .map(|rule| format!(" (ESLint {rule})"))
                                .unwrap_or_default()
                        ),
                        code_location: Some(CodeLocation {
                            start_line: Some(msg.line.saturating_sub(1)),
                            start_column: Some(msg.column.saturating_sub(1)),
                            end_line: msg.end_line.map(|line| line.saturating_sub(1)),
                            end_column: msg.end_column.map(|col| col.saturating_sub(1)),
                            ..Default::default()
                        }),
                        ..Default::default()
                    })
                    .collect();

                Some(messages)
            }
        } else {
            None
        };

        // If formatted or fixed, read in the possible changed code
        let code = if options.format || options.fix {
            let new_code = read_to_string(code_path)?;
            (new_code != code).then_some(new_code)
        } else {
            None
        };

        Ok(KernelLintingOutput {
            code,
            messages,
            authors: (!authors.is_empty()).then_some(authors),
            ..Default::default()
        })
    }
}

impl Microkernel for NodeJsKernel {
    fn executable_name(&self) -> String {
        "node".to_string()
    }

    fn microkernel_script(&self) -> (String, String) {
        ("kernel.js".into(), include_str!("kernel.js").into())
    }
}

#[cfg(test)]
mod tests {
    use common_dev::pretty_assertions::assert_eq;
    use kernel_micro::{
        common::{indexmap::IndexMap, tokio},
        schema::{
            Array, ArrayHint, CodeLocation, ExecutionMessage, Hint, MessageLevel, Node, Null,
            Object, ObjectHint, Primitive, StringHint, Variable,
        },
        tests::{create_instance, start_instance},
    };

    use super::*;

    // Pro-tip! Use get logs for these tests use:
    //
    // ```sh
    // RUST_LOG=trace cargo test -p kernel-node -- --nocapture
    // ```

    /// Standard kernel test for execution of code
    #[test_log::test(tokio::test)]
    async fn execution() -> Result<()> {
        let Some(instance) = create_instance::<NodeJsKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::execution(
            instance,
            vec![
                // Empty code: no outputs
                ("", vec![], vec![]),
                (" ", vec![], vec![]),
                ("\n\n", vec![], vec![]),
                // Only an expression: one output
                (
                    "
1 + 1",
                    vec![Node::Integer(2)],
                    vec![],
                ),
                // Prints and an expression: multiple, separate outputs
                (
                    "
console.log(1);
console.log(2, 3);
2 + 2",
                    vec![
                        Node::Integer(1),
                        Node::Integer(2),
                        Node::Integer(3),
                        Node::Integer(4),
                    ],
                    vec![],
                ),
                // Packages imported in one code chunk are available in the next
                (
                    "
const fs = require('fs');",
                    vec![],
                    vec![],
                ),
                (
                    "
typeof fs",
                    vec![Node::String("object".to_string())],
                    vec![],
                ),
                // Variables set in one chunk are available in the next
                (
                    "
a = 1;
var b = 2;
let c = 3;
const d = 4;",
                    vec![Node::Integer(1)], // Somewhat surprisingly, `a` gets returned here
                    vec![],
                ),
                (
                    "
console.log(a, b, c, d)",
                    vec![
                        Node::Integer(1),
                        Node::Integer(2),
                        Node::Integer(3),
                        Node::Integer(4),
                    ],
                    vec![],
                ),
            ],
        )
        .await
    }

    /// Standard kernel test for evaluation of expressions
    #[test_log::test(tokio::test)]
    async fn evaluation() -> Result<()> {
        let Some(instance) = create_instance::<NodeJsKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::evaluation(
            instance,
            vec![
                ("1 + 1", Node::Integer(2), None),
                ("2.0 * 2.2", Node::Number(4.4), None),
                ("Math.sqrt(16)", Node::Integer(4), None),
                ("'a' + 'bc'", Node::String("abc".to_string()), None),
                ("'ABC'.toLowerCase()", Node::String("abc".to_string()), None),
                (
                    "[...[1, 2], 3]",
                    Node::Array(Array(vec![
                        Primitive::Integer(1),
                        Primitive::Integer(2),
                        Primitive::Integer(3),
                    ])),
                    None,
                ),
                (
                    "({...{a: 1}, ['b']: 2.3})",
                    Node::Object(Object(IndexMap::from([
                        (String::from("a"), Primitive::Integer(1)),
                        (String::from("b"), Primitive::Number(2.3)),
                    ]))),
                    None,
                ),
                ("", Node::Null(Null), None),
                ("@", Node::Null(Null), Some("Invalid or unexpected token")),
                ("foo", Node::Null(Null), Some("foo is not defined")),
            ],
        )
        .await
    }

    /// Standard kernel test for printing nodes
    #[test_log::test(tokio::test)]
    async fn printing() -> Result<()> {
        let Some(instance) = create_instance::<NodeJsKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::printing(
            instance,
            r#"console.log('str')"#,
            r#"console.log('str1', 'str2')"#,
            r#"console.log(null, true, 1, 2.3, 'str', [1, 2.3, 'str'], {a:1, b:2.3, c:'str'})"#,
            r#"console.log({type:'Paragraph', content:[]})"#,
        )
        .await
    }

    /// Custom test for execution messages
    #[tokio::test]
    async fn messages() -> Result<()> {
        let Some(mut kernel) = start_instance::<NodeJsKernel>().await? else {
            return Ok(());
        };

        // Syntax error
        let (outputs, messages) = kernel.execute("bad ^ # syntax").await?;
        assert_eq!(messages[0].error_type.as_deref(), Some("SyntaxError"));
        assert_eq!(messages[0].message, "Invalid or unexpected token");
        assert!(messages[0].stack_trace.is_some());
        assert_eq!(
            messages[0].code_location,
            Some(CodeLocation {
                start_line: Some(0),
                ..Default::default()
            })
        );
        assert_eq!(outputs, vec![]);

        // Runtime error
        let (outputs, messages) = kernel.execute("foo").await?;
        assert_eq!(messages[0].error_type.as_deref(), Some("ReferenceError"));
        assert_eq!(messages[0].message, "foo is not defined");
        assert!(messages[0].stack_trace.is_some());
        assert_eq!(
            messages[0].code_location,
            Some(CodeLocation {
                start_line: Some(0),
                start_column: Some(0),
                ..Default::default()
            })
        );
        assert_eq!(outputs, vec![]);

        // Nested
        let (outputs, messages) = kernel
            .execute(
                r#"
function foo() {
    bar()
}
foo()
"#,
            )
            .await?;
        assert_eq!(messages[0].error_type.as_deref(), Some("ReferenceError"));
        assert_eq!(messages[0].message, "bar is not defined");
        assert_eq!(
            messages[0].stack_trace.as_deref(),
            Some(
                r#"code:3
    bar()
    ^

ReferenceError: bar is not defined
    at foo (code:3:5)
    at code:5:1
"#
            )
        );
        assert_eq!(
            messages[0].code_location,
            Some(CodeLocation {
                start_line: Some(2),
                start_column: Some(4),
                ..Default::default()
            })
        );
        assert_eq!(outputs, vec![]);

        let (outputs, messages) = kernel.execute("const a = 1\na + bar").await?;
        assert_eq!(messages[0].error_type.as_deref(), Some("ReferenceError"));
        assert_eq!(messages[0].message, "bar is not defined");
        assert!(messages[0].stack_trace.is_some());
        assert_eq!(
            messages[0].code_location,
            Some(CodeLocation {
                start_line: Some(1),
                start_column: Some(4),
                ..Default::default()
            })
        );
        assert_eq!(outputs, vec![]);

        // Console methods
        let (.., messages) = kernel
            .execute(
                r#"
console.debug("Debug message");
console.info("Info message");
console.warn("Warning message");
console.error("Error message");
"#,
            )
            .await?;

        assert_eq!(
            messages,
            vec![
                ExecutionMessage {
                    level: MessageLevel::Debug,
                    message: "Debug message".to_string(),
                    ..Default::default()
                },
                ExecutionMessage {
                    level: MessageLevel::Info,
                    message: "Info message".to_string(),
                    ..Default::default()
                },
                ExecutionMessage {
                    level: MessageLevel::Warning,
                    message: "Warning message".to_string(),
                    ..Default::default()
                },
                ExecutionMessage {
                    level: MessageLevel::Error,
                    message: "Error message".to_string(),
                    ..Default::default()
                }
            ]
        );

        Ok(())
    }

    /// Standard kernel test for getting runtime information
    #[test_log::test(tokio::test)]
    async fn info() -> Result<()> {
        let Some(instance) = create_instance::<NodeJsKernel>().await? else {
            return Ok(());
        };

        let sw = kernel_micro::tests::info(instance).await?;
        assert_eq!(sw.name, "Node.js");
        assert!(sw.options.software_version.is_some());
        assert!(sw.options.operating_system.is_some());

        Ok(())
    }

    /// Standard kernel test for listing installed packages
    #[test_log::test(tokio::test)]
    async fn packages() -> Result<()> {
        #[allow(clippy::print_stderr)]
        if std::env::var("CI").is_ok() {
            eprintln!("Skipping test on CI because requires `npm install` has been run (not always the case)");
            return Ok(());
        }

        let Some(instance) = start_instance::<NodeJsKernel>().await? else {
            return Ok(());
        };

        let pkgs = kernel_micro::tests::packages(instance).await?;
        assert!(!pkgs.is_empty());

        Ok(())
    }

    /// Standard kernel test for variable listing
    #[cfg(not(target_os = "windows"))] // TODO: Fix on windows
    #[test_log::test(tokio::test)]
    async fn var_listing() -> Result<()> {
        let Some(instance) = create_instance::<NodeJsKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::var_listing(
            instance,
            r#"
var nul = null;
var bool = true;
var int = 123n;
var num = 1.23;
var str = "abc👍";
var arr = [1, 2, 3];
var obj = {a:1, b:2.3};
var para = {type: "Paragraph", content:[]}
"#,
            vec![
                Variable {
                    name: "nul".to_string(),
                    native_type: Some("null".to_string()),
                    node_type: Some("Null".to_string()),
                    programming_language: Some("JavaScript".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "bool".to_string(),
                    native_type: Some("boolean".to_string()),
                    node_type: Some("Boolean".to_string()),
                    hint: Some(Hint::Boolean(true)),
                    programming_language: Some("JavaScript".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "int".to_string(),
                    native_type: Some("bigint".to_string()),
                    node_type: Some("Integer".to_string()),
                    programming_language: Some("JavaScript".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "num".to_string(),
                    native_type: Some("number".to_string()),
                    node_type: Some("Number".to_string()),
                    hint: Some(Hint::Number(1.23)),
                    programming_language: Some("JavaScript".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "str".to_string(),
                    native_type: Some("string".to_string()),
                    node_type: Some("String".to_string()),
                    hint: Some(Hint::StringHint(StringHint::new(4))),
                    programming_language: Some("JavaScript".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "arr".to_string(),
                    native_type: Some("array".to_string()),
                    node_type: Some("Array".to_string()),
                    hint: Some(Hint::ArrayHint(ArrayHint::new(3))),
                    programming_language: Some("JavaScript".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "obj".to_string(),
                    native_type: Some("object".to_string()),
                    node_type: Some("Object".to_string()),
                    hint: Some(Hint::ObjectHint(ObjectHint::new(
                        2,
                        vec!["a".to_string(), "b".to_string()],
                        vec![Hint::Integer(1), Hint::Number(2.3)],
                    ))),
                    programming_language: Some("JavaScript".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "para".to_string(),
                    native_type: Some("object".to_string()),
                    node_type: Some("Paragraph".to_string()),
                    programming_language: Some("JavaScript".to_string()),
                    ..Default::default()
                },
            ],
        )
        .await
    }

    /// Standard kernel test for variable management
    #[cfg(not(target_os = "windows"))] // TODO: Fix on windows
    #[test_log::test(tokio::test)]
    async fn var_management() -> Result<()> {
        let Some(instance) = create_instance::<NodeJsKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::var_management(instance).await
    }

    /// Standard kernel test for forking
    #[cfg(not(target_os = "windows"))]
    #[test_log::test(tokio::test)]
    async fn forking() -> Result<()> {
        let Some(instance) = create_instance::<NodeJsKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::forking(instance).await
    }

    /// Custom test to check that modules imported in the main kernel instance are
    /// available in the forked instance
    #[cfg(not(target_os = "windows"))]
    #[test_log::test(tokio::test)]
    async fn forking_imports() -> Result<()> {
        let Some(mut instance) = start_instance::<NodeJsKernel>().await? else {
            return Ok(());
        };

        let (outputs, messages) = instance
            .execute(
                r#"
const fs = require("fs");
let path = require("path");
var crypto = require("crypto");

console.log(typeof fs.read, typeof path.join, typeof crypto.createCipher)
"#,
            )
            .await?;
        assert_eq!(messages, vec![]);
        assert_eq!(
            outputs,
            vec![
                Node::String("function".to_string()),
                Node::String("function".to_string()),
                Node::String("function".to_string())
            ]
        );

        let mut fork = instance.fork().await?;
        let (outputs, messages) = fork
            .execute(
                r#"
console.log(typeof fs.read, typeof path.join, typeof crypto.createCipher)
"#,
            )
            .await?;
        assert_eq!(messages, vec![]);
        assert_eq!(
            outputs,
            vec![
                Node::String("function".to_string()),
                Node::String("function".to_string()),
                Node::String("function".to_string())
            ]
        );

        Ok(())
    }

    /// Standard kernel test for signals
    #[cfg(not(target_os = "windows"))]
    #[test_log::test(tokio::test)]
    async fn signals() -> Result<()> {
        let Some(instance) = create_instance::<NodeJsKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::signals(
            instance,
            "
// Setup step

// A crude sleep function which can be called at top level without using await
function sleep(seconds) {
    const startTime = new Date().getTime();
    let currentTime = null;
    do {
        currentTime = new Date().getTime();
    } while (currentTime - startTime < seconds * 1000);
}

sleep(0.1);
value = 1;
value",
            Some(
                "
// Interrupt step
sleep(100);
value = 2;",
            ),
            None,
            Some(
                "
// Kill step
sleep(100);",
            ),
        )
        .await
    }

    /// Standard kernel test for stopping
    #[test_log::test(tokio::test)]
    async fn stop() -> Result<()> {
        let Some(instance) = create_instance::<NodeJsKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::stop(instance).await
    }

    /// `NodeKernel` specific test for re-declarations of variables
    ///
    /// The `kernel.js` has special handling of `var` and `let` so that code chunks
    /// that use these to declare variables can be re-executed without error.
    #[tokio::test]
    async fn redeclarations() -> Result<()> {
        let Some(mut kernel) = start_instance::<NodeJsKernel>().await? else {
            return Ok(());
        };

        // A variable declared with `var`

        let (outputs, messages) = kernel.execute("var a = 1\na").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs[0], Node::Integer(1));

        let (outputs, messages) = kernel.execute("var a = 2\na").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs[0], Node::Integer(2));

        let (outputs, messages) = kernel.execute("let a = 3\na").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs[0], Node::Integer(3));

        let (outputs, messages) = kernel.execute("const a = 4\na").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs[0], Node::Integer(4));

        // A variable declared with `let`

        let (outputs, messages) = kernel.execute("let b = 1\nb").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs[0], Node::Integer(1));

        let (outputs, messages) = kernel.execute("let b = 2\nb").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs[0], Node::Integer(2));

        let (outputs, messages) = kernel.execute("b = 3\nb").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs[0], Node::Integer(3));

        // A variable declared with `const`

        let (outputs, messages) = kernel.execute("const c = 1\nc").await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs[0], Node::Integer(1));

        let (.., messages) = kernel.execute("const c = 2\nc").await?;
        assert_eq!(messages[0].message, "Assignment to constant variable.");

        let (.., messages) = kernel.execute("c = 3\nc").await?;
        assert_eq!(messages[0].message, "Assignment to constant variable.");

        Ok(())
    }
}
