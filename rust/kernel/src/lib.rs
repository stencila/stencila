use std::path::Path;

use common::{
    async_trait::async_trait,
    eyre::{bail, Result},
    strum::Display,
    tokio::sync::{mpsc, watch},
};
use format::Format;

// Re-exports for the convenience of internal crates implementing
// the `Kernel` trait
pub use common;
pub use format;
pub use schema;
use schema::{ExecutionMessage, Node, SoftwareApplication, SoftwareSourceCode, Variable};

/// A kernel for executing code in some language
///
/// Provides a common, shared interface for the various execution kernels
/// including those that use embedded languages (e.g. Rhai, Lua), those that
/// connect to databases to execute SQL (e.g. SQLite, Postgres, DuckDB),
/// Stencila 'microkernels', and Jupyter kernels.
///
/// This trait specifies the kernel and its capabilities (similar to a Jupyter "kernel spec")
/// The `KernelInstance` trait is the interface for instances of kernels.
pub trait Kernel: Sync + Send {
    /// Get the id of the kernel
    ///
    /// This id should be unique amongst all kernels.
    fn id(&self) -> String;

    /// Get the availability of the kernel on the current machine
    fn availability(&self) -> KernelAvailability;

    /// Is the kernel available on the current machine
    fn is_available(&self) -> bool {
        matches!(self.availability(), KernelAvailability::Available)
    }

    /// Get the languages supported by the kernel
    fn supports_languages(&self) -> Vec<Format>;

    /// Does the kernel support a particular language?
    fn supports_language(&self, format: &Format) -> bool {
        self.supports_languages().contains(format)
    }

    /// Does the kernel support the interrupt signal?
    fn supports_interrupt(&self) -> KernelInterrupt;

    /// Does the kernel support the terminate signal?
    fn supports_terminate(&self) -> KernelTerminate;

    /// Does the kernel support the kill signal?
    fn supports_kill(&self) -> KernelKill;

    /// Does the kernel support forking?
    fn supports_forks(&self) -> KernelForks;

    /// Create a new instance of the kernel
    fn create_instance(&self) -> Result<Box<dyn KernelInstance>>;
}

/// The availability of a kernel on the current machine
#[derive(Display)]
#[strum(serialize_all = "lowercase")]
pub enum KernelAvailability {
    /// Available on this machine
    Available,
    /// Available on this machine but requires installation
    Installable,
    /// Not available on this machine
    Unavailable,
}

/// Whether a kernel supports the interrupt signal on the current machine
///
/// The interrupt signal is used to stop the execution task the
/// kernel instance is current performing.
#[derive(Display)]
#[strum(serialize_all = "lowercase")]
pub enum KernelInterrupt {
    /// Kernel supports interrupt signal on this machine
    Yes,
    /// Kernel does not support interrupt signal on this machine
    No,
}

/// Whether a kernel supports the terminate signal on the current machine
///
/// The terminate signal is used to stop the kernel instance gracefully
/// (e.g. completing any current execution tasks)
#[derive(Display)]
#[strum(serialize_all = "lowercase")]
pub enum KernelTerminate {
    /// Kernel supports terminate signal on this machine
    Yes,
    /// Kernel does not support terminate signal on this machine
    No,
}

/// Whether a kernel supports the kill signal on the current machine
///
/// The kill signal is used to stop the kernel instance forcefully
/// (i.e. to exit immediately, aborting any current execution tasks)
#[derive(Display)]
#[strum(serialize_all = "lowercase")]
pub enum KernelKill {
    /// Kernel supports kill signal on this machine
    Yes,
    /// Kernel does not support kill signal on this machine
    No,
}

/// Whether a kernel supports forking on the current machine
#[derive(Display)]
#[strum(serialize_all = "lowercase")]
pub enum KernelForks {
    /// Kernel supports forking on this machine
    Yes,
    /// Kernel does not support forking on this machine
    No,
}

