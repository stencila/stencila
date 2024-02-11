use std::sync::atomic::{AtomicU64, Ordering};

use kernel_micro::{
    common::eyre::Result, format::Format, Kernel, KernelAvailability, KernelForks, KernelInstance,
    KernelInterrupt, KernelKill, KernelTerminate, Microkernel,
};

/// A kernel for executing Python code
#[derive(Default)]
pub struct PythonKernel {
    /// A counter of instances of this microkernel
    instances: AtomicU64,
}

impl Kernel for PythonKernel {
    fn id(&self) -> String {
        "python".to_string()
    }

    fn availability(&self) -> KernelAvailability {
        self.microkernel_availability()
    }

    fn supports_languages(&self) -> Vec<Format> {
        vec![Format::Python]
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
        // Uses Python `os.fork()` which is only available on POSIX-based systems
        self.microkernel_supports_forks()
    }

    fn create_instance(&self) -> Result<Box<dyn KernelInstance>> {
        self.microkernel_create_instance(self.instances.fetch_add(1, Ordering::SeqCst))
    }
}

impl Microkernel for PythonKernel {
    fn executable_name(&self) -> String {
        "python3".to_string()
    }

    fn microkernel_script(&self) -> String {
        include_str!("kernel.py").to_string()
    }
}

#[cfg(test)]
mod tests {
    use common_dev::pretty_assertions::assert_eq;
    use kernel_micro::{
        common::{eyre::Ok, indexmap::IndexMap, tokio},
        schema::{Array, Node, Null, Object, Primitive, Variable},
        tests::{create_instance, start_instance},
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
                // Prints and an expression: multiple, separate outputs
                (
                    "
print(1);
print(2, 3);
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

    /// Standard kernel test for evaluation of expressions
    #[test_log::test(tokio::test)]
    async fn evaluation() -> Result<()> {
        let Some(instance) = create_instance::<PythonKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::evaluation(
            instance,
            vec![
                ("1 + 1", Node::Integer(2), None),
                ("2.0 * 2.2", Node::Number(4.4), None),
                ("16 ** 0.5", Node::Number(4.0), None),
                ("'a' + 'bc'", Node::String("abc".to_string()), None),
                ("'ABC'.lower()", Node::String("abc".to_string()), None),
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
                    "{**{'a': 1}, **{'b':2.3}}",
                    Node::Object(Object(IndexMap::from([
                        (String::from("a"), Primitive::Integer(1)),
                        (String::from("b"), Primitive::Number(2.3)),
                    ]))),
                    None,
                ),
                ("", Node::Null(Null), None),
                (
                    "@",
                    Node::Null(Null),
                    Some("invalid syntax (<string>, line 1)"),
                ),
                ("foo", Node::Null(Null), Some("name 'foo' is not defined")),
            ],
        )
        .await
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
        assert_eq!(messages[0].error_type.as_deref(), Some("SyntaxError"));
        assert_eq!(messages[0].error_message, "invalid syntax (<code>, line 1)");
        assert!(messages[0].stack_trace.is_some());
        assert_eq!(outputs, vec![]);

        // Runtime error
        let (outputs, messages) = kernel.execute("foo").await?;
        assert_eq!(messages[0].error_type.as_deref(), Some("NameError"));
        assert_eq!(messages[0].error_message, "name 'foo' is not defined");
        assert!(messages[0].stack_trace.is_some());
        assert_eq!(outputs, vec![]);

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
str = "str"
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
                    value_hint: Some(Box::new(Node::Boolean(true))),
                    programming_language: Some("Python".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "int".to_string(),
                    native_type: Some("int".to_string()),
                    node_type: Some("Integer".to_string()),
                    value_hint: Some(Box::new(Node::Integer(123))),
                    programming_language: Some("Python".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "num".to_string(),
                    native_type: Some("float".to_string()),
                    node_type: Some("Number".to_string()),
                    value_hint: Some(Box::new(Node::Number(1.23))),
                    programming_language: Some("Python".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "str".to_string(),
                    native_type: Some("str".to_string()),
                    node_type: Some("String".to_string()),
                    value_hint: Some(Box::new(Node::Integer(3))),
                    programming_language: Some("Python".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "arr".to_string(),
                    native_type: Some("list".to_string()),
                    node_type: Some("Array".to_string()),
                    value_hint: Some(Box::new(Node::Integer(3))),
                    programming_language: Some("Python".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "obj".to_string(),
                    native_type: Some("dict".to_string()),
                    node_type: Some("Object".to_string()),
                    value_hint: Some(Box::new(Node::Integer(2))),
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

    /// Standard kernel test for variable management
    #[test_log::test(tokio::test)]
    async fn var_management() -> Result<()> {
        let Some(instance) = create_instance::<PythonKernel>().await? else {
                return Ok(());
            };

        kernel_micro::tests::var_management(instance).await
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

        let mut fork = instance.fork().await?;
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

    /// Standard kernel test for signals
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
            .execute("
import time
from datetime import datetime
")
            .await?;
        assert_eq!(messages, []);
        assert_eq!(outputs, []);

        // Check that both can be used from within a function in another task
        let (outputs, messages) = instance
            .execute("
def func():
    return (time.time(), datetime.now())

func()")
            .await?;
        assert_eq!(messages, []);
        assert_eq!(outputs.len(), 1);

        Ok(())
    }
}
