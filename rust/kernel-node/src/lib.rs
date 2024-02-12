use std::sync::atomic::{AtomicU64, Ordering};

use kernel_micro::{
    common::eyre::Result, format::Format, Kernel, KernelAvailability, KernelForks, KernelInstance,
    KernelInterrupt, KernelKill, KernelTerminate, Microkernel,
};

/// A kernel for executing JavaScript code in Node.js
#[derive(Default)]
pub struct NodeKernel {
    /// A counter of instances of this microkernel
    instances: AtomicU64,
}

impl Kernel for NodeKernel {
    fn id(&self) -> String {
        "node".to_string()
    }

    fn availability(&self) -> KernelAvailability {
        self.microkernel_availability()
    }

    fn supports_languages(&self) -> Vec<Format> {
        vec![Format::JavaScript]
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
        self.microkernel_create_instance(self.instances.fetch_add(1, Ordering::SeqCst))
    }
}

impl Microkernel for NodeKernel {
    fn executable_name(&self) -> String {
        "node".to_string()
    }

    fn microkernel_script(&self) -> String {
        include_str!("kernel.js").to_string()
    }
}

#[cfg(test)]
mod tests {
    use common_dev::pretty_assertions::assert_eq;
    use kernel_micro::{
        common::{indexmap::IndexMap, tokio},
        schema::{
            Array, ArrayHint, ExecutionMessage, ExecutionMessageLevel, Hint, Node, Null, Object,
            ObjectHint, Primitive, StringHint, Variable,
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
        let Some(instance) = create_instance::<NodeKernel>().await? else {
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
        let Some(instance) = create_instance::<NodeKernel>().await? else {
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
        let Some(instance) = create_instance::<NodeKernel>().await? else {
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
        let Some(mut kernel) = start_instance::<NodeKernel>().await? else {
            return Ok(())
        };

        // Syntax error
        let (outputs, messages) = kernel.execute("bad ^ # syntax").await?;
        assert_eq!(messages[0].error_type.as_deref(), Some("SyntaxError"));
        assert_eq!(messages[0].message, "Invalid or unexpected token");
        assert!(messages[0].stack_trace.is_some());
        assert_eq!(outputs, vec![]);

        // Runtime error
        let (outputs, messages) = kernel.execute("foo").await?;
        assert_eq!(messages[0].error_type.as_deref(), Some("ReferenceError"));
        assert_eq!(messages[0].message, "foo is not defined");
        assert!(messages[0].stack_trace.is_some());
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
                    level: ExecutionMessageLevel::Debug,
                    message: "Debug message".to_string(),
                    ..Default::default()
                },
                ExecutionMessage {
                    level: ExecutionMessageLevel::Info,
                    message: "Info message".to_string(),
                    ..Default::default()
                },
                ExecutionMessage {
                    level: ExecutionMessageLevel::Warn,
                    message: "Warning message".to_string(),
                    ..Default::default()
                },
                ExecutionMessage {
                    level: ExecutionMessageLevel::Error,
                    message: "Error message".to_string(),
                    ..Default::default()
                }
            ]
        );

        Ok(())
    }

    /// Standard kernel test for variable listing
    #[test_log::test(tokio::test)]
    async fn var_listing() -> Result<()> {
        let Some(instance) = create_instance::<NodeKernel>().await? else {
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
    #[test_log::test(tokio::test)]
    async fn var_management() -> Result<()> {
        let Some(instance) = create_instance::<NodeKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::var_management(instance).await
    }

    /// Standard kernel test for forking
    #[test_log::test(tokio::test)]
    async fn forking() -> Result<()> {
        let Some(instance) = create_instance::<NodeKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::forking(instance).await
    }

    /// Custom test to check that modules imported in the main kernel instance are
    /// available in the forked instance
    #[test_log::test(tokio::test)]
    async fn forking_imports() -> Result<()> {
        let Some(mut instance) = start_instance::<NodeKernel>().await? else {
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
    #[test_log::test(tokio::test)]
    async fn signals() -> Result<()> {
        let Some(instance) = create_instance::<NodeKernel>().await? else {
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
        let Some(instance) = create_instance::<NodeKernel>().await? else {
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
        let Some(mut kernel) = start_instance::<NodeKernel>().await? else {
            return Ok(())
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