/// An instance of a kernel
#[async_trait]
pub trait KernelInstance: Sync + Send {
    /// Get the id of the kernel instance
    ///
    /// This id should be unique amongst all kernel instances,
    /// including those for other `Kernel`s.
    fn id(&self) -> String;

    /// Get the status of the kernel instance
    async fn status(&self) -> Result<KernelStatus>;

    /// Get a watcher of the status of the kernel instance
    fn watcher(&self) -> Result<watch::Receiver<KernelStatus>>;

    /// Get a signaller to interrupt or kill the kernel instance
    fn signaller(&self) -> Result<mpsc::Sender<KernelSignal>>;

    /// Start the kernel in a working directory
    async fn start(&mut self, directory: &Path) -> Result<()>;

    /// Start the kernel in the current working directory
    async fn start_here(&mut self) -> Result<()> {
        self.start(&std::env::current_dir()?).await
    }

    /// Stop the kernel
    async fn stop(&mut self) -> Result<()>;

    /// Execute code, possibly with side effects, in the kernel instance
    async fn execute(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionMessage>)>;

    /// Evaluate a code expression, without side effects, in the kernel instance
    async fn evaluate(&mut self, code: &str) -> Result<(Node, Vec<ExecutionMessage>)>;

    /// Get runtime information about the kernel instance
    async fn info(&mut self) -> Result<SoftwareApplication>;

    /// Get a list of packages available in the kernel instance
    async fn packages(&mut self) -> Result<Vec<SoftwareSourceCode>>;

    /// Get a list of variables in the kernel instance
    async fn list(&mut self) -> Result<Vec<Variable>>;

    /// Get a variable from the kernel instance
    async fn get(&mut self, name: &str) -> Result<Option<Node>>;

    /// Set a variable in the kernel instance
    async fn set(&mut self, name: &str, value: &Node) -> Result<()>;

    /// Remove a variable from the kernel instance
    async fn remove(&mut self, name: &str) -> Result<()>;

    /// Create a fork of the kernel instance
    async fn fork(&mut self) -> Result<Box<dyn KernelInstance>> {
        bail!("Kernel `{}` does not support forks", self.id())
    }
}

/// The status of a kernel instance
#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display)]
#[strum(serialize_all = "lowercase")]
pub enum KernelStatus {
    #[default]
    Pending,
    Starting,
    Ready,
    Busy,
    Stopping,
    Stopped,
    Failed,
}

impl From<KernelStatus> for u8 {
    fn from(status: KernelStatus) -> Self {
        status as u8
    }
}

impl From<u8> for KernelStatus {
    fn from(value: u8) -> Self {
        use KernelStatus::*;
        match value {
            0 => Pending,
            1 => Starting,
            2 => Ready,
            3 => Busy,
            4 => Stopping,
            5 => Stopped,
            6 => Failed,
            _ => Pending,
        }
    }
}

/// A signal to send to a kernel instance
#[derive(Clone, Copy)]
pub enum KernelSignal {
    Interrupt,
    Terminate,
    Kill,
}

/// Standard tests for implementations of the `Kernel` and `KernelInstance` traits
pub mod tests {
    use std::{env, time::Duration};

    use common::{eyre::Report, indexmap::IndexMap, itertools::Itertools, tokio, tracing};
    use common_dev::pretty_assertions::assert_eq;
    use schema::{Array, Null, Object, Paragraph, Primitive, SoftwareApplication};

    use super::*;

    /// Create a new kernel instance if available
    pub async fn create_instance<K>() -> Result<Option<Box<dyn KernelInstance>>>
    where
        K: Default + Kernel,
    {
        let kernel = K::default();
        match kernel.availability() {
            KernelAvailability::Available => Ok(Some(kernel.create_instance()?)),
            _ => Ok(None),
        }
    }

    /// Create and start a new kernel instance if available
    pub async fn start_instance<K>() -> Result<Option<Box<dyn KernelInstance>>>
    where
        K: Default + Kernel,
    {
        match create_instance::<K>().await? {
            Some(mut instance) => {
                instance.start_here().await?;
                Ok(Some(instance))
            }
            _ => Ok(None),
        }
    }

