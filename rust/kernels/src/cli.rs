use cli_utils::{
    AsFormat, Code, ToStdout,
    color_print::cstr,
    tabulated::{Attribute, Cell, CellAlignment, Color, Tabulated},
};
use kernel::{
    KernelAvailability, KernelProvider, KernelSpecification, KernelType,
    common::{
        clap::{self, Args, Parser, Subcommand},
        eyre::Result,
        itertools::Itertools,
        tracing,
    },
    format::Format,
    schema::{ExecutionBounds, ExecutionMessage, Node, NodeId, NodeType, StringOrNumber},
};
use node_diagnostics::{Diagnostic, DiagnosticKind, DiagnosticLevel};

use crate::Kernels;

/// Manage execution kernels
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># List all available kernels</dim>
  <b>stencila kernels</b>

  <dim># Get information about a specific kernel</dim>
  <b>stencila kernels info</b> <g>python</g>

  <dim># List packages available to a kernel</dim>
  <b>stencila kernels packages</b> <g>r</g>

  <dim># Execute code in a kernel</dim>
  <b>stencila kernels execute</b> <g>python</g> <y>\"print('Hello')\"</y>

  <dim># Lint code using a kernel's linting tool integrations</dim>
  <b>stencila kernels lint</b> <g>script.py</g>
"
);

#[derive(Debug, Subcommand)]
enum Command {
    List(List),
    Info(Info),
    Packages(Packages),
    Execute(Execute),
    Evaluate(Evaluate),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let Some(command) = self.command else {
            return List::default().run().await;
        };

        match command {
            Command::List(list) => list.run().await,
            Command::Info(info) => info.run().await,
            Command::Packages(pkgs) => pkgs.run().await,
            Command::Execute(exec) => exec.run().await,
            Command::Evaluate(eval) => eval.run().await,
        }
    }
}

/// List the kernels available
#[derive(Debug, Default, Args)]
#[command(after_long_help = LIST_AFTER_LONG_HELP)]
struct List {
    /// Only list kernels of a particular type
    #[arg(short, long)]
    r#type: Option<KernelType>,

    /// Output the list as JSON or YAML
    #[arg(long, short)]
    r#as: Option<AsFormat>,
}

impl List {
    async fn run(self) -> Result<()> {
        let list = super::list().await;

        let list = if let Some(type_) = self.r#type {
            list.into_iter()
                .filter(|kernel| kernel.r#type() == type_)
                .collect()
        } else {
            list
        };

        if let Some(format) = self.r#as {
            let list = list
                .into_iter()
                .map(|kernel| KernelSpecification::from(kernel.as_ref()))
                .collect_vec();

            Code::new_from(format.into(), &list)?.to_stdout();

            return Ok(());
        }

        let mut table = Tabulated::new();
        table.set_header([
            "Name",
            "Type",
            "Provider",
            "Availability",
            "Languages",
            "Bounds",
        ]);

        for kernel in list {
            let r#type = kernel.r#type();
            let provider = kernel.provider();
            let availability = kernel.availability();
            let langs = kernel
                .supports_languages()
                .iter()
                .map(|format| format.name())
                .join(", ");
            let bounds = kernel.supported_bounds();
            let max_bounds = bounds.iter().max().unwrap_or(&ExecutionBounds::Main);

            table.add_row([
                Cell::new(kernel.name()).add_attribute(Attribute::Bold),
                match r#type {
                    KernelType::Diagrams => Cell::new("diagrams").fg(Color::DarkYellow),
                    KernelType::Math => Cell::new("math").fg(Color::Blue),
                    KernelType::Programming => Cell::new("programming").fg(Color::Green),
                    KernelType::Database => Cell::new("database").fg(Color::DarkCyan),
                    KernelType::Styling => Cell::new("styling").fg(Color::Magenta),
                    KernelType::Templating => Cell::new("templating").fg(Color::Cyan),
                },
                match provider {
                    KernelProvider::Builtin => Cell::new("builtin").fg(Color::Green),
                    KernelProvider::Environment => Cell::new("environ").fg(Color::Cyan),
                    KernelProvider::Plugin(name) => {
                        Cell::new(format!("plugin \"{name}\"")).fg(Color::Blue)
                    }
                },
                Cell::new(availability).fg(match availability {
                    KernelAvailability::Available => Color::Green,
                    KernelAvailability::Disabled => Color::DarkBlue,
                    KernelAvailability::Installable => Color::Cyan,
                    KernelAvailability::Unavailable => Color::Grey,
                }),
                Cell::new(langs),
                Cell::new(max_bounds.to_string().to_lowercase()).fg(match max_bounds {
                    ExecutionBounds::Main => Color::Yellow,
                    ExecutionBounds::Fork => Color::Cyan,
                    ExecutionBounds::Box => Color::Green,
                }),
            ]);
        }

        table.to_stdout();

        Ok(())
    }
}

