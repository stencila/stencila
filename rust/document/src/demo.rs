use std::{
    fs::File,
    io::{stdin, stdout, BufWriter, Write},
    path::PathBuf,
    thread,
    time::{Duration, SystemTime},
};

use format::Format;
use rand::{rng, rngs::ThreadRng, Rng};

use cli_utils::{
    color_print::cstr,
    tabulated::{Cell, Tabulated},
    Code, ToStdout,
};
use codec_text::to_text;
use common::{
    clap::{self, Args},
    eyre::{bail, Context, Result},
    itertools::Itertools,
    serde_json::json,
    tracing,
};
use schema::{Block, Inline, ListItem, Node, Visitor, WalkControl, WalkthroughStep};
use tools::{AsyncToolCommand, ToolStdio};

use crate::Document;

impl Document {
    /// Demonstrate the document in the terminal
    pub(super) async fn demo(&self, options: DemoOptions) -> Result<()> {
        let root = &*self.root.read().await;

        let mut walker = Walker::new(options)?;
        walker.walk(root);
        walker.finish().await?;

        Ok(())
    }
}

#[derive(Debug, Args)]
pub(super) struct DemoOptions {
    /// The path of the recording to generate
    ///
    /// Supported output formats are GIF, MP4 and ASCIICast and will be
    /// determined from the file extension.
    output: Option<PathBuf>,

    /// Typing speed in words per minute
    #[arg(long, default_value = "100")]
    speed: f64,

    /// Variance in typing speed (0.0 to 1.0)
    #[arg(long, default_value = "0.3")]
    speed_variance: f64,

    /// Probability of making a typo (0.0 to 1.0)
    #[arg(long, default_value = "0")]
    typo_rate: f64,

    /// How long to pause after typos before correcting
    #[arg(long, default_value = "500")]
    typo_pause_ms: u64,

    /// Probability of brief hesitation (0.0 to 1.0)
    #[arg(long, default_value = "0")]
    hesitation_rate: f64,

    /// Hesitation duration in milliseconds
    #[arg(long, default_value = "100")]
    hesitation_duration: u64,

    /// Do not apply syntax highlighting to code
    #[arg(long, alias = "no-highlight")]
    no_highlighting: bool,

    /// Minimum duration for running spinner in milliseconds
    ///
    /// The execution duration of executable nodes will be used for the
    /// spinner duration, but will be clamped to this minimum value.
    #[arg(long, default_value = "500")]
    min_running: u64,

    /// Maximum duration for running spinner in milliseconds
    ///
    /// The execution duration of executable nodes will be used for the
    /// spinner duration, but will be clamped to this maximum value.
    #[arg(long, default_value = "5000")]
    max_running: u64,

    /// Arguments to pass through to `agg` when recoding to GIF
    ///
    /// See `agg --help`, or `stencila tools run agg --help`
    #[arg(last = true, allow_hyphen_values = true)]
    agg_args: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OutputFormat {
    Asciicast,
    Gif,
}

/// A visitor that walks over the document and demos it to the terminal
struct Walker {
    /// Options passed in from the CLI
    options: DemoOptions,

    // Random number generator for simulating typing
    rng: ThreadRng,

    /// The path to the asciicast output file
    asciicast_path: Option<PathBuf>,

    /// Buffered writer for the asciicast output file
    asciicast_file: Option<BufWriter<File>>,

    /// The final output path requested by the user
    output_path: Option<PathBuf>,

    /// The output format determined from extension
    output_format: OutputFormat,

    /// Start time for recording
    start_time: SystemTime,
}

impl Walker {
    /// Create a new walker
    fn new(options: DemoOptions) -> Result<Self> {
        let rng = rng();

        let start_time = SystemTime::now();

        let (output, asciicast_path, output_path, output_format) =
            if let Some(path) = &options.output {
                // Determine output format from extension
                let format = match path.extension().and_then(|ext| ext.to_str()) {
                    Some("gif") => OutputFormat::Gif,
                    Some("cast") => OutputFormat::Asciicast,
                    Some(ext) => bail!("Unsupported output format: {ext}"),
                    None => {
                        bail!("Output file must have an extension (.gif or .cast)")
                    }
                };

                // Create the AsciiCast file to write to (temporary for GIF)
                let (file, asciicast_path) = match format {
                    OutputFormat::Gif => {
                        // Create a temporary file for the asciicast
                        let temp_dir = std::env::temp_dir();
                        let temp_filename = format!(
                            "stencila-demo-{}.cast",
                            std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_millis()
                        );
                        let temp_path = temp_dir.join(temp_filename);
                        let file = File::create(&temp_path)
                            .wrap_err("Failed to create temporary asciicast file")?;
                        (file, temp_path)
                    }
                    OutputFormat::Asciicast => {
                        // Write directly to the output file
                        let file = File::create(path).wrap_err("Failed to create output file")?;
                        (file, path.clone())
                    }
                };

                let mut writer = BufWriter::new(file);

                // Write AsciiCast v2 header
                // See https://docs.asciinema.org/manual/asciicast/v2/
                let header = common::serde_json::json!({
                    "version": 2,
                    "width": 80,  // Default terminal width
                    "height": 24, // Default terminal height
                });
                writeln!(&mut writer, "{}", header).wrap_err("Failed to write asciicast header")?;

                (
                    Some(writer),
                    Some(asciicast_path),
                    Some(path.clone()),
                    format,
                )
            } else {
                (None, None, None, OutputFormat::Asciicast)
            };

        Ok(Self {
            options,
            rng,
            asciicast_file: output,
            asciicast_path,
            output_path,
            output_format,
            start_time,
        })
    }

