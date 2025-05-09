//! Provides the `LatexCodec` trait for generating Latex for Stencila Schema nodes

use std::{
    env::temp_dir,
    fs::{create_dir_all, read_to_string, remove_file, write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use rand::{distr::Alphanumeric, rng, Rng};

use codec_info::{EncodeInfo, Losses, Mapping, NodeId, NodeProperty, NodeType};
use common::{
    eyre::{bail, Result},
    glob,
    itertools::Itertools,
    tracing,
};
use format::Format;

pub use codec_latex_derive::LatexCodec;

/// Encode a node that implements `LatexCodec` to Latex
///
/// A convenience function to save the caller from having to create a context etc.
pub fn to_latex<T>(
    node: &T,
    format: Format,
    standalone: bool,
    render: bool,
    highlight: bool,
) -> (String, EncodeInfo)
where
    T: LatexCodec,
{
    let mut context = LatexEncodeContext::new(format, standalone, render, highlight);
    node.to_latex(&mut context);

    let mut latex = context.content;

    // If the generated LaTeX does not have a preamble (might be one in a RawBlock),
    // then use necessary pages and wrap
    if standalone {
        if !latex.contains("\\end{document}") {
            latex.push_str("\\end{document}");
        }
        if !latex.contains("\\begin{document}") {
            latex.insert_str(0, "\\begin{document}\n\n");
        }
        if !latex.contains("\\documentclass") {
            let preamble = [
                "\\documentclass{article}\n\n",
                &use_packages(&latex),
                "\n\n",
            ]
            .concat();
            latex.insert_str(0, &preamble);
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

    // helper to check for either \usepackage or \RequirePackage (with or without options)
    let has_pkg = |pkg: &str| {
        latex.contains(&format!(r"\usepackage{{{}}}", pkg))
            || latex.contains(&format!(r"\RequirePackage{{{}}}", pkg))
            || latex.contains(&format!(r"\usepackage[")) && latex.contains(pkg)
    };

    // hyperref: links & urls
    if (latex.contains(r"\href") || latex.contains(r"\url")) && !has_pkg("hyperref") {
        packages.push("hyperref");
    }
    // graphicx: images
    if latex.contains(r"\includegraphics") && !has_pkg("graphicx") {
        packages.push("graphicx");
    }
    // amsmath: display‐math environments
    if (latex.contains(r"\begin{align}")
        || latex.contains(r"\begin{equation}")
        || latex.contains(r"\[")
        || latex.contains(r"\]"))
        && !has_pkg("amsmath")
    {
        packages.push("amsmath");
    }
    // amssymb: extra math symbols (\mathbb, \mathcal, etc.)
    if (latex.contains(r"\mathbb") || latex.contains(r"\mathcal") || latex.contains(r"\mathfrak"))
        && !has_pkg("amssymb")
    {
        packages.push("amssymb");
    }
    // xcolor: color support
    if (latex.contains(r"\color")
        || latex.contains(r"\textcolor")
        || latex.contains(r"\definecolor"))
        && !has_pkg("xcolor")
    {
        packages.push("xcolor");
    }
    // soul: text highlighting (\hl, \sethlcolor)
    if (latex.contains(r"\hl") || latex.contains(r"\sethlcolor")) && !has_pkg("soul") {
        packages.push("soul");
    }
    // colortbl: colored table rules (\arrayrulecolor)
    if latex.contains(r"\arrayrulecolor") && !has_pkg("colortbl") {
        packages.push("colortbl");
    }
    // geometry: page geometry
    if (latex.contains(r"\newgeometry") || latex.contains(r"\geometry{")) && !has_pkg("geometry") {
        packages.push("geometry");
    }
    // pdflscape: landscape pages
    if latex.contains(r"\begin{landscape}") && !has_pkg("pdflscape") {
        packages.push("pdflscape");
    }
    // placeins: \FloatBarrier
    if latex.contains(r"\FloatBarrier") && !has_pkg("placeins") {
        packages.push("placeins");
    }
    // booktabs: \toprule, \midrule, \bottomrule etc
    if (latex.contains(r"\toprule")
        || latex.contains(r"\midrule")
        || latex.contains(r"\bottomrule")
        || latex.contains(r"\addlinespace"))
        && !has_pkg("booktabs")
    {
        packages.push("booktabs");
    }
    // enumitem: customized lists
    if (latex.contains(r"\setlist")
        || latex.contains(r"\begin{itemize}[")
        || latex.contains(r"\begin{enumerate}["))
        && !has_pkg("enumitem")
    {
        packages.push("enumitem");
    }
    // listings/minted: source code
    if (latex.contains(r"\begin{lstlisting}") || latex.contains(r"\lstinline"))
        && !has_pkg("listings")
    {
        packages.push("listings");
    }
    if latex.contains(r"\begin{minted}") && !has_pkg("minted") {
        packages.push("minted");
    }
    // caption/subcaption: captions outside floats & sub‐floats
    if latex.contains(r"\captionof{") && !has_pkg("caption") {
        packages.push("caption");
    }
    if latex.contains(r"\begin{subfigure}") && !has_pkg("subcaption") {
        packages.push("subcaption");
    }
    // longtable: multipage tables (\begin{longtable})
    if latex.contains(r"\begin{longtable}") && !has_pkg("longtable") {
        packages.push("longtable");
    }
    // threeparttable: table notes environment
    if latex.contains(r"\begin{threeparttable}") && !has_pkg("threeparttable") {
        packages.push("threeparttable");
    }
    // fancyhdr: fancy headers/footers
    if (latex.contains(r"\pagestyle{fancy}")
        || latex.contains(r"\fancyhead")
        || latex.contains(r"\fancyhf")
        || latex.contains(r"\fancyfoot"))
        && !has_pkg("fancyhdr")
    {
        packages.push("fancyhdr");
    }
    // tikz/pgfplots: graphics & plots
    if (latex.contains(r"\begin{tikzpicture}") || latex.contains(r"\tikz")) && !has_pkg("tikz") {
        packages.push("tikz");
    }
    if latex.contains(r"\begin{axis}") && !has_pkg("pgfplots") {
        packages.push("pgfplots");
    }
    // subfiles: stand-alone sub-documents
    if latex.contains(r"\subfile{") && !has_pkg("subfiles") {
        packages.push("subfiles");
    }

    // Build the final string
    packages
        .iter()
        .map(|pkg| [r"\usepackage{", pkg, "}"].concat())
        .join("\n")
}

/// Encode a node to PNG using `latex` binary
#[tracing::instrument(skip(latex))]
pub fn latex_to_png(latex: &str, path: &Path) -> Result<()> {
    tracing::trace!("Generating PNG from LaTeX");

    // Use a unique job name to be able to run `latex` in the current working directory
    // (because paths in \input and \includegraphics commands are relative to that)
    // whilst also being able to clean up temporary file afterwards
    let job: String = rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    //...and then wrap in standalone \documentclass if a \documentclass is not specified
    let latex = if !latex.contains("\\documentclass") {
        [
            r"
\documentclass[border=5pt,preview]{standalone}

",
            &use_packages(latex),
            r"

\begin{document}

",
            latex,
            r"
\end{document}
",
        ]
        .concat()
    } else {
        latex.to_string()
    };

    let input_file = format!("{job}.tex");
    write(&input_file, latex)?;

    let latex_status = Command::new("latex")
        .args([
            "-interaction=batchmode",
            "-halt-on-error",
            "-output-format=dvi",
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
    let dvi_status = Command::new("dvipng")
        .args([
            "-D300",
            "-o",
            &path.to_string_lossy(),
            &format!("{job}.dvi"),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    for path in glob::glob(&format!("{job}.*"))?.flatten() {
        remove_file(path)?;
    }

    if !latex_status.success() {
        bail!("latex failed:\n\n{}", log);
    }
    if !dvi_status.success() {
        bail!("dvipng failed");
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

    /// Whether the root node is "coarse grained" (i.e. decoded with the `--coarse` option).
    /// Used to determine whether newlines are needed between blocks.
    pub coarse: bool,

    /// The encoded Latex content
    pub content: String,

    /// The temporary directory where images are encoded to if necessary
    pub temp_dir: PathBuf,

    /// A stack of node types, ids and start positions
    node_stack: Vec<(NodeType, NodeId, usize)>,

    /// Node to position mapping
    pub mapping: Mapping,

    /// Encoding losses
    pub losses: Losses,

    /// The nesting depth for any node type using fenced divs
    depth: usize,
}

impl LatexEncodeContext {
    pub fn new(format: Format, standalone: bool, render: bool, highlight: bool) -> Self {
        let temp_dir = temp_dir();

        Self {
            format,
            standalone,
            render,
            highlight,
            temp_dir,
            coarse: false,
            content: String::default(),
            node_stack: Vec::default(),
            mapping: Mapping::default(),
            losses: Losses::default(),
            depth: 0,
        }
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

        self.content.push_str(value);

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

        func(self);

        if let Some((node_type, node_id, ..)) = self.node_stack.last() {
            let end = self.char_index();
            self.mapping
                .add(start, end, *node_type, node_id.clone(), Some(prop), None);
        }
        self
    }

    /// Increase the nesting depth
    pub fn increase_depth(&mut self) -> &mut Self {
        self.depth += 1;
        self
    }

    /// Decrease the nesting depth
    pub fn decrease_depth(&mut self) -> &mut Self {
        self.depth = self.depth.saturating_sub(1);
        self
    }

    /// Push a string onto the Latex content
    pub fn str(&mut self, value: &str) -> &mut Self {
        if self.depth > 0 && matches!(self.content.chars().last(), None | Some('\n')) {
            self.content.push_str(&"    ".repeat(self.depth));
        }
        self.content.push_str(value);
        self
    }

    /// Push a character onto the Latex content
    pub fn char(&mut self, value: char) -> &mut Self {
        self.content.push(value);
        self
    }

    /// Add a single space to the end of the content
    pub fn space(&mut self) -> &mut Self {
        self.content.push(' ');
        self
    }

    /// Add a single newline to the end of the content
    pub fn newline(&mut self) -> &mut Self {
        self.content.push('\n');
        self
    }

    /// Enter a LaTeX environment
    pub fn environ_begin(&mut self, name: &str) -> &mut Self {
        self.str("\\begin{");
        self.content.push_str(name);
        self.content.push('}');
        self
    }

    /// Exit a LaTeX environment
    pub fn environ_end(&mut self, name: &str) -> &mut Self {
        self.str("\\end{");
        self.content.push_str(name);
        self.content.push_str("}\n");
        self
    }

    /// Enter a LaTeX command
    pub fn command_enter(&mut self, name: &str) -> &mut Self {
        self.content.push('\\');
        self.content.push_str(name);
        self.content.push('{');
        self
    }

    /// Exit a LaTeX command
    pub fn command_exit(&mut self) -> &mut Self {
        self.content.push('}');
        self
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
        for item in self.iter() {
            item.to_latex(context);
        }
    }
}