pub static LIST_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># List all available kernels</dim>
  <b>stencila kernels list</b>

  <dim># List only math kernels</dim>
  <b>stencila kernels list</b> <c>--type</c> <g>math</g>

  <dim># Output kernel list as YAML</dim>
  <b>stencila kernels list</b> <c>--as</c> <g>yaml</g>
"
);

/// Get information about a kernel
///
/// Mainly used to check the version of the kernel runtime and
/// operating system for debugging purpose.
#[derive(Debug, Args)]
#[command(after_long_help = INFO_AFTER_LONG_HELP)]
struct Info {
    /// The name of the kernel to get information for
    name: String,
}

impl Info {
    #[allow(clippy::print_stderr)]
    async fn run(self) -> Result<()> {
        let mut kernels = Kernels::new_here(ExecutionBounds::Main);
        let instance = kernels.create_instance(Some(&self.name)).await?;

        let info = instance.lock().await.info().await?;
        eprintln!(
            "Name: {}\nVersion: {}\nOperating system: {}\n",
            info.name,
            info.options.software_version.as_deref().unwrap_or("?"),
            info.options.operating_system.as_deref().unwrap_or("?"),
        );

        Ok(())
    }
}

pub static INFO_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Get information about the Python kernel</dim>
  <b>stencila kernels info</b> <g>python</g>

  <dim># Get information about the R kernel</dim>
  <b>stencila kernels info</b> <g>r</g>

  <dim># Get information about the JavaScript kernel</dim>
  <b>stencila kernels info</b> <g>javascript</g>
"
);

/// List packages available to a kernel
///
/// Mainly used to check libraries available to a kernel
/// for debugging purpose.
#[derive(Debug, Args)]
#[command(after_long_help = PACKAGES_AFTER_LONG_HELP)]
struct Packages {
    /// The name of the kernel to list packages for
    name: String,

    /// A filter on the name of the kernel
    ///
    /// Only packages whose name contains this string will be included
    /// (case insensitive)
    filter: Option<String>,
}

impl Packages {
    async fn run(self) -> Result<()> {
        let mut kernels = Kernels::new_here(ExecutionBounds::Main);
        let instance = kernels.create_instance(Some(&self.name)).await?;

        let packages = instance.lock().await.packages().await?;
        let packages = packages
            .into_iter()
            .filter(|package| {
                if let Some(filter) = &self.filter {
                    package.name.to_lowercase().contains(&filter.to_lowercase())
                } else {
                    true
                }
            })
            .sorted_by(|a, b| a.name.cmp(&b.name));

        let mut table = Tabulated::new();
        table.set_header(["Package", "Version"]);

        for package in packages {
            let version = match package.version.unwrap_or_default() {
                StringOrNumber::String(version) => version,
                StringOrNumber::Number(version) => version.to_string(),
            };
            table.add_row([
                Cell::new(package.name).add_attribute(Attribute::Bold),
                Cell::new(version).set_alignment(CellAlignment::Right),
            ]);
        }

        table.to_stdout();

        Ok(())
    }
}

pub static PACKAGES_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># List all packages available to Python kernel</dim>
  <b>stencila kernels packages</b> <g>python</g>

  <dim># Filter packages by name (case insensitive)</dim>
  <b>stencila kernels packages</b> <g>python</g> <g>numpy</g>

  <dim># List R packages containing 'plot'</dim>
  <b>stencila kernels packages</b> <g>r</g> <g>plot</g>
"
);