    /// Test execution of code by a kernel instance
    ///
    /// All kernel instances must implement this method. This tests is
    /// passed a vector of test cases of code chunks and checks for
    /// expected output nodes and any message.
    ///
    /// The following are examples of things that should be tested using this.
    ///
    /// - variables set in one chunk are available in following chunks
    /// - packages imported in one chunk are available in following chunks
    /// - if the last line is an expression it is returned as a value
    /// - if outputs are printed, they should be separate to the returned last expression value
    pub async fn execution(
        mut instance: Box<dyn KernelInstance>,
        cases: Vec<(&str, Vec<Node>, Vec<&str>)>,
    ) -> Result<()> {
        instance.start_here().await?;
        assert_eq!(instance.status().await?, KernelStatus::Ready);

        for (code, expected_outputs, expected_messages) in cases {
            let (outputs, messages) = instance.execute(code).await?;
            assert_eq!(
                messages
                    .iter()
                    .map(|message| message.message.to_string())
                    .collect_vec(),
                expected_messages
                    .iter()
                    .map(|message| message.to_string())
                    .collect_vec(),
                "with code: {code}"
            );
            assert_eq!(outputs, expected_outputs);
        }

        Ok(())
    }

    /// Test evaluation of expressions by a kernel instance
    ///
    /// All kernel instances must implement this method. This tests is
    /// passed a vector of test cases and checks for expected output node
    /// and any message (in case where there is an error).
    pub async fn evaluation(
        mut instance: Box<dyn KernelInstance>,
        cases: Vec<(&str, Node, Option<&str>)>,
    ) -> Result<()> {
        instance.start_here().await?;
        assert_eq!(instance.status().await?, KernelStatus::Ready);

        for (code, expected_output, expected_message) in cases {
            let (output, messages) = instance.evaluate(code).await?;
            assert_eq!(
                messages.first().map(|message| message.message.as_ref()),
                expected_message,
                "with expression: {code}"
            );
            assert_eq!(output, expected_output);
        }

        Ok(())
    }

    /// Test getting runtime info
    pub async fn info(mut instance: Box<dyn KernelInstance>) -> Result<SoftwareApplication> {
        instance.start_here().await?;
        assert_eq!(instance.status().await?, KernelStatus::Ready);

        instance.info().await
    }

    /// Test getting list of packages
    pub async fn packages(
        mut instance: Box<dyn KernelInstance>,
    ) -> Result<Vec<SoftwareSourceCode>> {
        instance.start_here().await?;
        assert_eq!(instance.status().await?, KernelStatus::Ready);

        instance.packages().await
    }

    /// Test printing of nodes by a kernel instance
    ///
    /// Kernels implementations are encouraged to override the usual
    /// print function/statement so that arguments are outputted from the
    /// task as separate Stencila nodes rather than a blob of test.
    ///
    /// See arg names below and example usage of this function in the `kernel-*` crates for
    /// what the code in each task should print to match expected output nodes.
    pub async fn printing(
        mut instance: Box<dyn KernelInstance>,
        string: &str,
        strings: &str,
        null_bool_int_num_string_arr_obj: &str,
        paragraph: &str,
    ) -> Result<()> {
        instance.start_here().await?;
        assert_eq!(instance.status().await?, KernelStatus::Ready);

        let (outputs, messages) = instance.execute(string).await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs, vec![Node::String("str".to_string())]);

        let (outputs, messages) = instance.execute(strings).await?;
        assert_eq!(messages, vec![]);
        assert_eq!(
            outputs,
            vec![
                Node::String("str1".to_string()),
                Node::String("str2".to_string())
            ]
        );

