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
    fn name(&self) -> String {
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
        common::{
            eyre::{bail, Ok},
            indexmap::IndexMap,
            tokio,
        },
        schema::MessageLevel,
        schema::{
            Array, ArrayHint, ArrayValidator, BooleanValidator, Datatable, DatatableColumn,
            DatatableColumnHint, DatatableHint, Hint, IntegerValidator, Node, Null,
            NumberValidator, Object, ObjectHint, Primitive, StringHint, Validator, Variable,
        },
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
        assert_eq!(messages[0].error_type.as_deref(), Some("SyntaxError"));
        assert_eq!(messages[0].message, "invalid syntax (<code>, line 1)");
        assert!(messages[0].stack_trace.is_some());
        assert_eq!(outputs, vec![]);

        // Runtime error
        let (outputs, messages) = kernel.execute("foo").await?;
        assert_eq!(messages[0].error_type.as_deref(), Some("NameError"));
        assert_eq!(messages[0].message, "name 'foo' is not defined");
        assert!(messages[0].stack_trace.is_some());
        assert_eq!(outputs, vec![]);

        Ok(())
    }

    /// Standard kernel test for getting runtime information
    #[test_log::test(tokio::test)]
    async fn info() -> Result<()> {
        let Some(instance) = create_instance::<PythonKernel>().await? else {
            return Ok(());
        };

        let sw = kernel_micro::tests::info(instance).await?;
        assert_eq!(sw.name, "python");
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
            println!("Skipping test because `numpy` not available");
            return Ok(());
        }

        let (.., messages) = instance
            .execute(
                "
a1 = np.array([True, False], dtype=np.bool_)
a2 = np.array([-1, 0, 1], dtype=np.int_)
a3 = np.array([1, 2 , 3], dtype=np.uint)
a4 = np.array([1.23, 4.56], dtype=np.float_)

# TODO: implement handling for these
#a5 = np.array(['2020-01-01', '2020-01-02', '2020-01-03'], dtype=np.datetime64)
#a6 = np.array([], dtype=np.timedelta64)
",
            )
            .await?;
        assert_eq!(messages, []);

        let list = instance.list().await?;

        macro_rules! var {
            ($name:expr) => {
                list.iter().find(|var| var.name == $name).unwrap().clone()
            };
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
            println!("Skipping test because `pandas` not available");
            return Ok(());
        }

        let (.., messages) = instance
            .execute(
                "
df1 = pd.DataFrame({
    'c1': [True, False],
    'c2': [1, 2],
    'c3': [1.23, 4.56]
})
",
            )
            .await?;
        assert_eq!(messages, []);

        let list = instance.list().await?;

        macro_rules! var {
            ($name:expr) => {
                list.iter().find(|var| var.name == $name).unwrap().clone()
            };
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
                        }
                    ]
                ))),
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
            println!("Skipping test because `pandas` not available");
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
            println!("Skipping test because `matplotlib` not available");
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
}