/// Execute code in a kernel
///
/// Creates a temporary kernel instance, executes one or more lines of code,
/// and returns any outputs and execution messages.
///
/// Mainly intended for quick testing of kernels during development.
#[derive(Debug, Args)]
#[clap(alias = "exec")]
#[command(after_long_help = EXECUTE_AFTER_LONG_HELP)]
struct Execute {
    /// The name of the kernel to execute code in
    name: String,

    /// The code to execute
    ///
    /// Escaped newline characters (i.e. "\n") in the code will be transformed into new lines
    /// before passing to the kernel.
    code: String,

    /// Execute code in a kernel instance with `Box` execution bounds
    #[arg(long, short)]
    r#box: bool,
}

impl Execute {
    async fn run(self) -> Result<()> {
        let bounds = if self.r#box {
            ExecutionBounds::Box
        } else {
            ExecutionBounds::Main
        };

        let mut kernels = Kernels::new_here(bounds);

        let code = self.code.replace("\\n", "\n");
        let (outputs, messages, instance) = kernels.execute(&code, Some(&self.name)).await?;

        tracing::debug!("Executed code in kernel instance: {instance}");

        display(NodeType::CodeChunk, self.code, messages, outputs)
    }
}

pub static EXECUTE_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Execute Python code</dim>
  <b>stencila kernels execute</b> <g>python</g> <y>\"print('Hello World')\"</y>

  <dim># Execute multi-line code with escaped newlines</dim>
  <b>stencila kernels execute</b> <g>python</g> <y>\"x = 5\\nprint(x * 2)\"</y>

  <dim># Execute code in a sandboxed environment</dim>
  <b>stencila kernels execute</b> <g>python</g> <y>\"import os\\nprint(os.environ)\"</y> <c>--box</c>

  <dim># Use the exec alias</dim>
  <b>stencila kernels exec</b> <g>r</g> <y>\"print(mean(c(1,2,3,4,5)))\"</y>
"
);

/// Evaluate a code expression in a kernel
///
/// Creates a temporary kernel instance, evaluates the expression in it,
/// and returns the output and any execution messages.
///
/// Mainly intended for quick testing of kernels during development.
#[derive(Debug, Args)]
#[clap(alias = "eval")]
#[command(after_long_help = EVALUATE_AFTER_LONG_HELP)]
struct Evaluate {
    /// The name of the kernel to evaluate code in
    name: String,

    /// The code expression to evaluate
    code: String,
}

impl Evaluate {
    async fn run(self) -> Result<()> {
        let mut kernels = Kernels::new_here(ExecutionBounds::Main);

        let (output, messages, instance) = kernels.evaluate(&self.code, Some(&self.name)).await?;

        tracing::debug!("Executed code in kernel instance: {instance}");

        display(NodeType::CodeExpression, self.code, messages, vec![output])
    }
}

pub static EVALUATE_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Evaluate a Python expression</dim>
  <b>stencila kernels evaluate</b> <g>python</g> <y>\"2 + 2\"</y>

  <dim># Evaluate an R expression</dim>
  <b>stencila kernels evaluate</b> <g>r</g> <y>\"sqrt(16)\"</y>

  <dim># Evaluate a JavaScript expression</dim>
  <b>stencila kernels evaluate</b> <g>javascript</g> <y>\"Math.PI * 2\"</y>

  <dim># Use the eval alias</dim>
  <b>stencila kernels eval</b> <g>python</g> <y>\"sum([1, 2, 3, 4, 5])\"</y>
"
);

fn display(
    node_type: NodeType,
    source: String,
    messages: Vec<ExecutionMessage>,
    outputs: Vec<Node>,
) -> Result<()> {
    for msg in messages {
        Diagnostic {
            node_type,
            node_id: NodeId::null(),
            node_property: None,
            level: DiagnosticLevel::from(&msg.level),
            kind: DiagnosticKind::Execution,
            error_type: msg.error_type.clone(),
            message: msg.message.clone(),
            format: None,
            code: None,
            code_location: msg.code_location.clone(),
        }
        .to_stderr("<code>", &source, &None)
        .ok();
    }

    for output in outputs {
        match output {
            Node::Datatable(dt) => dt.to_stdout(),
            _ => Code::new_from(Format::Yaml, &output)?.to_stdout(),
        };
    }

    Ok(())
}
