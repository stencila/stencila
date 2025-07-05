use std::{cmp::Ordering, env, fs::read_to_string, io::Write, path::Path};

use which::which;

use tools::ToolCommand;

use kernel_micro::{
    common::{eyre::Result, serde::Deserialize, serde_json, tempfile::NamedTempFile, tracing},
    format::Format,
    schema::{
        AuthorRole, AuthorRoleName, CodeLocation, CompilationMessage, ExecutionBounds,
        MessageLevel, SoftwareApplication, Timestamp,
    },
    Kernel, KernelAvailability, KernelInstance, KernelInterrupt, KernelKill, KernelLint,
    KernelLinting, KernelLintingOptions, KernelLintingOutput, KernelProvider, KernelTerminate,
    Microkernel,
};

/// A kernel for executing Python code
#[derive(Default)]
pub struct PythonKernel;

const NAME: &str = "python";

impl Kernel for PythonKernel {
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
        vec![Format::Python]
    }

    fn supports_linting(&self) -> KernelLinting {
        let ruff = which("ruff").is_ok();
        let pyright = which("pyright").is_ok();

        KernelLinting::new(ruff, ruff || pyright, ruff)
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
        let mut bounds = vec![ExecutionBounds::Main];

        // Fork & Box both use Python `os.fork()` which is only available on POSIX-based systems
        if cfg!(unix) {
            bounds.append(&mut vec![ExecutionBounds::Fork, ExecutionBounds::Box]);
        }

        bounds
    }

    fn create_instance(&self, bounds: ExecutionBounds) -> Result<Box<dyn KernelInstance>> {
        self.microkernel_create_instance(NAME, bounds)
    }
}

impl KernelLint for PythonKernel {
    #[tracing::instrument(skip(self, code))]
    async fn lint(
        &self,
        code: &str,
        dir: &Path,
        options: KernelLintingOptions,
    ) -> Result<KernelLintingOutput> {
        tracing::trace!("Linting Python code");

        // Write the code to a temporary file. Avoid temptation to add any import
        // before the code as that mucks up line numbers using for matching
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "{}", code)?;
        let temp_path = temp_file.path();

        let mut authors: Vec<AuthorRole> = Vec::new();

        // Format code if specified
        if options.format {
            let result = ToolCommand::new("ruff")
                .arg("format")
                .arg(temp_path)
                .output();

            if let Ok(output) = result {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                if stdout.contains("reformatted") {
                    // Successfully ran Ruff, and it made changes, so add as an author
                    authors.push(
                        SoftwareApplication::new("Ruff".to_string()).into_author_role(
                            AuthorRoleName::Formatter,
                            Some(Format::Python),
                            Some(Timestamp::now()),
                        ),
                    );
                }
            }
        }

        // Run Ruff with JSON output for parsing of diagnostic to messages
        let mut cmd = ToolCommand::new("ruff");
        cmd.arg("check").arg("--output-format=json").arg(temp_path);
        if options.fix {
            cmd.arg("--fix");
        }
        let mut messages: Vec<CompilationMessage> = if let Ok(output) = cmd.output() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();

            // Successfully ran Ruff so add as an author (regardless of whether it made any fixes)
            authors.push(
                SoftwareApplication::new("Ruff".to_string()).into_author_role(
                    AuthorRoleName::Linter,
                    Some(Format::Python),
                    Some(Timestamp::now()),
                ),
            );

            // A diagnostic message from Ruff
            #[derive(Deserialize)]
            #[serde(crate = "kernel_micro::common::serde")]
            struct RuffMessage {
                code: Option<String>,
                message: String,
                location: Option<RuffLocation>,
                end_location: Option<RuffLocation>,
            }
            #[derive(Deserialize)]
            #[serde(crate = "kernel_micro::common::serde")]
            struct RuffLocation {
                column: u64,
                row: u64,
            }