    /// Finish the recording by flushing the output and if necessary and, if the
    /// output is GIF, convert using `agg` tool.
    async fn finish(&mut self) -> Result<()> {
        if let Some(ref mut asciicast_file) = self.asciicast_file {
            asciicast_file
                .flush()
                .wrap_err("Failed to flush asciicast file")?;

            // Handle conversion to GIF
            if matches!(self.output_format, OutputFormat::Gif) {
                if let (Some(asciicast_path), Some(final_output_path)) =
                    (&self.asciicast_path, &self.output_path)
                {
                    let status = AsyncToolCommand::new("agg")
                        .arg(asciicast_path)
                        .arg(final_output_path)
                        .stdout(ToolStdio::Inherit)
                        .stderr(ToolStdio::Inherit)
                        .status()
                        .await
                        .wrap_err("Failed to run agg tool")?;

                    if !status.success() {
                        bail!("agg conversion failed with exit code: {:?}", status.code());
                    }

                    // Clean up temporary asciicast file
                    if asciicast_path != final_output_path {
                        std::fs::remove_file(asciicast_path)
                            .wrap_err("Failed to remove temporary asciicast file")?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Write text to stdout and record to asciicast if configured
    #[allow(clippy::print_stdout)]
    fn write(&mut self, text: &str) -> &mut Self {
        // Print to stdout
        print!("{}", text);
        stdout().flush().ok();

        // Record to asciicast file if configured
        if let Some(asciicast_file) = &mut self.asciicast_file {
            let elapsed = self.start_time.elapsed().unwrap_or_default().as_secs_f64();

            // Ensure all newlines, \n, are made into returns+newlines, \r\n to
            // avoid indentation after
            let text = if text.contains("\r\n") {
                text.to_string()
            } else {
                text.replace('\n', "\r\n")
            };

            if let Err(error) = writeln!(asciicast_file, "{}", json!([elapsed, "o", text])) {
                tracing::error!("Failed to write to asciicast file: {error}",);
            }
        }

        self
    }

    /// Add a marker to the asciicast if configured
    /// https://docs.asciinema.org/manual/asciicast/v2/#m-marker
    fn marker(&mut self, label: &str) -> &mut Self {
        if let Some(asciicast_file) = &mut self.asciicast_file {
            let elapsed = self.start_time.elapsed().unwrap_or_default().as_secs_f64();

            if let Err(error) = writeln!(asciicast_file, "{}", json!([elapsed, "m", label])) {
                tracing::error!("Failed to write to asciicast file: {error}");
            }
        }

        self
    }

    /// Write a control sequence
    fn control(&mut self, sequence: &str) -> &mut Self {
        self.write(sequence)
    }

    /// Write control sequences
    fn controls(&mut self, sequences: &[&str]) -> &mut Self {
        self.write(&sequences.concat())
    }

    /// Write a new lines
    fn newline(&mut self) -> &mut Self {
        self.write("\n")
    }

    /// Write two or more new lines
    fn newlines(&mut self, num: usize) -> &mut Self {
        self.write(&"\n".repeat(num))
    }

    /// Display a spinner for a given duration in milliseconds
    fn spinner(&mut self, duration_ms: u64, message: &str) -> &mut Self {
        // Hide cursor and dim during spinner
        self.controls(&[HIDE_CURSOR, DIM]);

        // Spinner animation frames
        let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let mut frame_index = 0;

        let start_time = std::time::Instant::now();
        let duration = Duration::from_millis(duration_ms);

        // Animate spinner for the specified duration
        while start_time.elapsed() < duration {
            // Draw spinner frame
            self.write(&format!("\r{} {}", spinner_frames[frame_index], message));

            // Move to next frame
            frame_index = (frame_index + 1) % spinner_frames.len();

            // Small delay between frames
            thread::sleep(Duration::from_millis(80));
        }

        // Clear the spinner line
        let clear_line = "\r".to_string() + &" ".repeat(message.len() + 3) + "\r";
        self.write(&clear_line);

        // Reset and show cursor again
        self.controls(&[RESET, SHOW_CURSOR]);

        self
    }

    /// Simulate manual typing
    fn typing(&mut self, text: &str) -> &mut Self {
        let DemoOptions {
            speed: wpm,
            speed_variance,
            typo_rate,
            typo_pause_ms,
            hesitation_rate,
            hesitation_duration,
            ..
        } = self.options;

        // Convert WPM to delay between characters
        // Assuming average word length of 5 characters
        let base_delay_ms = (60_000.0 / (wpm * 5.0)) as u64;

        let mut in_escape_sequence = false;

        for ch in text.chars() {
            // Check if we're entering an escape sequence
            if ch == '\x1b' {
                in_escape_sequence = true;
                self.write(&ch.to_string());
                continue;
            }

            // Check if we're exiting an escape sequence
            if in_escape_sequence {
                self.write(&ch.to_string());
                if ch.is_alphabetic() || ch == 'm' {
                    in_escape_sequence = false;
                }
                continue;
            }

            // Skip delay for control characters
            if ch.is_control() {
                self.write(&ch.to_string());
                continue;
            }

            // Random variance in typing speed
            let variance = self.rng.random_range(-speed_variance..speed_variance);
            let delay_ms = ((base_delay_ms as f64) * (1.0 + variance)) as u64;

            // Maybe make a typo
            if self.rng.random::<f64>() < typo_rate && ch.is_alphabetic() {
                // Type a wrong character
                let typo_ch = self.typo(ch).to_string();
                self.write(&typo_ch);

                // Pause to "notice" the mistake
                thread::sleep(Duration::from_millis(typo_pause_ms));

                // Backspace and correct
                self.write(&format!("\x08 \x08{}", ch));
            } else {
                self.write(&ch.to_string());
            }

            stdout().flush().ok();

            // Maybe hesitate
            if self.rng.random::<f64>() < hesitation_rate {
                thread::sleep(Duration::from_millis(hesitation_duration));
            }

            // Normal typing delay
            thread::sleep(Duration::from_millis(delay_ms));
        }

        self
    }

    /// Generate a typo while typing
    fn typo(&mut self, intended: char) -> char {
        // Simple keyboard layout for generating realistic typos
        let keyboard_neighbors = match intended.to_ascii_lowercase() {
            'a' => "sqwz",
            'b' => "vghn",
            'c' => "xdfv",
            'd' => "sfre",
            'e' => "wrd",
            'f' => "dgrt",
            'g' => "fhty",
            'h' => "gjyu",
            'i' => "uko",
            'j' => "hkin",
            'k' => "jlim",
            'l' => "kop",
            'm' => "njk",
            'n' => "bmj",
            'o' => "ilp",
            'p' => "ol",
            'q' => "wa",
            'r' => "etf",
            's' => "awedz",
            't' => "rgy",
            'u' => "yhi",
            'v' => "cfgb",
            'w' => "qase",
            'x' => "zsd",
            'y' => "tgu",
            'z' => "xas",
            _ => "abcdefghijklmnopqrstuvwxyz",
        };

        let neighbors: Vec<char> = keyboard_neighbors.chars().collect();
        if neighbors.is_empty() {
            return intended;
        }

        let typo = neighbors[self.rng.random_range(0..neighbors.len())];

        // Preserve case
        if intended.is_uppercase() {
            typo.to_uppercase().next().unwrap_or(typo)
        } else {
            typo
        }
    }
}

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const ITALIC: &str = "\x1b[3m";
const UNDERLINE: &str = "\x1b[4m";
// const BLINK: &str = "\x1b[5m";
// const REVERSE: &str = "\x1b[7m";
const STRIKETHROUGH: &str = "\x1b[9m";

// const FG_BLACK: &str = "\x1b[30m";
const FG_RED: &str = "\x1b[31m";
// const FG_GREEN: &str = "\x1b[32m";
const FG_YELLOW: &str = "\x1b[33m";
// const FG_BLUE: &str = "\x1b[34m";
// const FG_MAGENTA: &str = "\x1b[35m";
const FG_CYAN: &str = "\x1b[36m";
// const FG_WHITE: &str = "\x1b[37m";
// const FG_DEFAULT: &str = "\x1b[39m";

const SHOW_CURSOR: &str = "\x1b[?25h";
const HIDE_CURSOR: &str = "\x1b[?25l";

impl Visitor for Walker {
    fn visit_node(&mut self, node: &schema::Node) -> WalkControl {
        if let Node::Datatable(node) = node {
            node.to_stdout()
        }

        WalkControl::Continue
    }

    fn visit_inline(&mut self, inline: &Inline) -> WalkControl {
        let (style, content) = match inline {
            Inline::Emphasis(inline) => (ITALIC, &inline.content),
            Inline::Strong(inline) => (BOLD, &inline.content),
            Inline::Strikeout(inline) => (STRIKETHROUGH, &inline.content),
            Inline::Underline(inline) => (UNDERLINE, &inline.content),

            Inline::QuoteInline(quote) => {
                self.typing("“").walk(&quote.content).typing("”");
                return WalkControl::Break;
            }

            Inline::CodeInline(code) => {
                self.typing(cstr!("<dim>`"))
                    .control(FG_RED)
                    .typing(&code.code)
                    .control(RESET)
                    .typing(cstr!("<dim>`"));
                return WalkControl::Break;
            }

            Inline::MathInline(math) => {
                self.control(FG_CYAN).typing(&math.code).control(RESET);
                return WalkControl::Break;
            }

            Inline::Text(text) => {
                self.typing(&text.value);
                return WalkControl::Break;
            }

            _ => {
                return WalkControl::Continue;
            }
        };

        self.control(style).walk(content).control(RESET);
        WalkControl::Break
    }

    fn visit_block(&mut self, block: &Block) -> WalkControl {
        match block {
            Block::Heading(block) => {
                if block.level == 1 {
                    self.marker(&to_text(&block.content));
                }
                self.controls(&[BOLD, FG_YELLOW])
                    .typing(&"#".repeat(block.level as usize))
                    .typing(" ")
                    .walk(&block.content)
                    .control(RESET)
                    .newlines(2);
                return WalkControl::Break;
            }

            Block::Paragraph(block) => {
                self.walk(&block.content).newlines(2);
                return WalkControl::Break;
            }

            Block::QuoteBlock(block) => {
                self.typing("“")
                    .control(ITALIC)
                    .walk(&block.content)
                    .control(RESET)
                    .newlines(2);
                return WalkControl::Break;
            }

            Block::CodeBlock(block) => {
                let lang = block.programming_language.clone().unwrap_or_default();

                let code = if self.options.no_highlighting {
                    block.code.to_string()
                } else {
                    Code::new(Format::from_name(&lang), &block.code)
                        .to_terminal()
                        .to_string()
                };

                self.typing(cstr!("<dim>```"))
                    .control(FG_CYAN)
                    .typing(&lang)
                    .control(RESET)
                    .newline()
                    .typing(&code);
                if !(code.ends_with("\n") || code.ends_with(&["\n", RESET].concat())) {
                    self.newline();
                }
                self.typing(cstr!("<dim>```")).newlines(2);

                return WalkControl::Break;
            }

            Block::CodeChunk(block) => {
                let lang = block.programming_language.clone().unwrap_or_default();

                let code = if self.options.no_highlighting {
                    block.code.to_string()
                } else {
                    Code::new(Format::from_name(&lang), &block.code)
                        .to_terminal()
                        .to_string()
                };

                self.typing(cstr!("<dim>```"))
                    .control(FG_CYAN)
                    .typing(&lang)
                    .control(RESET)
                    .typing(cstr!(" <bold,magenta>exec\n"))
                    .typing(&code);
                if !(code.ends_with("\n") || code.ends_with(&["\n", RESET].concat())) {
                    self.newline();
                }
                self.typing(cstr!("<dim>```")).newlines(2);

                let duration = block
                    .options
                    .execution_duration
                    .as_ref()
                    .map(|duration| duration.to_milliseconds())
                    .unwrap_or_default() as u64;

                // Clamp duration between min and max running time
                let clamped_duration = duration
                    .max(self.options.min_running)
                    .min(self.options.max_running);

                self.spinner(clamped_duration, "Running");

                // Continue walk over outputs
                return WalkControl::Continue;
            }

            Block::Table(table) => {
                let mut display = Tabulated::new();

                // TODO: add header row

                for row in &table.rows {
                    let cells = row
                        .cells
                        .iter()
                        .map(|cell| Cell::new(to_text(&cell.content)))
                        .collect_vec();
                    display.add_row(cells);
                }

                let table = display.to_string();

                self.write(&table).newlines(2);
                return WalkControl::Break;
            }

            _ => {}
        }

        WalkControl::Continue
    }

    fn visit_list_item(&mut self, _list_item: &ListItem) -> WalkControl {
        self.write("• ");
        WalkControl::Continue
    }

    fn visit_walkthrough_step(&mut self, _step: &WalkthroughStep) -> WalkControl {
        // Show cursor to indicate that waiting
        self.control(SHOW_CURSOR);

        // Wait for input
        let mut input = String::new();
        stdin().read_line(&mut input).ok();

        // Add a marker for the asciicast
        self.marker("");

        // Hide cursor again
        self.control(HIDE_CURSOR);

        WalkControl::Continue
    }
}
