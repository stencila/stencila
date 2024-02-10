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
        schema::{Array, Node, Null, Object, Primitive},
        tests::{create_instance, start_instance},
    };

    use super::*;

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

    /// Test list, set and get tasks
    #[tokio::test]
    async fn vars() -> Result<()> {
        let Some(mut kernel) = start_instance::<PythonKernel>().await? else {
            return Ok(());
        };

        // List existing env vars
        let initial = kernel.list().await?;
        assert_eq!(initial.len(), 0);

        // Set a var
        let var_name = "my_var";
        let var_val = Node::String("Hello Python!".to_string());
        kernel.set(var_name, &var_val).await?;
        assert_eq!(kernel.list().await?.len(), initial.len() + 1);

        // Get the var
        assert_eq!(kernel.get(var_name).await?, Some(var_val));

        // Remove the var
        kernel.remove(var_name).await?;
        assert_eq!(kernel.get(var_name).await?, None);
        assert_eq!(kernel.list().await?.len(), initial.len());

        Ok(())
    }

    /// Test declaring JavaScript variables with different types
    #[tokio::test]
    async fn var_types() -> Result<()> {
        let Some(mut kernel) = start_instance::<PythonKernel>().await? else {
            return Ok(());
        };

        kernel
            .execute(
                r#"
n = 1.23
s = "str"
a = [1, 2, 3]
o = {'a':1, 'b':2.3}
        "#,
            )
            .await?;

        let vars = kernel.list().await?;

        let var = vars.iter().find(|var| var.name == "n").unwrap();
        assert_eq!(var.node_type.as_deref(), Some("Number"));
        assert_eq!(var.native_type.as_deref(), Some("float"));
        assert_eq!(kernel.get("n").await?, Some(Node::Number(1.23)));

        let var = vars.iter().find(|var| var.name == "s").unwrap();
        assert_eq!(var.node_type.as_deref(), Some("String"));
        assert_eq!(var.native_type.as_deref(), Some("str"));
        assert!(matches!(kernel.get("s").await?, Some(Node::String(..))));

        let var = vars.iter().find(|var| var.name == "a").unwrap();
        assert_eq!(var.node_type.as_deref(), Some("Array"));
        assert_eq!(var.native_type.as_deref(), Some("list"));
        assert_eq!(
            kernel.get("a").await?,
            Some(Node::Array(Array(vec![
                Primitive::Integer(1),
                Primitive::Integer(2),
                Primitive::Integer(3)
            ])))
        );

        let var = vars.iter().find(|var| var.name == "o").unwrap();
        assert_eq!(var.node_type.as_deref(), Some("Object"));
        assert_eq!(var.native_type.as_deref(), Some("dict"));
        assert_eq!(
            kernel.get("o").await?,
            Some(Node::Object(Object(IndexMap::from([
                (String::from("a"), Primitive::Integer(1),),
                (String::from("b"), Primitive::Number(2.3))
            ]))))
        );

        Ok(())
    }

    /// Test execute tasks that intentionally generate error messages
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

    /// Test forking of microkernel
    ///
    /// Pro-tip! Use this to get logs for this test:
    ///
    /// ```sh
    /// RUST_LOG=trace cargo test -p kernel-node forks -- --nocapture
    /// ```
    #[test_log::test(tokio::test)]
    async fn forks() -> Result<()> {
        let Some(mut kernel) = start_instance::<PythonKernel>().await? else {
            return Ok(());
        };

        // Set variables in the kernel
        kernel.set("var1", &Node::Integer(123)).await?;
        kernel.set("var2", &Node::Number(4.56)).await?;
        kernel
            .set("var3", &Node::String("Hello world".to_string()))
            .await?;

        // Create a fork and check that the variables are available in it
        let mut fork = kernel.fork().await?;
        assert_eq!(fork.get("var1").await?, Some(Node::Integer(123)));
        assert_eq!(fork.get("var2").await?, Some(Node::Number(4.56)));
        assert_eq!(
            fork.get("var3").await?,
            Some(Node::String("Hello world".to_string()))
        );

        // Change variables in fork and check that they are unchanged in main kernel
        fork.set("var1", &Node::Integer(321)).await?;
        fork.remove("var2").await?;
        fork.execute("var3 = 'Hello from fork'").await?;
        assert_eq!(kernel.get("var1").await?, Some(Node::Integer(123)));
        assert_eq!(kernel.get("var2").await?, Some(Node::Number(4.56)));
        assert_eq!(
            kernel.get("var3").await?,
            Some(Node::String("Hello world".to_string()))
        );

        Ok(())
    }
}