            let ruff_messages = serde_json::from_str::<Vec<RuffMessage>>(&stdout)?;
            ruff_messages
                .into_iter()
                .filter(|message| {
                    // Ignore some messages which make no sense when concatenating code chunks
                    // E402: Module level import not at top of file
                    !matches!(message.code.as_deref(), Some("E402"))
                })
                .map(|message| CompilationMessage {
                    error_type: Some("Linting warning".into()),
                    level: MessageLevel::Warning,
                    message: format!(
                        "{}{}",
                        message.message,
                        message
                            .code
                            .map(|code| format!(" (Ruff {code})"))
                            .unwrap_or_default()
                    ),
                    code_location: Some(CodeLocation {
                        // Note that Ruff provides 1-based row and column indices
                        start_line: message
                            .location
                            .as_ref()
                            .map(|location| location.row.saturating_sub(1)),
                        start_column: message
                            .location
                            .as_ref()
                            .map(|location| location.column.saturating_sub(1)),
                        end_line: message
                            .end_location
                            .as_ref()
                            .map(|location| location.row.saturating_sub(1)),
                        end_column: message
                            .end_location
                            .as_ref()
                            .map(|location| location.column.saturating_sub(1)),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .collect()
        } else {
            Vec::new()
        };

        // Run Pyright with JSON output to parse into messages
        // See https://github.com/Microsoft/pyright/blob/main/docs/command-line.md
        let mut pyright = ToolCommand::new("pyright");
        pyright.arg("--outputjson");
        if let Ok(python_path) = env::var("PYTHON_PATH") {
            // Use the PYTHON_PATH provided
            pyright.arg(format!("--pythonpath={python_path}"));
        } else {
            // Search up the tree from the document for a virtual env
            let mut dir = dir.to_path_buf();
            loop {
                let python_path = dir.join(".venv").join("bin").join("python");
                if python_path.exists() {
                    pyright.arg(format!("--pythonpath={}", python_path.display()));
                    break;
                }
                if !dir.pop() {
                    break;
                }
            }
        }
        if let Ok(output) = pyright.arg(temp_path).output() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();

            // Successfully ran Pyright so add as an author (regardless of whether it made any fixes)
            authors.push(
                SoftwareApplication::new("Pyright".to_string()).into_author_role(
                    AuthorRoleName::Linter,
                    Some(Format::Python),
                    Some(Timestamp::now()),
                ),
            );

            // A diagnostic report from Pyright
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase", crate = "kernel_micro::common::serde")]
            struct PyrightDiagnostics {
                general_diagnostics: Vec<PyrightDiagnostic>,
            }
            #[derive(Deserialize)]
            #[serde(crate = "kernel_micro::common::serde")]
            struct PyrightDiagnostic {
                rule: Option<String>,
                severity: String,
                message: String,
                range: PyrightRange,
            }
            #[derive(Deserialize)]
            #[serde(crate = "kernel_micro::common::serde")]
            struct PyrightRange {
                start: PyrightLocation,
                end: PyrightLocation,
            }
            #[derive(Deserialize)]
            #[serde(crate = "kernel_micro::common::serde")]
            struct PyrightLocation {
                line: u64,
                character: u64,
            }

            let pyright_diagnostics = serde_json::from_str::<PyrightDiagnostics>(&stdout)?;
            for diag in pyright_diagnostics
                .general_diagnostics
                .into_iter()
                .filter(|diag| {
                    // Ignore some diagnostics which do not make so much sense in code cells
                    !matches!(diag.rule.as_deref(), Some("reportUnusedExpression"))
                })
            {
                let code_location = Some(CodeLocation {
                    start_line: Some(diag.range.start.line),
                    start_column: Some(diag.range.start.character),
                    end_line: Some(diag.range.end.line),
                    end_column: Some(diag.range.end.character),
                    ..Default::default()
                });

                let level = match diag.severity.as_str() {
                    "warning" => MessageLevel::Warning,
                    _ => MessageLevel::Error,
                };

                // Only add message for a line if higher severity level
                if messages
                    .iter()
                    .any(|msg| msg.code_location == code_location && msg.level >= level)
                {
                    continue;
                }

                let message = format!(
                    "{}{}",
                    diag.message,
                    diag.rule
                        .map(|rule| format!(" (Pyright {})", rule.trim_start_matches("report")))
                        .unwrap_or_default()
                )
                .trim()
                .to_string();

                let message = CompilationMessage {
                    error_type: Some(format!("Linting {}", level.to_string().to_lowercase())),
                    level,
                    message,
                    code_location,
                    ..Default::default()
                };

                messages.push(message);
            }
        }

        // Sort messages by location (necessary because there may be more than one linter)
        messages.sort_by(|a, b| match (&a.code_location, &b.code_location) {
            (Some(a), Some(b)) => match (a.start_line, b.start_line) {
                (Some(a), Some(b)) => a.cmp(&b),
                (Some(..), None) => Ordering::Less,
                (None, Some(..)) => Ordering::Greater,
                (None, None) => Ordering::Equal,
            },
            (Some(..), None) => Ordering::Less,
            (None, Some(..)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        });

        // Read the updated file if formatted or fixed
        let code = if options.format || options.fix {
            let new_code = read_to_string(temp_path)?;
            (new_code != code).then_some(new_code)
        } else {
            None
        };

        Ok(KernelLintingOutput {
            code,
            messages: (!messages.is_empty()).then_some(messages),
            authors: (!authors.is_empty()).then_some(authors),
            ..Default::default()
        })
    }
}

impl Microkernel for PythonKernel {
    fn executable_name(&self) -> String {
        "python3".into()
    }

    fn executable_arguments(&self, _executable_name: &str) -> Vec<String> {
        vec!["{{script}}".into()]
    }

    fn microkernel_script(&self) -> (String, String) {
        ("kernel.py".into(), include_str!("kernel.py").into())
    }
}

// These tests fail on Windows CI with error
//   Error: When flushing code to kernel: The pipe is being closed. (os error 232)
// This is likely due to communication with the Python stdin/stdout pipes on Windows
// TODO: Fix Python microkernel on Windows
#[cfg(not(target_os = "windows"))]
#[cfg(test)]
#[allow(clippy::print_stderr, clippy::unwrap_used)]
mod tests {
    use common_dev::pretty_assertions::assert_eq;
    use kernel_micro::{
        common::{
            eyre::{bail, Ok},
            indexmap::IndexMap,
            tokio,
        },
        schema::{
            Array, ArrayHint, ArrayValidator, BooleanValidator, CodeLocation, Datatable,
            DatatableColumn, DatatableColumnHint, DatatableHint, Hint, ImageObject,
            IntegerValidator, MessageLevel, Node, Null, NumberValidator, Object, ObjectHint,
            Primitive, StringHint, StringValidator, Validator, Variable,
        },
        tests::{create_instance, start_instance, start_instance_with},
    };

    use super::*;

    // Pro-tip! Use get logs for these tests use:
    //
    // ```sh
    // RUST_LOG=trace cargo test -p kernel-python -- --nocapture
    // ```

    /// Standard kernel test for execution of code
    #[test_log::test(tokio::test)]
    async fn execution() -> Result<()> {
        let Some(instance) = create_instance::<PythonKernel>().await? else {
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
                // Only the last value is output; outputs can span lines
                (
                    "
1

sum([
  1,
  2
])
",
                    vec![Node::Integer(3)],
                    vec![],
                ),
                // Prints and an expression: multiple, separate outputs
                (
                    "
print(1)
print(2, 3)
2 + 2",
                    vec![
                        Node::Integer(1),
                        Node::Integer(2),
                        Node::Integer(3),
                        Node::Integer(4),
                    ],
                    vec![],
                ),
                // Imports in one code chunk are available in the next
                (
                    "
import sys
from sys import argv",
                    vec![],
                    vec![],
                ),
                (
                    "
print(type(sys), type(argv))",
                    vec![
                        Node::String("<class 'module'>".to_string()),
                        Node::String("<class 'list'>".to_string()),
                    ],
                    vec![],
                ),
                // Variables set in one chunk are available in the next
                (
                    "
a = 1
b = 2",
                    vec![],
                    vec![],
                ),
                (
                    "
print(a, b)",
                    vec![Node::Integer(1), Node::Integer(2)],
                    vec![],
                ),
            ],
        )
        .await
    }

    /// Custom test for indented code
    ///
    /// Regression test to ensure that if/elif blocks and function definitions are not prematurely executed.
    #[test_log::test(tokio::test)]
    async fn execution_indented() -> Result<()> {
        let Some(mut instance) = start_instance::<PythonKernel>().await? else {
            return Ok(());
        };

        let (outputs, messages) = instance
            .execute(
                r"
if False:
  x = 1
elif False:
  x = 2
else:
  x = 3

x",
            )
            .await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs, vec![Node::Integer(3)]);

        let (outputs, messages) = instance
            .execute(
                r"
x = 0 
for i in range(10):

  if i < 5:
    continue

  else:
    x = i
    break

x",
            )
            .await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs, vec![Node::Integer(5)]);

