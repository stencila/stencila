//! Provides the `LatexCodec` trait for generating Latex for Stencila Schema nodes

use std::{
    env::temp_dir,
    fs::{create_dir_all, read_to_string, remove_file, write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use eyre::{Result, bail};
use itertools::Itertools;
use rand::{Rng, distr::Alphanumeric, rng};

use stencila_codec_info::{EncodeInfo, Losses, Mapping, NodeId, NodeProperty, NodeType};
use stencila_codec_utils::{move_file, split_paragraph};
use stencila_format::Format;
use stencila_node_path::{NodePath, NodeSlot};
use stencila_node_url::{NodePosition, NodeUrl};

pub use stencila_codec_latex_derive::LatexCodec;

mod escape;
use escape::escape_latex;

/// Encode a node that implements `LatexCodec` to Latex
///
/// A convenience function to save the caller from having to create a context etc.
///
/// Note: the `format` argument is the final destination format. LaTeX is always generated,
/// but it will differ depending on the destination format.
pub fn to_latex<T>(
    node: &T,
    format: Format,
    standalone: bool,
    render: bool,
    highlight: bool,
    reproducible: bool,
) -> (String, EncodeInfo)
where
    T: LatexCodec,
{
    let mut context = LatexEncodeContext::new(format, standalone, render, highlight, reproducible);
    node.to_latex(&mut context);

    let mut latex = context.content;

    if standalone {
        // Ensure necessary syntax for standalone, including any missing use packages

        if !latex.contains("\\end{document}") {
            latex.push_str("\\end{document}");
        }

        if !latex.contains("\\begin{document}") {
            latex.insert_str(0, "\\begin{document}\n\n");
        }

        if !latex.contains("\\documentclass") {
            latex.insert_str(0, "\\documentclass{article}\n\n");
        }

        let pkgs = use_packages(&latex);
        if !pkgs.trim().is_empty() {
            // Insert any missing package definitions after document class (ensured above)
            if let Some(pos) = latex.find("}\n") {
                latex.insert_str(pos + 2, &["\n", &pkgs, "\n"].concat());
            }
        }
    }

    if latex.ends_with("\n\n") {
        latex.pop();
    }

    let info = EncodeInfo {
        losses: context.losses,
        mapping: context.mapping,
    };

    (latex, info)
}

/// Generate `\usepackage` commands necessary for the included LaTeX
///
/// Looks for commonly used commands and environments and returns `\usepackage`
/// commands for the corresponding packages.
pub fn use_packages(latex: &str) -> String {
    let mut packages = Vec::new();

    // Helper to check for usage of packages
    let has = |usage: &str| latex.contains(usage);

    // Helper to check for either \usepackage or \RequirePackage (with or without options)
    let has_pkg = |pkg: &str| {
        latex.contains(&format!(r"\usepackage{{{pkg}}}"))
            || latex.contains(&format!(r"\RequirePackage{{{pkg}}}"))
            || latex.contains(&r"\usepackage[".to_string()) && latex.contains(pkg)
    };

    // hyperref: links & urls
    if (has(r"\href") || has(r"\url") || has(r"\autoref") || has(r"\ref")) && !has_pkg("hyperref") {
        packages.push("hyperref");
    }
    // graphicx: images
    if has(r"\includegraphics") && !has_pkg("graphicx") {
        packages.push("graphicx");
    }
    // amsmath: display‐math environments
    if (has(r"\begin{align}") || has(r"\begin{equation}") || has(r"\[") || has(r"\]"))
        && !has_pkg("amsmath")
    {
        packages.push("amsmath");
    }
    // amssymb: extra math symbols (\mathbb, \mathcal, etc.)
    if (has(r"\mathbb") || has(r"\mathcal") || has(r"\mathfrak")) && !has_pkg("amssymb") {
        packages.push("amssymb");
    }
    // array: custom column types (\newcolumntype)
    if has(r"\newcolumntype") && !has_pkg("array") {
        packages.push("array");
    }
    // xcolor: color support
    if (has(r"\color") || has(r"\textcolor") || has(r"\definecolor") || has(r"\rowcolors"))
        && !has_pkg("xcolor")
    {
        packages.push("\\usepackage[table]{xcolor}");
    }
    // soul: text highlighting (\hl, \sethlcolor)
    if (has(r"\hl") || has(r"\sethlcolor")) && !has_pkg("soul") {
        packages.push("soul");
    }
    // colortbl: colored table rules (\arrayrulecolor)
    if has(r"\arrayrulecolor") && !has_pkg("colortbl") {
        packages.push("colortbl");
    }
    // geometry: page geometry
    if (has(r"\newgeometry") || has(r"\geometry{")) && !has_pkg("geometry") {
        packages.push("geometry");
    }
    // pdflscape: landscape pages
    if has(r"\begin{landscape}") && !has_pkg("pdflscape") {
        packages.push("pdflscape");
    }
    // placeins: \FloatBarrier
    if has(r"\FloatBarrier") && !has_pkg("placeins") {
        packages.push("placeins");
    }
    // floatrow: float configuration & utilities (\floatsetup, floatrow env, ffigbox, ttabbox, capbeside, fcapside, floatbox, floatfoot, DeclareNewFloatType, restylefloat, newfloatcommand)
    if (has(r"\floatsetup")
        || has(r"\begin{floatrow}")
        || has(r"\ffigbox")
        || has(r"\ttabbox")
        || has(r"\capbeside")
        || has(r"\fcapside")
        || has(r"\floatbox")
        || has(r"\floatfoot")
        || has(r"\DeclareNewFloatType")
        || has(r"\restylefloat")
        || has(r"\newfloatcommand"))
        && !has_pkg("floatrow")
    {
        packages.push("floatrow");
    }
    // booktabs: \toprule, \midrule, \bottomrule, \cmidrule etc
    if (has(r"\toprule")
        || has(r"\midrule")
        || has(r"\bottomrule")
        || has(r"\addlinespace")
        || has(r"\cmidrule"))
        && !has_pkg("booktabs")
    {
        packages.push("booktabs");
    }
    // multirow: cells spanning multiple rows
    if has(r"\multirow") && !has_pkg("multirow") {
        packages.push("multirow");
    }
    // enumitem: customized lists
    if (has(r"\setlist") || has(r"\begin{itemize}[") || has(r"\begin{enumerate}["))
        && !has_pkg("enumitem")
    {
        packages.push("enumitem");
    }
    // listings/minted: source code
    if (has(r"\begin{lstlisting}") || has(r"\lstinline")) && !has_pkg("listings") {
        packages.push("listings");
    }
    if has(r"\begin{minted}") && !has_pkg("minted") {
        packages.push("minted");
    }
    // caption/subcaption: captions outside floats & sub‐floats
    if has(r"\captionof{") && !has_pkg("caption") {
        packages.push("caption");
    }
    if has(r"\begin{subfigure}") && !has_pkg("subcaption") {
        packages.push("subcaption");
    }
    // longtable: multipage tables (\begin{longtable})
    if has(r"\begin{longtable}") && !has_pkg("longtable") {
        packages.push("longtable");
    }
    // threeparttable: table notes environment
    if has(r"\begin{threeparttable}") && !has_pkg("threeparttable") {
        packages.push("threeparttable");
    }
    // fancyhdr: fancy headers/footers
    if (has(r"\pagestyle{fancy}") || has(r"\fancyhead") || has(r"\fancyhf") || has(r"\fancyfoot"))
        && !has_pkg("fancyhdr")
    {
        packages.push("fancyhdr");
    }
    // tikz/pgfplots: graphics & plots
    if (has(r"\begin{tikzpicture}") || has(r"\tikz")) && !has_pkg("tikz") {
        packages.push("tikz");
    }
    if has(r"\begin{axis}") && !has_pkg("pgfplots") {
        packages.push("pgfplots");
    }
    // subfiles: stand-alone sub-documents
    if has(r"\subfile{") && !has_pkg("subfiles") {
        packages.push("subfiles");
    }

    // Build the final string
    packages
        .iter()
        .map(|pkg| {
            if !pkg.contains(r"\usepackage") {
                [r"\usepackage{", pkg, "}"].concat()
            } else {
                pkg.to_string()
            }
        })
        .join("\n")
}

/// Extract undefined control sequence names from a LaTeX log
fn extract_undefined_commands(log: &str) -> Vec<String> {
    let mut commands = Vec::new();
    let lines: Vec<&str> = log.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        if line.contains("! Undefined control sequence.") {
            // The command is usually on the next line after "<recently read>"
            if let Some(next_line) = lines.get(i + 1)
                && let Some(cmd_start) = next_line.find('\\')
            {
                // Extract the command name (ends at space or end of line)
                let cmd_part = &next_line[cmd_start..];
                let cmd_end = cmd_part
                    .find(|c: char| c.is_whitespace())
                    .unwrap_or(cmd_part.len());
                let cmd = &cmd_part[..cmd_end];
                if !cmd.is_empty() && !commands.contains(&cmd.to_string()) {
                    commands.push(cmd.to_string());
                }
            }
        }
    }

    commands
}

/// Encode a node to PNG using `latex` binary
#[tracing::instrument(skip(latex))]
pub fn latex_to_image(latex: &str, path: &Path, class: Option<&str>) -> Result<()> {
    let format = Format::from_path(path);
    let (latex_tool, image_tool) = match format {
        Format::Pdf => ("xelatex", ""),
        Format::Png => ("latex", "dvipng"),
        Format::Svg => ("xelatex", "pdf2svg"), // dvisvgm is an alternative here but does not handle raster images (e.g. PNG) well
        _ => bail!("unhandled format {format}"),
    };

    tracing::trace!("Generating {format} from LaTeX using `{latex_tool}`");

    // Use a unique job name to be able to run `latex` in the current working directory
    // (because paths in \input and \includegraphics commands are relative to that)
    // whilst also being able to clean up temporary file afterwards
    let job = [
        "temp-",
        rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect::<String>()
            .as_str(),
    ]
    .concat();

    // The varwidth option tells the standalone class to size the PDF page to the natural width of
    // its contents (up to the specified maximum) instead of a fixed text width. Without it content
    // (particulary in wide) table can be cut off. However, this errors with landscape tables, so ther
    // we need to use preview.
    let width = if latex.contains(r"\begin{landscape}") {
        "preview"
    } else {
        "varwidth=16cm"
    };

    //...and then wrap in standalone \documentclass if a \documentclass is not specified
    let class = class.unwrap_or("standalone");
    let preamble = use_packages(latex);
    let latex = if !latex.contains("\\documentclass") {
        format!(
            r"
\documentclass[{width},border=8pt]{{{class}}}

{preamble}

\begin{{document}}

{latex}

\end{{document}}
",
        )
    } else {
        latex.to_string()
    };

    let input_file = format!("{job}.tex");
    write(&input_file, latex)?;

    let latex_status = Command::new(latex_tool)
        .args([
            "-interaction=batchmode",
            "-halt-on-error",
            if latex_tool == "xelatex" {
                if matches!(format, Format::Pdf) || image_tool == "pdf2svg" {
                    "-output-format=pdf"
                } else {
                    "-no-pdf"
                }
            } else if latex_tool == "latex" {
                "-output-format=dvi"
            } else {
                ""
            },
            "-jobname",
            &job,
            &input_file,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    let log_file = PathBuf::from(format!("{job}.log"));
    let log = if log_file.exists() {
        read_to_string(log_file)?
    } else {
        String::new()
    };

    if let Some(dir) = path.parent() {
        create_dir_all(dir)?;
    }

    let image_status = if image_tool.is_empty() {
        move_file(format!("{job}.pdf"), path)?;

        None
    } else {
        let mut image_command = Command::new(image_tool);

        let input = format!(
            "{job}.{}",
            if image_tool == "pdf2svg" {
                "pdf"
            } else if latex_tool == "xelatex" {
                "xdv"
            } else {
                "dvi"
            }
        );
        let output = path.to_string_lossy().to_string();

        let args = if image_tool == "dvisvgm" {
            // Using --no-fonts when generating SVGs was found
            // to improve layout of text
            vec!["--no-fonts", "-o", &output, &input]
        } else if image_tool == "dvipng" {
            vec!["-o", &output, &input]
        } else if image_tool == "pdf2svg" {
            vec![input.as_str(), &output.as_str()]
        } else {
            vec![]
        };

        let status = image_command
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;

        Some(status)
    };

    for path in glob::glob(&format!("{job}.*"))?.flatten() {
        remove_file(path)?;
    }

    if !latex_status.success() {
        let undefined_cmds = extract_undefined_commands(&log);
        if !undefined_cmds.is_empty() {
            let cmd_list = undefined_cmds.join(", ");
            bail!(
                "{latex_tool} failed: undefined control sequence(s): {cmd_list}\n\n\
                 These commands are not defined. You may need to:\n\
                 - Add a \\usepackage{{}} for the package that defines them\n\
                 - Define them in your document preamble"
            );
        } else {
            bail!("{latex_tool} failed:\n\n{log}");
        }
    }
    if !image_status.map(|status| status.success()).unwrap_or(true) {
        bail!("{image_tool} failed");
    }

    Ok(())
}

pub trait LatexCodec {
    /// Encode a Stencila Schema node to Latex
    fn to_latex(&self, context: &mut LatexEncodeContext);
}

pub struct LatexEncodeContext {
    /// The format to encode (Latex or Rnw)
    pub format: Format,

    /// Whether the root node should be encoded standalone
    pub standalone: bool,

    /// Encode the outputs, rather than the source, of executable nodes
    pub render: bool,

    /// Highlight the rendered outputs of executable nodes
    pub highlight: bool,

    /// Encode such that changes in the encoded document can be applied back to its source
    pub reproducible: bool,

    /// Whether the root node is "coarse grained" (i.e. decoded with the `--coarse` option).
    /// Used to determine whether newlines are needed between blocks.
    pub coarse: bool,

    /// The temporary directory where images are encoded to if necessary
    pub temp_dir: PathBuf,

    /// The encoded Latex content
    content: String,

    /// Whether paragraph wrapping is turned on
    paragraph_content: Option<String>,

    /// A stack of node types, ids and start positions
    node_stack: Vec<(NodeType, NodeId, usize)>,

    /// The path to the current node
    node_path: NodePath,

    /// Node to position mapping
    pub mapping: Mapping,

    /// Encoding losses
    pub losses: Losses,
}

impl LatexEncodeContext {
    pub fn new(
        format: Format,
        standalone: bool,
        render: bool,
        highlight: bool,
        reproducible: bool,
    ) -> Self {
        let temp_dir = temp_dir();

        let content = String::new();

        Self {
            format,
            standalone,
            render,
            highlight,
            reproducible,
            temp_dir,
            coarse: false,
            content,
            paragraph_content: None,
            node_stack: Vec::default(),
            node_path: NodePath::new(),
            mapping: Mapping::default(),
            losses: Losses::default(),
        }
    }

    /// Whether encoding to LaTeX for a final target format that is generated using Pandoc
    ///
    /// Used when encoding "coarse" documents to alter encoding to that which is handled
    /// by Pandoc.
    pub fn has_format_via_pandoc(&self) -> bool {
        matches!(self.format, Format::Docx | Format::Odt)
    }

    /// Get the current insertion position (i.e. the number of characters in the content)
    fn char_index(&self) -> usize {
        self.content.chars().count()
    }

    /// Enter a node
    ///
    /// Pushes the node id and start position onto the stack.
    pub fn enter_node(&mut self, node_type: NodeType, node_id: NodeId) -> &mut Self {
        self.node_stack
            .push((node_type, node_id, self.char_index()));
        self
    }

    /// Exit a node
    ///
    /// Pops the node id and start position off the stack and creates a
    /// new mapping entry with those and the current position as end position.
    pub fn exit_node(&mut self) -> &mut Self {
        if let Some((node_type, node_id, start)) = self.node_stack.pop() {
            let mut end = self.char_index();
            if self.content.ends_with("\n\n") {
                end -= 1;
            }
            self.mapping.add(start, end, node_type, node_id, None, None)
        }
        self
    }

    /// Exit the final node
    ///
    /// Should only be used by the top level `Article`. Does not exclude any double newline
    /// at the end from the range.
    pub fn exit_node_final(&mut self) -> &mut Self {
        if let Some((node_type, node_id, start)) = self.node_stack.pop() {
            let end = self.char_index();
            self.mapping.add(start, end, node_type, node_id, None, None)
        }
        self
    }

    /// Push a property represented by a string content onto the LaTex
    ///
    /// Creates a new mapping entry for the property.
    pub fn property_str(&mut self, prop: NodeProperty, value: &str) -> &mut Self {
        let start = self.char_index();

        self.str(value);

        if let Some((node_type, node_id, ..)) = self.node_stack.last() {
            let end = self.char_index();
            self.mapping
                .add(start, end, *node_type, node_id.clone(), Some(prop), None);
        }
        self
    }

    /// Push a property by calling a function to push content onto the LaTex
    ///
    /// Creates a new mapping entry for the property.
    pub fn property_fn<F>(&mut self, prop: NodeProperty, func: F) -> &mut Self
    where
        F: Fn(&mut Self),
    {
        let start = self.char_index();

        self.node_path.push_back(NodeSlot::from(prop));
        func(self);
        self.node_path.pop_back();

        if let Some((node_type, node_id, ..)) = self.node_stack.last() {
            let end = self.char_index();
            self.mapping
                .add(start, end, *node_type, node_id.clone(), Some(prop), None);
        }
        self
    }

    /// Push a character onto the LaTeX content
    pub fn char(&mut self, value: char) -> &mut Self {
        if let Some(paragraph_content) = self.paragraph_content.as_mut() {
            paragraph_content.push(value);
        } else {
            self.content.push(value);
        }
        self
    }

    /// Push a string onto the LaTeX content
    pub fn str(&mut self, value: &str) -> &mut Self {
        if let Some(paragraph_content) = self.paragraph_content.as_mut() {
            paragraph_content.push_str(value);
        } else {
            self.content.push_str(value);
        }
        self
    }

    /// Escape a string and push it onto the LaTeX content
    pub fn escaped_str(&mut self, value: &str) -> &mut Self {
        let escaped = escape_latex(value);
        self.str(&escaped)
    }

    /// Add a single space to the end of the content
    pub fn space(&mut self) -> &mut Self {
        self.char(' ')
    }

    /// Add a single newline to the end of the content
    pub fn newline(&mut self) -> &mut Self {
        self.char('\n')
    }

    /// Ensure that there is a blank line at the end of the content
    pub fn ensure_blankline(&mut self) -> &mut Self {
        if !self.content.is_empty() {
            if !self.content.ends_with("\n") {
                self.newline();
            }
            if !self.content.ends_with("\n\n") {
                self.newline();
            }
        }
        self
    }

    /// Begin a LaTeX environment
    pub fn environ_begin(&mut self, name: &str) -> &mut Self {
        self.str("\\begin{").str(name).char('}')
    }

    /// End a LaTeX environment
    pub fn environ_end(&mut self, name: &str) -> &mut Self {
        self.str("\\end{").str(name).char('}')
    }

    /// Begin a LaTeX command
    pub fn command_begin(&mut self, name: &str) -> &mut Self {
        self.char('\\').str(name).char('{')
    }

    /// End a LaTeX command
    pub fn command_end(&mut self) -> &mut Self {
        self.char('}')
    }

    /// Begin a LaTeX paragraph
    pub fn paragraph_begin(&mut self) -> &mut Self {
        self.paragraph_content = Some(String::new());
        self
    }

    /// End a LaTeX paragraph
    pub fn paragraph_end(&mut self) -> &mut Self {
        if let Some(paragraph_content) = self.paragraph_content.take() {
            let para = split_paragraph::split(&paragraph_content, 80, true).join("\n");
            self.content.push_str(&para);
        }
        self
    }

    /// Begin a link to the current node
    pub fn link_begin(&mut self, position: Option<NodePosition>) -> &mut Self {
        let r#type = self.node_stack.last().map(|(node_type, ..)| *node_type);

        let path = Some(self.node_path.clone());

        let node_url = NodeUrl {
            r#type,
            path,
            position,
            ..Default::default()
        };

        self.str("\\href{").str(&node_url.to_string()).str("}{")
    }

    /// End a link to the current node
    pub fn link_end(&mut self) -> &mut Self {
        self.char('}')
    }

    /// Create a link to the current with some content
    pub fn link_with(&mut self, position: Option<NodePosition>, content: &str) -> &mut Self {
        self.link_begin(position).str(content).link_end()
    }

    /// Trim whitespace from the end of the content in-place
    ///
    /// According to [this](https://users.rust-lang.org/t/trim-string-in-place/15809/18)
    /// this is the recommended way to trim in place.
    pub fn trim_end(&mut self) -> &mut Self {
        let trimmed = self.content.trim_end();
        self.content.truncate(trimmed.len());
        self
    }

    /// Add a single loss
    pub fn add_loss(&mut self, label: &str) -> &mut Self {
        self.losses.add(label);
        self
    }

    /// Append a vector of losses
    pub fn merge_losses(&mut self, losses: Losses) -> &mut Self {
        self.losses.merge(losses);
        self
    }
}

macro_rules! to_string {
    ($type:ty, $name:literal) => {
        impl LatexCodec for $type {
            fn to_latex(&self, context: &mut LatexEncodeContext) {
                context.str(&self.to_string());
            }
        }
    };
}

to_string!(bool, "Boolean");
to_string!(i64, "Integer");
to_string!(u64, "UnsignedInteger");
to_string!(f64, "Number");

impl LatexCodec for String {
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        context.str(&self.to_string());
    }
}

impl<T> LatexCodec for Box<T>
where
    T: LatexCodec,
{
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        self.as_ref().to_latex(context)
    }
}

impl<T> LatexCodec for Option<T>
where
    T: LatexCodec,
{
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        if let Some(value) = self {
            value.to_latex(context);
        }
    }
}

impl<T> LatexCodec for Vec<T>
where
    T: LatexCodec,
{
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        for (index, item) in self.iter().enumerate() {
            context.node_path.push_back(NodeSlot::from(index));
            item.to_latex(context);
            context.node_path.pop_back();
        }
    }
}