        let (outputs, messages) = instance.execute(null_bool_int_num_string_arr_obj).await?;
        assert_eq!(messages, vec![]);
        assert_eq!(
            outputs,
            vec![
                Node::Null(Null),
                Node::Boolean(true),
                Node::Integer(1),
                Node::Number(2.3),
                Node::String("str".to_string()),
                Node::Array(Array(vec![
                    Primitive::Integer(1),
                    Primitive::Number(2.3),
                    Primitive::String("str".to_string())
                ])),
                Node::Object(Object(IndexMap::from([
                    (String::from("a"), Primitive::Integer(1),),
                    (String::from("b"), Primitive::Number(2.3)),
                    (String::from("c"), Primitive::String("str".to_string()))
                ])))
            ]
        );

        let (outputs, messages) = instance.execute(paragraph).await?;
        assert_eq!(messages, vec![]);
        assert_eq!(outputs, vec![Node::Paragraph(Paragraph::new(vec![]))]);

        Ok(())
    }

    /// Test listing of variables
    ///
    /// All kernel instances must implement the `list` method
    /// for retrieving a list of variables including their names, types
    /// and hints on their values (e.g. lengths of array, column names etc).
    ///
    /// This function should be passed a code chunk that sets variables of various types
    /// and a vector of expected variables.
    pub async fn var_listing(
        mut instance: Box<dyn KernelInstance>,
        code: &str,
        expected_vars: Vec<Variable>,
    ) -> Result<()> {
        instance.start_here().await?;
        assert_eq!(instance.status().await?, KernelStatus::Ready);

        let (.., messages) = instance.execute(code).await?;
        assert_eq!(messages, vec![]);

        let actual_vars = instance.list().await?;
        for expected in expected_vars {
            let Some(actual) = actual_vars
                .iter()
                .find(|actual| actual.name == expected.name)
            else {
                bail!("no variable named `{}` in list", expected.name);
            };
            assert_eq!(actual, &expected)
        }

        Ok(())
    }

    /// Test managing of variables using `set`, `get` and `remove`
    ///
    /// All kernel instances must implement these methods by setting, getting
    /// and removing variables of different types.
    pub async fn var_management(mut instance: Box<dyn KernelInstance>) -> Result<()> {
        instance.start_here().await?;
        assert_eq!(instance.status().await?, KernelStatus::Ready);

        // List existing vars
        let initial = instance.list().await?;

        for (name, value) in [
            ("bool_", Node::Boolean(true)),
            ("int_", Node::Integer(123)),
            ("num_", Node::Number(1.23)),
            ("string_", Node::String("str".to_string())),
            (
                "array_",
                Node::Array(Array(vec![
                    Primitive::Integer(1),
                    Primitive::Number(2.3),
                    Primitive::String("str".to_string()),
                ])),
            ),
            (
                "object_",
                Node::Object(Object(IndexMap::from([
                    (String::from("a"), Primitive::Integer(1)),
                    (String::from("b"), Primitive::Number(2.3)),
                    (String::from("c"), Primitive::String("str".to_string())),
                ]))),
            ),
        ] {
            // Set a var
            instance.set(name, &value).await?;
            let vars = instance.list().await?;
            assert_eq!(vars.len(), initial.len() + 1);
            assert!(vars.iter().any(|var| var.name == name));

            // Get the var
            assert_eq!(instance.get(name).await?, Some(value));

            // Remove the var
            instance.remove(name).await?;
            assert_eq!(instance.get(name).await?, None);
            let vars = instance.list().await?;
            assert_eq!(vars.len(), initial.len());
            assert!(!vars.iter().any(|var| var.name == name));
        }

        Ok(())
    }

    /// Test forking a kernel instance
    pub async fn forking(mut instance: Box<dyn KernelInstance>) -> Result<()> {
        instance.start_here().await?;
        assert_eq!(instance.status().await?, KernelStatus::Ready);

        // Set variables in the kernel
        instance.set("var1", &Node::Integer(123)).await?;
        instance.set("var2", &Node::Number(4.56)).await?;
        instance
            .set("var3", &Node::String("Hello from main".to_string()))
            .await?;

        // Create a fork and check that the variables are available in it
        let mut fork = instance.fork().await?;
        assert_eq!(fork.get("var1").await?, Some(Node::Integer(123)));
        assert_eq!(fork.get("var2").await?, Some(Node::Number(4.56)));
        assert_eq!(
            fork.get("var3").await?,
            Some(Node::String("Hello from main".to_string()))
        );

        // Change variables in fork
        fork.set("var1", &Node::Integer(321)).await?;
        fork.remove("var2").await?;
        let (.., messages) = fork.execute("var3=\"Hello from fork\"").await?;
        assert_eq!(messages, vec![]);

        // Check changes had effect in fork
        assert_eq!(fork.get("var1").await?, Some(Node::Integer(321)));
        assert_eq!(fork.get("var2").await?, None);
        assert_eq!(
            fork.get("var3").await?,
            Some(Node::String("Hello from fork".to_string()))
        );

        // Check that variables are unchanged in main instance
        assert_eq!(instance.get("var1").await?, Some(Node::Integer(123)));
        assert_eq!(instance.get("var2").await?, Some(Node::Number(4.56)));
        assert_eq!(
            instance.get("var3").await?,
            Some(Node::String("Hello from main".to_string()))
        );

        Ok(())
    }

    /// Test sending asynchronous signals to a kernel instance
    ///
    /// Kernel implementations can handle interrupt, terminate and kill signals to varying degrees
    /// (e.g. some implementations handle all, some may only handle one).
    ///
    /// This tests the signal handling logic of an implementation of `KernelInstance` trait.
    /// In addition it also tests the asynchronous watching of kernel status.
    ///
    /// The kernel instance passed to this function is expected to have status `pending`
    /// (i.e. to have not been started yet). See example usage of this function in the
    /// `kernel-*` crates for what the code in each step should do.
    pub async fn signals(
        mut instance: Box<dyn KernelInstance>,
        setup_step: &str,
        interrupt_step: Option<&str>,
        terminate_step: Option<&str>,
        kill_step: Option<&str>,
    ) -> Result<()> {
        let mut watcher = instance.watcher()?;

        // Convenience macro for checking changes in the watched status
        macro_rules! assert_status_changed {
            ($status:ident) => {
                watcher.changed().await?;
                assert_eq!(*watcher.borrow_and_update(), KernelStatus::$status);
            };
        }

        // Kernel should be passed to this function as pending
        assert_eq!(instance.status().await?, KernelStatus::Pending);
        assert_eq!(*watcher.borrow_and_update(), KernelStatus::Pending);

        // Start kernel and check for status changes
        instance.start_here().await?;
        assert_status_changed!(Ready);
        assert_eq!(instance.status().await?, KernelStatus::Ready);

        // Signaller is only available once kernel has started
        let signaller = instance.signaller()?;

        // Move the kernel into a task so we can asynchronously do things within it.
        // The "step" channel helps coordinate with this thread.
        // We collect errors and return them to main thread so they can be
        // asserted to be empty.
        let (step_sender, mut step_receiver) = mpsc::channel::<()>(1);
        let setup_step = setup_step.to_owned();
        let has_interrupt_step = interrupt_step.is_some();
        let interrupt_step = interrupt_step.map(|value| value.to_owned());
        let has_terminate_step = terminate_step.is_some();
        let terminate_step = terminate_step.map(|value| value.to_owned());
        let has_kill_step = kill_step.is_some();
        let kill_step = kill_step.map(|value| value.to_owned());
        let task = tokio::spawn(async move {
            let mut errors = Vec::new();

            // Macro to both log and collect errors
            macro_rules! error {
                ($($arg:tt)*) => {{
                    tracing::error!($($arg)*);
                    errors.push(format!($($arg)*));
                }};
            }

            // Setup step
            step_receiver.recv().await.unwrap();
            let (outputs, messages) = instance.execute(&setup_step).await?;
            if !messages.is_empty() {
                error!("Unexpected messages in setup step: {messages:?}")
            }
            let initial_value = outputs.first().cloned();
            if initial_value.is_none() {
                error!("Setup step did not return a value")
            }
            let status = instance.status().await?;
            if status != KernelStatus::Ready {
                error!("Unexpected status after setup step: {status}")
            }

            if let Some(interrupt_step) = interrupt_step {
                // Interrupt step
                step_receiver.recv().await.unwrap();
                let (.., messages) = instance.execute(&interrupt_step).await?;
                if !messages.is_empty() {
                    error!("Unexpected messages in interrupt step: {messages:?}")
                }
                let status = instance.status().await?;
                if status != KernelStatus::Ready {
                    error!("Unexpected status after interrupt step: {status}")
                }

                // Value should not have changed because task was interrupted
                // before it completed
                step_receiver.recv().await.unwrap();
                let value = instance.get("value").await?;
                if value != initial_value {
                    error!("Unexpected value after interrupt step: {value:?} !== {initial_value:?}")
                }
            }

            if let Some(terminate_step) = terminate_step {
                // Terminate step
                step_receiver.recv().await.unwrap();
                let (.., messages) = instance.execute(&terminate_step).await?;
                if !messages.is_empty() {
                    error!("Unexpected messages in terminate step: {messages:?}")
                }
                let status = instance.status().await?;
                if status != KernelStatus::Stopped {
                    error!("Unexpected status after terminate step: {status}")
                }
            } else if let Some(kill_step) = kill_step {
                // Kill step
                step_receiver.recv().await.unwrap();
                let (.., messages) = instance.execute(&kill_step).await?;
                if !messages.is_empty() {
                    error!("Unexpected messages in kill step: {messages:?}")
                }
                // Wait a little, to allow async kill signal to actually take effect,
                // before confirming status
                tokio::time::sleep(Duration::from_millis(if env::var("CI").is_ok() {
                    1000
                } else {
                    200
                }))
                .await;
                let status = instance.status().await?;
                if status != KernelStatus::Failed {
                    error!("Unexpected status after kill step: {status}")
                }
            }

            let status = instance.status().await?;
            Ok::<_, Report>((status, errors))
        });

        // Iterate over steps, sending signals and checking that status changes as expected

        {
            // Should have busy/ready status changes during setup step
            step_sender.send(()).await?;
            assert_status_changed!(Busy);
            assert_status_changed!(Ready);
        }

        if has_interrupt_step {
            // Should be busy at start of interrupt step
            step_sender.send(()).await?;
            assert_status_changed!(Busy);

            // Interrupt (if this fails then the test would keep running)
            signaller.send(KernelSignal::Interrupt).await?;

            // Should be ready after interrupt
            assert_status_changed!(Ready);

            // Should have busy/ready status changes during get
            step_sender.send(()).await?;
            assert_status_changed!(Busy);
            assert_status_changed!(Ready);
        }

        let expected_status = if has_terminate_step {
            // Should be busy at start of terminate step
            step_sender.send(()).await?;
            assert_status_changed!(Busy);

            // Terminate (if this fails then the test would keep running)
            signaller.send(KernelSignal::Terminate).await?;

            KernelStatus::Stopped
        } else if has_kill_step {
            // Should be busy at start of kill step
            step_sender.send(()).await?;
            assert_status_changed!(Busy);

            // Kill (if this fails then the test would keep running)
            signaller.send(KernelSignal::Kill).await?;

            KernelStatus::Failed
        } else {
            KernelStatus::Ready
        };

        // Should have finished the task with correct status and no errors
        let (status, errors) = task.await??;
        assert_eq!(status, expected_status);
        assert_eq!(errors, Vec::<String>::new());

        Ok(())
    }

    /// Test stopping a kernel instance
    pub async fn stop(mut instance: Box<dyn KernelInstance>) -> Result<()> {
        instance.start_here().await?;
        assert_eq!(instance.status().await?, KernelStatus::Ready);

        instance.stop().await?;
        assert_eq!(instance.status().await?, KernelStatus::Stopped);

        Ok(())
    }
}