        let (outputs, messages) = instance
            .execute(
                r"
def func(x):
  '''With empty lines and
    blank lines'''

  
  return x * 7

func(
  1
)",
            )
            .await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs, vec![Node::Integer(7)]);

        let (outputs, messages) = instance
            .execute(
                r"
try:
    raise ValueError()
except NameError:
    print(2)
except ValueError as e:
    print(3)
finally:
    print(4)
",
            )
            .await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs, vec![Node::Integer(3), Node::Integer(4)]);

        Ok(())
    }

    /// Standard kernel test for evaluation of expressions
    #[test_log::test(tokio::test)]
    async fn evaluation() -> Result<()> {
        let Some(mut instance) = create_instance::<PythonKernel>().await? else {
            return Ok(());
        };

        instance.start_here().await?;

        // Deal with python exception message differences.
        let sw = instance.info().await?;
        let syntax_err = {
            // After 3.9 the error message changed (we only support 3.9 onward).
            if sw.options.software_version.unwrap().starts_with("3.9") {
                Some("unexpected EOF while parsing (<string>, line 1)")
            } else {
                Some("invalid syntax (<string>, line 1)")
            }
        };

        kernel_micro::tests::evaluation(
            instance,
            vec![
                ("1 + 1", Node::Integer(2), None),
                ("2.0 * 2.2", Node::Number(4.4), None),
                ("16 ** 0.5", Node::Number(4.0), None),
                ("'a' + 'bc'", Node::String("abc".to_string()), None),
                ("'ABC'.lower()", Node::String("abc".to_string()), None),
                ("[1]", Node::Array(Array(vec![Primitive::Integer(1)])), None),
                (
                    "['one']",
                    Node::Array(Array(vec![Primitive::String("one".into())])),
                    None,
                ),
                (
                    "[1, 2] + [3]",
                    Node::Array(Array(vec![
                        Primitive::Integer(1),
                        Primitive::Integer(2),
                        Primitive::Integer(3),
                    ])),
                    None,
                ),
                (
                    "range(4, 7)",
                    Node::Array(Array(vec![
                        Primitive::Integer(4),
                        Primitive::Integer(5),
                        Primitive::Integer(6),
                    ])),
                    None,
                ),
                (
                    "{**{'a': 1}, **{'b':2.3}}",
                    Node::Object(Object(IndexMap::from([
                        (String::from("a"), Primitive::Integer(1)),
                        (String::from("b"), Primitive::Number(2.3)),
                    ]))),
                    None,
                ),
                ("1+\n2", Node::Integer(3), None),
                ("", Node::Null(Null), None),
                ("@", Node::Null(Null), syntax_err),
                ("foo", Node::Null(Null), Some("name 'foo' is not defined")),
            ],
        )
        .await
    }

    /// Check that the logging is installed and captures warnings too.
    #[test_log::test(tokio::test)]
    async fn logging() -> Result<()> {
        let Some(mut instance) = start_instance::<PythonKernel>().await? else {
            return Ok(());
        };

        let (.., messages) = instance
            .execute(
                "
import logging
logger = logging.getLogger('just.a.test')
logger.error('oh no')
",
            )
            .await?;

        assert_eq!(messages.len(), 1);
        let m = messages.first().unwrap();
        assert_eq!(m.error_type.as_deref(), Some("just.a.test"));
        assert_eq!(m.level, MessageLevel::Error);

        let (.., messages) = instance
            .execute(
                "
import logging
logger = logging.getLogger('just.a.test')
logger.setLevel('DEBUG')
logger.debug('debug message')
logger.info('info message')
logger.warn('warning message')
logger.error('error message')
",
            )
            .await?;

        assert_eq!(messages.len(), 4);

        let mut messages = messages.into_iter();

        let m = messages.next().unwrap();
        assert_eq!(m.level, MessageLevel::Debug);
        assert_eq!(m.message, "debug message");

        let m = messages.next().unwrap();
        assert_eq!(m.level, MessageLevel::Info);
        assert_eq!(m.message, "info message");

        let m = messages.next().unwrap();
        assert_eq!(m.level, MessageLevel::Warning);
        assert_eq!(m.message, "warning message");

        let m = messages.next().unwrap();
        assert_eq!(m.level, MessageLevel::Error);
        assert_eq!(m.message, "error message");

        let (.., messages) = instance
            .execute(
                "
import warnings
warnings.warn('This is a warning message', UserWarning)
        ",
            )
            .await?;

        assert_eq!(messages.len(), 1);
        let m = messages.first().unwrap();
        assert_eq!(m.error_type.as_deref(), Some("UserWarning"));
        assert_eq!(m.level, MessageLevel::Warning);

        Ok(())
    }

    /// Standard kernel test for printing nodes
    #[test_log::test(tokio::test)]
    async fn printing() -> Result<()> {
        let Some(instance) = create_instance::<PythonKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::printing(
            instance,
            r#"print('str')"#,
            r#"print('str1', 'str2')"#,
            r#"print(None, True, 1, 2.3, 'str', [1, 2.3, 'str'], {'a':1, 'b':2.3, 'c':'str'})"#,
            r#"print({'type':'Paragraph', 'content':[]})"#,
        )
        .await
    }

    /// Custom test for execution messages
    #[tokio::test]
    async fn messages() -> Result<()> {
        let Some(mut kernel) = start_instance::<PythonKernel>().await? else {
            return Ok(());
        };

        // Syntax error
        let (outputs, messages) = kernel.execute("bad ^ # syntax").await?;
        let msg = &messages[0];
        assert_eq!(msg.error_type.as_deref(), Some("SyntaxError"));
        assert_eq!(msg.message, "invalid syntax (Code chunk #1, line 1)");
        assert!(msg.stack_trace.is_none());
        assert_eq!(
            msg.code_location,
            Some(CodeLocation {
                start_line: Some(0),
                end_line: Some(0),
                ..Default::default()
            })
        );
        assert_eq!(outputs, vec![]);

        // Runtime error
        let (outputs, messages) = kernel.execute("foo").await?;
        let msg = &messages[0];
        assert_eq!(msg.error_type.as_deref(), Some("NameError"));
        assert_eq!(msg.message, "name 'foo' is not defined");
        assert_eq!(
            msg.stack_trace.as_deref(),
            Some("Code chunk #2, line 1, in <module>\n")
        );
        assert_eq!(
            msg.code_location,
            Some(CodeLocation {
                start_line: Some(0),
                start_column: Some(0),
                end_line: Some(0),
                end_column: Some(3),
                ..Default::default()
            })
        );
        assert_eq!(outputs, vec![]);

        // Runtime error on last line
        let (.., messages) = kernel.execute("# Comment\n\n1 / 0").await?;
        let msg = &messages[0];
        assert_eq!(msg.error_type.as_deref(), Some("ZeroDivisionError"));
        assert_eq!(msg.message, "division by zero");
        assert_eq!(
            msg.stack_trace.as_deref(),
            Some("Code chunk #3, line 3, in <module>\n")
        );
        assert_eq!(
            msg.code_location,
            Some(CodeLocation {
                start_line: Some(2),
                start_column: Some(0),
                end_line: Some(2),
                end_column: Some(5),
                ..Default::default()
            })
        );

        // Nested error
        let (.., messages) = kernel
            .execute(
                r#"
# Comment   
def foo():
    bar()    
def baz():
    foo()
baz()
"#,
            )
            .await?;
        let msg = &messages[0];
        assert_eq!(msg.error_type.as_deref(), Some("NameError"));
        assert_eq!(msg.message, "name 'bar' is not defined");
        assert_eq!(
            msg.stack_trace.as_deref(),
            Some("Code chunk #4, line 7, in <module>\nCode chunk #4, line 6, in baz\nCode chunk #4, line 4, in foo\n")
        );
        assert_eq!(
            msg.code_location,
            Some(CodeLocation {
                start_line: Some(3),
                start_column: Some(4),
                end_line: Some(3),
                end_column: Some(7),
                ..Default::default()
            })
        );

        Ok(())
    }

    /// Standard kernel test for getting runtime information
    #[test_log::test(tokio::test)]
    async fn info() -> Result<()> {
        let Some(instance) = create_instance::<PythonKernel>().await? else {
            return Ok(());
        };

        let sw = kernel_micro::tests::info(instance).await?;
        assert_eq!(sw.name, "Python");
        assert!(sw.options.software_version.is_some());
        assert!(sw.options.software_version.unwrap().starts_with("3."));
        assert!(sw.options.operating_system.is_some());

        Ok(())
    }

    /// Standard kernel test for listing installed packages
    #[test_log::test(tokio::test)]
    async fn packages() -> Result<()> {
        let Some(instance) = start_instance::<PythonKernel>().await? else {
            return Ok(());
        };

        let pkgs = kernel_micro::tests::packages(instance).await?;
        assert!(!pkgs.is_empty());

        Ok(())
    }

    /// Standard kernel test for variable listing
    #[test_log::test(tokio::test)]
    async fn var_listing() -> Result<()> {
        let Some(instance) = create_instance::<PythonKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::var_listing(
            instance,
            r#"
nul = None
bool = True
int = 123
num = 1.23
str = "abc👍"
arr = [1, 2, 3]
obj = {'a':1, 'b':2.3}
para = {'type':'Paragraph', 'content':[]}
"#,
            vec![
                Variable {
                    name: "nul".to_string(),
                    native_type: Some("NoneType".to_string()),
                    node_type: Some("Null".to_string()),
                    programming_language: Some("Python".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "bool".to_string(),
                    native_type: Some("bool".to_string()),
                    node_type: Some("Boolean".to_string()),
                    hint: Some(Hint::Boolean(true)),
                    programming_language: Some("Python".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "int".to_string(),
                    native_type: Some("int".to_string()),
                    node_type: Some("Integer".to_string()),
                    hint: Some(Hint::Integer(123)),
                    programming_language: Some("Python".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "num".to_string(),
                    native_type: Some("float".to_string()),
                    node_type: Some("Number".to_string()),
                    hint: Some(Hint::Number(1.23)),
                    programming_language: Some("Python".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "str".to_string(),
                    native_type: Some("str".to_string()),
                    node_type: Some("String".to_string()),
                    hint: Some(Hint::StringHint(StringHint::new(4))),
                    programming_language: Some("Python".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "arr".to_string(),
                    native_type: Some("list".to_string()),
                    node_type: Some("Array".to_string()),
                    hint: Some(Hint::ArrayHint(ArrayHint {
                        length: 3,
                        ..Default::default()
                    })),
                    programming_language: Some("Python".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "obj".to_string(),
                    native_type: Some("dict".to_string()),
                    node_type: Some("Object".to_string()),
                    hint: Some(Hint::ObjectHint(ObjectHint::new(
                        2,
                        vec!["a".to_string(), "b".to_string()],
                        vec![Hint::Integer(1), Hint::Number(2.3)],
                    ))),
                    programming_language: Some("Python".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "para".to_string(),
                    native_type: Some("dict".to_string()),
                    node_type: Some("Paragraph".to_string()),
                    programming_language: Some("Python".to_string()),
                    ..Default::default()
                },
            ],
        )
        .await
    }

    /// Custom Python kernel test for variable listing to ensure some globals
    /// and imported modules are excluded
    #[test_log::test(tokio::test)]
    async fn var_listing_excluded() -> Result<()> {
        let Some(mut instance) = start_instance::<PythonKernel>().await? else {
            return Ok(());
        };

        // Import a module to check that does not appear in the list
        instance.execute("import datetime").await?;

        let vars = instance.list().await?;
        assert!(!vars.iter().any(|var| var.name == "__builtins__"
            || var.name == "print"
            || var.name == "datetime"));

        Ok(())
    }

    /// Standard kernel test for variable management
    #[test_log::test(tokio::test)]
    async fn var_management() -> Result<()> {
        let Some(instance) = create_instance::<PythonKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::var_management(instance).await
    }

    /// `PythonKernel` specific test for `list` and `get` with `numpy.ndarray`s
    #[test_log::test(tokio::test)]
    async fn numpy() -> Result<()> {
        let Some(mut instance) = start_instance::<PythonKernel>().await? else {
            return Ok(());
        };

        let (.., messages) = instance.execute("import numpy as np").await?;
        if messages
            .first()
            .and_then(|message| message.error_type.as_deref())
            == Some("ModuleNotFoundError")
        {
            eprintln!("Skipping test because `numpy` not available");
            return Ok(());
        }

        let (.., messages) = instance
            .execute(
                "
a1 = np.array([True, False], dtype=np.bool_)
a2 = np.array([-1, 0, 1], dtype=np.int_)
a3 = np.array([1, 2 , 3], dtype=np.uint)
a364 = np.array([4, 5, 6], dtype=np.uint64)
a4 = np.array([1.23, 4.56], dtype=np.float64)

# TODO: implement handling for these
#a5 = np.array(['2020-01-01', '2020-01-02', '2020-01-03'], dtype=np.datetime64)
#a6 = np.array([], dtype=np.timedelta64)
",
            )
            .await?;
        assert_eq!(messages, []);

        let list = instance.list().await?;

        macro_rules! var {
            ($name:expr) => {{
                let mut var = list.iter().find(|var| var.name == $name).unwrap().clone();
                var.native_hint = None;
                var
            }};
        }
        macro_rules! get {
            ($name:expr) => {
                instance.get($name).await?.unwrap()
            };
        }

        assert_eq!(
            var!("a1"),
            Variable {
                name: "a1".to_string(),
                native_type: Some("ndarray".to_string()),
                node_type: Some("Array".to_string()),
                hint: Some(Hint::ArrayHint(ArrayHint {
                    length: 2,
                    item_types: Some(vec!["Boolean".to_string()]),
                    minimum: Some(Primitive::Boolean(false)),
                    maximum: Some(Primitive::Boolean(true)),
                    nulls: Some(0),
                    ..Default::default()
                })),
                programming_language: Some("Python".to_string()),
                ..Default::default()
            },
        );
        assert_eq!(
            get!("a1"),
            Node::Array(Array(vec![
                Primitive::Boolean(true),
                Primitive::Boolean(false)
            ]))
        );

        assert_eq!(
            var!("a2"),
            Variable {
                name: "a2".to_string(),
                native_type: Some("ndarray".to_string()),
                node_type: Some("Array".to_string()),
                hint: Some(Hint::ArrayHint(ArrayHint {
                    length: 3,
                    item_types: Some(vec!["Integer".to_string()]),
                    minimum: Some(Primitive::Integer(-1)),
                    maximum: Some(Primitive::Integer(1)),
                    nulls: Some(0),
                    ..Default::default()
                })),
                programming_language: Some("Python".to_string()),
                ..Default::default()
            },
        );
        assert_eq!(
            get!("a2"),
            Node::Array(Array(vec![
                Primitive::Integer(-1),
                Primitive::Integer(0),
                Primitive::Integer(1)
            ]))
        );

        assert_eq!(
            var!("a3"),
            Variable {
                name: "a3".to_string(),
                native_type: Some("ndarray".to_string()),
                node_type: Some("Array".to_string()),
                hint: Some(Hint::ArrayHint(ArrayHint {
                    length: 3,
                    item_types: Some(vec!["UnsignedInteger".to_string()]),
                    minimum: Some(Primitive::Integer(1)),
                    maximum: Some(Primitive::Integer(3)),
                    nulls: Some(0),
                    ..Default::default()
                })),
                programming_language: Some("Python".to_string()),
                ..Default::default()
            },
        );
        assert_eq!(
            get!("a3"),
            Node::Array(Array(vec![
                Primitive::Integer(1),
                Primitive::Integer(2),
                Primitive::Integer(3)
            ]))
        );

        assert_eq!(
            var!("a364"),
            Variable {
                name: "a364".to_string(),
                native_type: Some("ndarray".to_string()),
                node_type: Some("Array".to_string()),
                hint: Some(Hint::ArrayHint(ArrayHint {
                    length: 3,
                    item_types: Some(vec!["UnsignedInteger".to_string()]),
                    minimum: Some(Primitive::Integer(4)),
                    maximum: Some(Primitive::Integer(6)),
                    nulls: Some(0),
                    ..Default::default()
                })),
                programming_language: Some("Python".to_string()),
                ..Default::default()
            },
        );
        assert_eq!(
            get!("a364"),
            Node::Array(Array(vec![
                Primitive::Integer(4),
                Primitive::Integer(5),
                Primitive::Integer(6)
            ]))
        );

        assert_eq!(
            var!("a4"),
            Variable {
                name: "a4".to_string(),
                native_type: Some("ndarray".to_string()),
                node_type: Some("Array".to_string()),
                hint: Some(Hint::ArrayHint(ArrayHint {
                    length: 2,
                    item_types: Some(vec!["Number".to_string()]),
                    minimum: Some(Primitive::Number(1.23)),
                    maximum: Some(Primitive::Number(4.56)),
                    nulls: Some(0),
                    ..Default::default()
                })),
                programming_language: Some("Python".to_string()),
                ..Default::default()
            },
        );
        assert_eq!(
            get!("a4"),
            Node::Array(Array(vec![
                Primitive::Number(1.23),
                Primitive::Number(4.56)
            ]))
        );

        // TODO: asserts for a5, a6

        Ok(())
    }

    /// `PythonKernel` specific test for `list` and `get` with `pandas.DataFrame`s
    #[test_log::test(tokio::test)]
    async fn pandas_list_get() -> Result<()> {
        let Some(mut instance) = start_instance::<PythonKernel>().await? else {
            return Ok(());
        };

        let (.., messages) = instance.execute("import pandas as pd").await?;
        if messages
            .first()
            .and_then(|message| message.error_type.as_deref())
            == Some("ModuleNotFoundError")
        {
            eprintln!("Skipping test because `pandas` not available");
            return Ok(());
        }

        let (.., messages) = instance
            .execute(
                "
df1 = pd.DataFrame({
    'c1': [True, False],
    'c2': [1, 2],
    'c3': [1.23, 4.56],
    'c4': ['A', 'B']
})
",
            )
            .await?;
        assert_eq!(messages, []);

        let list = instance.list().await?;

        macro_rules! var {
            ($name:expr) => {{
                list.iter().find(|var| var.name == $name).unwrap().clone()
            }};
        }
        macro_rules! get {
            ($name:expr) => {
                instance.get($name).await?.unwrap()
            };
        }

        assert_eq!(
            var!("df1"),
            Variable {
                name: "df1".to_string(),
                native_type: Some("DataFrame".to_string()),
                node_type: Some("Datatable".to_string()),
                hint: Some(Hint::DatatableHint(DatatableHint::new(
                    2,
                    vec![
                        DatatableColumnHint {
                            name: "c1".to_string(),
                            item_type: "Boolean".to_string(),
                            minimum: Some(Primitive::Boolean(false)),
                            maximum: Some(Primitive::Boolean(true)),
                            nulls: Some(0),
                            ..Default::default()
                        },
                        DatatableColumnHint {
                            name: "c2".to_string(),
                            item_type: "Integer".to_string(),
                            minimum: Some(Primitive::Integer(1)),
                            maximum: Some(Primitive::Integer(2)),
                            nulls: Some(0),
                            ..Default::default()
                        },
                        DatatableColumnHint {
                            name: "c3".to_string(),
                            item_type: "Number".to_string(),
                            minimum: Some(Primitive::Number(1.23)),
                            maximum: Some(Primitive::Number(4.56)),
                            nulls: Some(0),
                            ..Default::default()
                        },
                        DatatableColumnHint {
                            name: "c4".to_string(),
                            item_type: "String".to_string(),
                            ..Default::default()
                        }
                    ]
                ))),
                native_hint: Some(
                    r#"The `dtypes` of the `DataFrame` are:

```
c1       bool
c2      int64
c3    float64
c4     object
dtype: object
```

The first few rows of the `DataFrame` are:

```
      c1  c2    c3 c4
0   True   1  1.23  A
1  False   2  4.56  B
```

The `describe` method of the `DataFrame` returns:

```
             c2        c3
count  2.000000  2.000000
mean   1.500000  2.895000
std    0.707107  2.354666
min    1.000000  1.230000
25%    1.250000  2.062500
50%    1.500000  2.895000
75%    1.750000  3.727500
max    2.000000  4.560000
```"#
                        .to_string()
                ),
                programming_language: Some("Python".to_string()),
                ..Default::default()
            },
        );
        assert_eq!(
            get!("df1"),
            Node::Datatable(Datatable::new(vec![
                DatatableColumn {
                    name: "c1".to_string(),
                    values: vec![Primitive::Boolean(true), Primitive::Boolean(false)],
                    validator: Some(ArrayValidator {
                        items_validator: Some(Box::new(Validator::BooleanValidator(
                            BooleanValidator::new()
                        ))),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                DatatableColumn {
                    name: "c2".to_string(),
                    values: vec![Primitive::Integer(1), Primitive::Integer(2)],
                    validator: Some(ArrayValidator {
                        items_validator: Some(Box::new(Validator::IntegerValidator(
                            IntegerValidator::new()
                        ))),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                DatatableColumn {
                    name: "c3".to_string(),
                    values: vec![Primitive::Number(1.23), Primitive::Number(4.56)],
                    validator: Some(ArrayValidator {
                        items_validator: Some(Box::new(Validator::NumberValidator(
                            NumberValidator::new()
                        ))),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                DatatableColumn {
                    name: "c4".to_string(),
                    values: vec![Primitive::String("A".into()), Primitive::String("B".into())],
                    validator: Some(ArrayValidator {
                        items_validator: Some(Box::new(Validator::StringValidator(
                            StringValidator::new()
                        ))),
                        ..Default::default()
                    }),
                    ..Default::default()
                }
            ]))
        );

        Ok(())
    }

    /// `PythonKernel` specific test to test round-trip `set`/`get` with `pandas.DataFrame`s
    #[test_log::test(tokio::test)]
    async fn pandas_set_get() -> Result<()> {
        let Some(mut instance) = start_instance::<PythonKernel>().await? else {
            return Ok(());
        };

        let (.., messages) = instance.execute("import pandas as pd").await?;
        if messages
            .first()
            .and_then(|message| message.error_type.as_deref())
            == Some("ModuleNotFoundError")
        {
            eprintln!("Skipping test because `pandas` not available");
            return Ok(());
        }

        let dt_in = Node::Datatable(Datatable::new(vec![
            DatatableColumn {
                name: "c1".to_string(),
                values: vec![Primitive::Boolean(true), Primitive::Boolean(false)],
                validator: Some(ArrayValidator {
                    items_validator: Some(Box::new(Validator::BooleanValidator(
                        BooleanValidator::new(),
                    ))),
                    ..Default::default()
                }),
                ..Default::default()
            },
            DatatableColumn {
                name: "c2".to_string(),
                values: vec![Primitive::Integer(1), Primitive::Integer(2)],
                validator: Some(ArrayValidator {
                    items_validator: Some(Box::new(Validator::IntegerValidator(
                        IntegerValidator::new(),
                    ))),
                    ..Default::default()
                }),
                ..Default::default()
            },
            DatatableColumn {
                name: "c3".to_string(),
                values: vec![Primitive::Number(1.23), Primitive::Number(4.56)],
                validator: Some(ArrayValidator {
                    items_validator: Some(Box::new(Validator::NumberValidator(
                        NumberValidator::new(),
                    ))),
                    ..Default::default()
                }),
                ..Default::default()
            },
        ]));

        instance.set("dt", &dt_in).await?;

        let (output, messages) = instance.evaluate("type(dt)").await?;
        assert_eq!(messages, []);
        assert_eq!(
            output,
            Node::String("<class 'pandas.core.frame.DataFrame'>".to_string())
        );

        let dt_out = instance.get("dt").await?.unwrap();
        assert_eq!(dt_out, dt_in);

        Ok(())
    }

    /// `PythonKernel` specific test for getting a `matplotlib` plot as output
    #[test_log::test(tokio::test)]
    async fn matplotlib() -> Result<()> {
        let Some(mut instance) = start_instance::<PythonKernel>().await? else {
            return Ok(());
        };

        let (.., messages) = instance.execute("import matplotlib.pyplot as plt").await?;
        if messages
            .first()
            .and_then(|message| message.error_type.as_deref())
            == Some("ModuleNotFoundError")
        {
            eprintln!("Skipping test because `matplotlib` not available");
            return Ok(());
        }

        let (outputs, messages) = instance
            .execute(
                "
plt.plot([1, 2], [3, 4]);
plt.show()",
            )
            .await?;
        assert_eq!(messages, []);

        assert_eq!(outputs.len(), 1);

        if let Some(Node::ImageObject(image)) = outputs.first() {
            assert!(image.content_url.starts_with("data:image/png;base64"));
        } else {
            bail!("Expected an image, got: {outputs:?}")
        }

        Ok(())
    }

    /// `PythonKernel` specific test for getting a `plotly` plot as output
    #[test_log::test(tokio::test)]
    async fn plotly() -> Result<()> {
        let Some(mut instance) = start_instance::<PythonKernel>().await? else {
            return Ok(());
        };

        let (.., messages) = instance.execute("import plotly.express as px").await?;
        if messages
            .first()
            .and_then(|message| message.error_type.as_deref())
            == Some("ModuleNotFoundError")
        {
            eprintln!("Skipping test because `plotly` not available");
            return Ok(());
        }

        let (outputs, messages) = instance
            .execute("fig = px.scatter(px.data.iris(), x='sepal_width', y='sepal_length')")
            .await?;
        assert_eq!(messages, []);
        assert_eq!(outputs, []);

        let (outputs, messages) = instance.execute("fig.show()").await?;
        assert_eq!(messages, []);
        if let Some(Node::ImageObject(ImageObject {
            media_type: Some(media_type),
            ..
        })) = outputs.first()
        {
            assert_eq!(media_type, "application/vnd.plotly.v1+json");
        } else {
            bail!("Expected an image with a media_type, got: {outputs:?}")
        }

        let (outputs, messages) = instance.execute("fig").await?;
        assert_eq!(messages, []);
        if let Some(Node::ImageObject(ImageObject {
            media_type: Some(media_type),
            ..
        })) = outputs.first()
        {
            assert_eq!(media_type, "application/vnd.plotly.v1+json");
        } else {
            bail!("Expected an image with a media_type, got: {outputs:?}")
        }

        Ok(())
    }

    /// `PythonKernel` specific test for getting an Altair plot as output
    #[test_log::test(tokio::test)]
    async fn altair() -> Result<()> {
        let Some(mut instance) = start_instance::<PythonKernel>().await? else {
            return Ok(());
        };

        let (.., messages) = instance.execute("import altair as alt").await?;
        if messages
            .first()
            .and_then(|message| message.error_type.as_deref())
            == Some("ModuleNotFoundError")
        {
            eprintln!("Skipping test because `altair` not available");
            return Ok(());
        }

        let (outputs, messages) = instance
            .execute(
                "
import numpy as np
import pandas as pd

df = pd.DataFrame({
    'x': np.random.uniform(0, 1, 100),
    'y': np.random.uniform(0, 1, 100)
})

# Charts can span multiple lines
alt.Chart(df).mark_point().encode(
    x=alt.X('x', scale=alt.Scale(domain=[0, 1])),
    y=alt.Y('y', scale=alt.Scale(domain=[0, 1]))
).properties(width=400, height=300)
",
            )
            .await?;
        assert_eq!(messages, []);
        if let Some(Node::ImageObject(ImageObject {
            media_type: Some(media_type),
            ..
        })) = outputs.first()
        {
            assert_eq!(media_type, "application/vnd.vegalite.v5+json");
        } else {
            bail!("Expected an image with a media_type, got: {outputs:?}")
        }

        let (outputs, messages) = instance
            .execute(
                "
# Assigned and 'returned'
chart = alt.Chart(df).mark_point().encode(
  x=alt.X('x'),
  y=alt.Y('y')
)
chart
",
            )
            .await?;
        assert_eq!(messages, []);
        if let Some(Node::ImageObject(ImageObject {
            media_type: Some(media_type),
            ..
        })) = outputs.first()
        {
            assert_eq!(media_type, "application/vnd.vegalite.v5+json");
        } else {
            bail!("Expected an image with a media_type, got: {outputs:?}")
        }

        Ok(())
    }

    /// Standard kernel test for forking
    #[test_log::test(tokio::test)]
    async fn forking() -> Result<()> {
        let Some(instance) = create_instance::<PythonKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::forking(instance).await
    }

    /// Custom test to check that modules imported in the main kernel instance are
    /// available in the forked instance
    #[test_log::test(tokio::test)]
    async fn forking_imports() -> Result<()> {
        let Some(mut instance) = start_instance::<PythonKernel>().await? else {
            return Ok(());
        };

        let (outputs, messages) = instance
            .execute(
                r#"
import sys
from datetime import datetime
from glob import *

print(type(sys), type(datetime), type(glob))
"#,
            )
            .await?;
        assert_eq!(messages, vec![]);
        assert_eq!(
            outputs,
            vec![
                Node::String("<class 'module'>".to_string()),
                Node::String("<class 'type'>".to_string()),
                Node::String("<class 'function'>".to_string())
            ]
        );

        let mut fork = instance.replicate(ExecutionBounds::Fork).await?;
        let (outputs, messages) = fork
            .execute(
                r#"
print(type(sys), type(datetime), type(glob))
"#,
            )
            .await?;
        assert_eq!(messages, vec![]);
        assert_eq!(
            outputs,
            vec![
                Node::String("<class 'module'>".to_string()),
                Node::String("<class 'type'>".to_string()),
                Node::String("<class 'function'>".to_string())
            ]
        );

        Ok(())
    }

    /// Test that each forked kernel generates unique random numbers
    #[test_log::test(tokio::test)]
    async fn forking_randoms() -> Result<()> {
        let Some(mut instance) = start_instance::<PythonKernel>().await? else {
            return Ok(());
        };

        instance.execute("from random import random").await?;

        let (rand, ..) = instance.evaluate("random()").await?;

        let mut fork1 = instance.replicate(ExecutionBounds::Fork).await?;
        let (rand1, ..) = fork1.evaluate("random()").await?;
        assert_ne!(rand, rand1);

        let mut fork2 = instance.replicate(ExecutionBounds::Fork).await?;
        let (rand2, ..) = fork2.evaluate("random()").await?;
        assert_ne!(rand, rand2);
        assert_ne!(rand1, rand2);

        Ok(())
    }

    /// Standard kernel test for signals
    #[ignore = "signals not received when `uv run` is used"]
    #[test_log::test(tokio::test)]
    async fn signals() -> Result<()> {
        let Some(instance) = create_instance::<PythonKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::signals(
            instance,
            "
# Setup step
from time import sleep
sleep(0.1)
value = 1
value",
            Some(
                "
# Interrupt step
sleep(100)
value = 2",
            ),
            None,
            Some(
                "
# Kill step
sleep(100)",
            ),
        )
        .await
    }

    /// Standard kernel test for stopping
    #[test_log::test(tokio::test)]
    async fn stop() -> Result<()> {
        let Some(instance) = create_instance::<PythonKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::stop(instance).await
    }

    /// `PythonKernel` specific test that imported modules are available in functions
    ///
    /// This is a regression test for a bug found during usage with v1.
    /// Before the associated fix would get error "name 'time' is not defined"
    #[tokio::test]
    async fn imports() -> Result<()> {
        let Some(mut instance) = start_instance::<PythonKernel>().await? else {
            return Ok(());
        };

        // Import a module and a function from another module in one task
        let (outputs, messages) = instance
            .execute(
                "
import time
from datetime import datetime
",
            )
            .await?;
        assert_eq!(messages, []);
        assert_eq!(outputs, []);

        // Check that both can be used from within a function in another task
        let (outputs, messages) = instance
            .execute(
                "
def func():
    return (time.time(), datetime.now())

func()",
            )
            .await?;
        assert_eq!(messages, []);
        assert_eq!(outputs.len(), 1);

        Ok(())
    }

    /// Custom test for boxed kernel
    ///
    /// Currently just a few tests covering the main categories of restriction.
    #[tokio::test]
    async fn boxed() -> Result<()> {
        let Some(mut instance) = start_instance_with::<PythonKernel>(ExecutionBounds::Box).await?
        else {
            return Ok(());
        };

        instance.execute("import os").await?;

        // Read-only access to files
        let (.., messages) = instance.execute("open('write.txt', 'w')").await?;
        assert_eq!(
            messages[0].message,
            "Write access to filesystem is restricted in boxed kernel"
        );

        let (.., messages) = instance
            .execute("os.open('read-write.txt', os.O_RDWR)")
            .await?;
        assert_eq!(
            messages[0].message,
            "Write access to filesystem is restricted in boxed kernel"
        );

        let (.., messages) = instance.execute("os.remove('some-file.txt')").await?;
        assert_eq!(
            messages[0].message,
            "Write access to filesystem is restricted in boxed kernel"
        );

        // No process management
        let (.., messages) = instance.execute("os.system('command')").await?;
        assert_eq!(
            messages[0].message,
            "Process management is restricted in boxed kernel"
        );

        // No network access
        let (.., messages) = instance
            .execute("from urllib.request import urlopen; urlopen('http://example.com')")
            .await?;
        assert_eq!(
            messages[0].message,
            "<urlopen error Network access is restricted in boxed kernel>"
        );

        Ok(())
    }
}
