use std::sync::atomic::{AtomicU64, Ordering};

use kernel_micro::{
    common::eyre::Result, format::Format, Kernel, KernelAvailability, KernelForks, KernelInstance,
    KernelInterrupt, KernelKill, KernelTerminate, Microkernel,
};

/// A kernel for executing R code
#[derive(Default)]
pub struct RKernel {
    /// A counter of instances of this microkernel
    instances: AtomicU64,
}

impl Kernel for RKernel {
    fn name(&self) -> String {
        "r".to_string()
    }

    fn availability(&self) -> KernelAvailability {
        self.microkernel_availability()
    }

    fn supports_languages(&self) -> Vec<Format> {
        vec![Format::R]
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
        self.microkernel_supports_forks()
    }

    fn create_instance(&self) -> Result<Box<dyn KernelInstance>> {
        self.microkernel_create_instance(self.instances.fetch_add(1, Ordering::SeqCst))
    }
}

impl Microkernel for RKernel {
    fn executable_name(&self) -> String {
        "Rscript".to_string()
    }

    fn microkernel_script(&self) -> String {
        include_str!("kernel.r").to_string()
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
        schema::{
            Array, ArrayHint, ArrayValidator, BooleanValidator, Datatable, DatatableColumn,
            DatatableColumnHint, DatatableHint, EnumValidator, Hint, IntegerValidator, Node, Null,
            NumberValidator, Object, ObjectHint, Primitive, StringHint, StringValidator, Validator,
            Variable,
        },
        tests::{create_instance, start_instance},
    };

    use super::*;

    // Pro-tip! Use get logs for these tests use:
    //
    // ```sh
    // RUST_LOG=trace cargo test -p kernel-r -- --nocapture
    // ```

    /// Standard kernel test for execution of code
    #[test_log::test(tokio::test)]
    async fn execution() -> Result<()> {
        let Some(instance) = create_instance::<RKernel>().await? else {
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
print(2);
2 + 1",
                    vec![Node::Integer(1), Node::Integer(2), Node::Integer(3)],
                    vec![],
                ),
                // Imports in one code chunk are available in the next
                (
                    "
library(tools)
to_ignore_library_output <- TRUE
",
                    vec![],
                    vec![],
                ),
                (
                    "
grep(\"package:tools\", search()) > 0",
                    vec![Node::Boolean(true)],
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
print(a)
b",
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
        let Some(instance) = create_instance::<RKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::evaluation(
            instance,
            vec![
                ("1 + 1", Node::Integer(2), None),
                ("2.0 * 2.2", Node::Number(4.4), None),
                ("16 ** 0.5", Node::Integer(4), None),
                ("paste0('a', 'bc')", Node::String("abc".to_string()), None),
                (
                    "c(c(1, 2), 3)",
                    Node::Array(Array(vec![
                        Primitive::Integer(1),
                        Primitive::Integer(2),
                        Primitive::Integer(3),
                    ])),
                    None,
                ),
                (
                    "list(a=1, b=2.3)",
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
                    Some("<text>:1:1: unexpected '@'\n1: @\n    ^"),
                ),
                ("foo", Node::Null(Null), Some("object 'foo' not found")),
            ],
        )
        .await
    }

    /// Standard kernel test for printing nodes
    #[test_log::test(tokio::test)]
    async fn printing() -> Result<()> {
        let Some(instance) = create_instance::<RKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::printing(
            instance,
            r#"print('str')"#,
            r#"print('str1'); print('str2')"#,
            r#"print(NULL); print(TRUE); print(1); print(2.3); print('str'); print(list(1, 2.3, 'str')); print(list(a=1, b=2.3, c='str'))"#,
            r#"print(list(type='Paragraph', content=list()))"#,
        )
        .await
    }

    /// Custom test for execution messages
    #[tokio::test]
    async fn messages() -> Result<()> {
        let Some(mut kernel) = start_instance::<RKernel>().await? else {
            return Ok(());
        };

        // Syntax error
        let (outputs, messages) = kernel.execute("bad ^ # syntax").await?;
        assert_eq!(messages[0].error_type.as_deref(), Some("SyntaxError"));
        assert_eq!(
            messages[0].message,
            "<text>:2:0: unexpected end of input
1: bad ^ # syntax
   ^"
        );
        assert!(messages[0].stack_trace.is_none());
        assert_eq!(outputs, vec![]);

        // Runtime error
        let (outputs, messages) = kernel.execute("foo").await?;
        assert_eq!(messages[0].error_type.as_deref(), Some("RuntimeError"));
        assert_eq!(messages[0].message, "object 'foo' not found");
        assert!(messages[0].stack_trace.is_none());
        assert_eq!(outputs, vec![]);

        Ok(())
    }

    /// Standard kernel test for getting runtime information
    #[test_log::test(tokio::test)]
    async fn info() -> Result<()> {
        let Some(instance) = create_instance::<RKernel>().await? else {
            return Ok(());
        };

        let sw = kernel_micro::tests::info(instance).await?;
        assert_eq!(sw.name, "R");
        assert!(sw.options.software_version.is_some());
        assert!(sw.options.operating_system.is_some());

        Ok(())
    }

    /// Standard kernel test for listing installed packages
    #[test_log::test(tokio::test)]
    async fn packages() -> Result<()> {
        let Some(instance) = start_instance::<RKernel>().await? else {
            return Ok(());
        };

        let pkgs = kernel_micro::tests::packages(instance).await?;
        assert!(!pkgs.is_empty());

        Ok(())
    }

    /// Standard kernel test for variable listing
    #[test_log::test(tokio::test)]
    async fn var_listing() -> Result<()> {
        let Some(instance) = create_instance::<RKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::var_listing(
            instance,
            r#"
nul <- NULL
bool <- TRUE
int <- 123
num <- 1.23
str <- "abc👍"
arr <- c(1, 2, 3)
obj <- list(a=1, b=2.3)
para <- list(type='Paragraph', content=list())
"#,
            vec![
                Variable {
                    name: "nul".to_string(),
                    native_type: Some("NULL".to_string()),
                    node_type: Some("Null".to_string()),
                    programming_language: Some("R".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "bool".to_string(),
                    native_type: Some("logical".to_string()),
                    node_type: Some("Boolean".to_string()),
                    hint: Some(Hint::Boolean(true)),
                    programming_language: Some("R".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "int".to_string(),
                    native_type: Some("numeric".to_string()),
                    node_type: Some("Number".to_string()),
                    hint: Some(Hint::Integer(123)),
                    programming_language: Some("R".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "num".to_string(),
                    native_type: Some("numeric".to_string()),
                    node_type: Some("Number".to_string()),
                    hint: Some(Hint::Number(1.23)),
                    programming_language: Some("R".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "str".to_string(),
                    native_type: Some("character".to_string()),
                    node_type: Some("String".to_string()),
                    hint: Some(Hint::StringHint(StringHint::new(4))),
                    programming_language: Some("R".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "arr".to_string(),
                    native_type: Some("numeric".to_string()),
                    node_type: Some("Array".to_string()),
                    hint: Some(Hint::ArrayHint(ArrayHint {
                        length: 3,
                        item_types: Some(vec!["Number".to_string()]),
                        minimum: Some(Primitive::Integer(1)),
                        maximum: Some(Primitive::Integer(3)),
                        nulls: Some(0),
                        ..Default::default()
                    })),
                    programming_language: Some("R".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "obj".to_string(),
                    native_type: Some("list".to_string()),
                    node_type: Some("Object".to_string()),
                    hint: Some(Hint::ObjectHint(ObjectHint::new(
                        2,
                        vec!["a".to_string(), "b".to_string()],
                        vec![Hint::Integer(1), Hint::Number(2.3)],
                    ))),
                    programming_language: Some("R".to_string()),
                    ..Default::default()
                },
                Variable {
                    name: "para".to_string(),
                    native_type: Some("list".to_string()),
                    node_type: Some("Paragraph".to_string()),
                    programming_language: Some("R".to_string()),
                    ..Default::default()
                },
            ],
        )
        .await
    }

    /// Standard kernel test for variable management
    #[test_log::test(tokio::test)]
    async fn var_management() -> Result<()> {
        let Some(instance) = create_instance::<RKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::var_management(instance).await
    }

    /// `RKernel` specific test for `list` and `get` with `data.frame`s
    #[test_log::test(tokio::test)]
    async fn dataframe_list_get() -> Result<()> {
        let Some(mut instance) = start_instance::<RKernel>().await? else {
            return Ok(());
        };

        let (.., messages) = instance
            .execute(
                "
df1 = data.frame(
    c1 = c(TRUE, NA, FALSE),
    c2 = c(NA, 1, 2),
    c3 = c(1.23, NA, 4.56),
    c4 = c('a', 'b', NA),
    c5 = as.factor(c('c', NA, 'd')),
    stringsAsFactors = FALSE
)
",
            )
            .await?;
        assert_eq!(messages, []);

        let list = instance.list().await?;
        assert_eq!(
            list.iter().find(|var| var.name == "df1").unwrap(),
            &Variable {
                name: "df1".to_string(),
                native_type: Some("data.frame".to_string()),
                node_type: Some("Datatable".to_string()),
                hint: Some(Hint::DatatableHint(DatatableHint::new(
                    3,
                    vec![
                        DatatableColumnHint {
                            name: "c1".to_string(),
                            item_type: "Boolean".to_string(),
                            nulls: Some(1),
                            ..Default::default()
                        },
                        DatatableColumnHint {
                            name: "c2".to_string(),
                            item_type: "Number".to_string(),
                            minimum: Some(Primitive::Integer(1)),
                            maximum: Some(Primitive::Integer(2)),
                            nulls: Some(1),
                            ..Default::default()
                        },
                        DatatableColumnHint {
                            name: "c3".to_string(),
                            item_type: "Number".to_string(),
                            minimum: Some(Primitive::Number(1.23)),
                            maximum: Some(Primitive::Number(4.56)),
                            nulls: Some(1),
                            ..Default::default()
                        },
                        DatatableColumnHint {
                            name: "c4".to_string(),
                            item_type: "String".to_string(),
                            minimum: Some(Primitive::String("a".to_string())),
                            maximum: Some(Primitive::String("b".to_string())),
                            nulls: Some(1),
                            ..Default::default()
                        },
                        DatatableColumnHint {
                            name: "c5".to_string(),
                            item_type: "String".to_string(),
                            nulls: Some(1),
                            ..Default::default()
                        }
                    ]
                ))),
                programming_language: Some("R".to_string()),
                ..Default::default()
            },
        );

        assert_eq!(
            instance.get("df1").await?.unwrap(),
            Node::Datatable(Datatable::new(vec![
                DatatableColumn {
                    name: "c1".to_string(),
                    values: vec![
                        Primitive::Boolean(true),
                        Primitive::Null(Null),
                        Primitive::Boolean(false)
                    ],
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
                    values: vec![
                        Primitive::Null(Null),
                        Primitive::Integer(1),
                        Primitive::Integer(2)
                    ],
                    validator: Some(ArrayValidator {
                        items_validator: Some(Box::new(Validator::NumberValidator(
                            NumberValidator::new()
                        ))),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                DatatableColumn {
                    name: "c3".to_string(),
                    values: vec![
                        Primitive::Number(1.23),
                        Primitive::Null(Null),
                        Primitive::Number(4.56)
                    ],
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
                    values: vec![
                        Primitive::String("a".to_string()),
                        Primitive::String("b".to_string()),
                        Primitive::Null(Null),
                    ],
                    validator: Some(ArrayValidator {
                        items_validator: Some(Box::new(Validator::StringValidator(
                            StringValidator::new()
                        ))),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                DatatableColumn {
                    name: "c5".to_string(),
                    values: vec![
                        Primitive::String("c".to_string()),
                        Primitive::Null(Null),
                        Primitive::String("d".to_string())
                    ],
                    validator: Some(ArrayValidator {
                        items_validator: Some(Box::new(Validator::EnumValidator(
                            EnumValidator::new(vec![
                                Node::String("c".to_string()),
                                Node::String("d".to_string())
                            ])
                        ))),
                        ..Default::default()
                    }),
                    ..Default::default()
                }
            ]))
        );

        Ok(())
    }

    /// `RKernel` specific test to test round-trip `set`/`get` with `data.frame`s
    #[test_log::test(tokio::test)]
    async fn dataframe_set_get() -> Result<()> {
        let Some(mut instance) = start_instance::<RKernel>().await? else {
            return Ok(());
        };

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

        let (output, messages) = instance.evaluate("class(dt)").await?;
        assert_eq!(messages, []);
        assert_eq!(output, Node::String("data.frame".to_string()));

        let dt_out = instance.get("dt").await?;
        assert_eq!(dt_out, dt_out);

        Ok(())
    }

    /// `RKernel` specific test for getting a `plot` as output
    #[test_log::test(tokio::test)]
    async fn plot() -> Result<()> {
        let Some(mut instance) = start_instance::<RKernel>().await? else {
            return Ok(());
        };

        let (outputs, messages) = instance.execute("plot(1)").await?;
        assert_eq!(messages, []);
        assert_eq!(outputs.len(), 1);

        if let Some(Node::ImageObject(image)) = outputs.get(0) {
            assert!(image.content_url.starts_with("data:image/png;base64"));
        } else {
            bail!("Expected an image, got: {outputs:?}")
        }

        Ok(())
    }

    /// Standard kernel test for forking
    #[test_log::test(tokio::test)]
    async fn forking() -> Result<()> {
        let Some(instance) = create_instance::<RKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::forking(instance).await
    }

    /// Custom test to check that modules imported in the main kernel instance are
    /// available in the forked instance
    #[test_log::test(tokio::test)]
    async fn forking_imports() -> Result<()> {
        let Some(mut instance) = start_instance::<RKernel>().await? else {
            return Ok(());
        };

        let (outputs, messages) = instance
            .execute(
                r#"
library(tools)

class(toRd)
"#,
            )
            .await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs, vec![Node::String("function".to_string())]);

        let mut fork = instance.fork().await?;
        let (outputs, messages) = fork
            .execute(
                r#"
class(toRd)
"#,
            )
            .await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs, vec![Node::String("function".to_string())]);

        Ok(())
    }

    /// Standard kernel test for signals
    #[test_log::test(tokio::test)]
    async fn signals() -> Result<()> {
        let Some(instance) = create_instance::<RKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::signals(
            instance,
            "
# Setup step
Sys.sleep(0.1)
value <- 1
value",
            Some(
                "
# Interrupt step
Sys.sleep(100)
value = 2",
            ),
            None,
            Some(
                "
# Kill step
Sys.sleep(100)",
            ),
        )
        .await
    }

    /// Standard kernel test for stopping
    #[test_log::test(tokio::test)]
    async fn stop() -> Result<()> {
        let Some(instance) = create_instance::<RKernel>().await? else {
            return Ok(());
        };

        kernel_micro::tests::stop(instance).await
    }
}
