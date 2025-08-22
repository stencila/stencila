use std::{
    fs::File,
    io::{BufWriter, Write, stdin, stdout},
    path::PathBuf,
    thread,
    time::{Duration, SystemTime},
};

use format::Format;
use rand::{Rng, rng, rngs::ThreadRng};

use cli_utils::{
    Code, ToStdout, clear_terminal,
    color_print::cstr,
    strip_ansi_escapes,
    tabulated::{Cell, Tabulated},
    terminal_size::terminal_size,
};
use codec_text::to_text;
use common::{
    clap::{self, Args, ValueEnum},
    eyre::{Context, Result, bail},
    itertools::Itertools,
    serde_json::json,
    tracing,
};
use schema::{Block, Inline, ListItem, MessageLevel, Node, Visitor, WalkControl, WalkthroughStep};
use tools::{Agg, Tool, ToolStdio};

use crate::Document;

impl Document {
    /// Demonstrate the document in the terminal
    pub async fn demo(&self, options: DemoOptions) -> Result<()> {
        let root = &*self.root.read().await;

        let mut walker = Walker::new(options)?;

        // Clear the terminal if we're starting from the first slide or no slides specified
        if walker.in_active_slide {
            clear_terminal();
        }

        walker.walk(root);
        walker.finish().await?;

        Ok(())
    }
}

// Default values for demo options
const SPEED_DEFAULT: f64 = 100.0;
const SPEED_VARIANCE_DEFAULT: f64 = 0.3;
const PUNCTUATION_PAUSE_DEFAULT: u64 = 200;
const TYPO_RATE_DEFAULT: f64 = 0.0;
const TYPO_PAUSE_DEFAULT: u64 = 500;
const HESITATION_RATE_DEFAULT: f64 = 0.0;
const HESITATION_DURATION_DEFAULT: u64 = 100;
const MIN_RUNNING_DEFAULT: u64 = 500;
const MAX_RUNNING_DEFAULT: u64 = 5000;

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
enum DemoPreset {
    /// Slower typing with some typos and hesitation
    Slow,
    /// Average WPM, typo and hesitation rate
    Natural,
    /// 200 WPM, no hesitation, no typos, consistent code running time
    Fast,
    /// Very high WPM and zero code running times
    Instant,
}

#[derive(Debug, Args)]
pub struct DemoOptions {
    /// The path of the recording to generate
    ///
    /// Supported output formats are GIF, MP4 and ASCIICast and will be
    /// determined from the file extension.
    output: Option<PathBuf>,

    /// Preset for demo style
    #[arg(long, value_enum, default_value_t = DemoPreset::Natural)]
    preset: DemoPreset,

    /// Typing speed in words per minute
    #[arg(long, default_value_t = SPEED_DEFAULT)]
    speed: f64,

    /// Variance in typing speed (0.0 to 1.0)
    #[arg(long, default_value_t = SPEED_VARIANCE_DEFAULT)]
    speed_variance: f64,

    /// How long to pause after punctuation (milliseconds)
    #[arg(long, default_value_t = PUNCTUATION_PAUSE_DEFAULT)]
    punctuation_pause: u64,

    /// Probability of making a typo (0.0 to 1.0)
    #[arg(long, default_value_t = TYPO_RATE_DEFAULT)]
    typo_rate: f64,

    /// How long to pause after typos before correcting (milliseconds)
    #[arg(long, default_value_t = TYPO_PAUSE_DEFAULT)]
    typo_pause: u64,

    /// Probability of brief hesitation (0.0 to 1.0)
    #[arg(long, default_value_t = HESITATION_RATE_DEFAULT)]
    hesitation_rate: f64,

    /// Hesitation duration in milliseconds
    #[arg(long, default_value_t = HESITATION_DURATION_DEFAULT)]
    hesitation_duration: u64,

    /// Do not apply syntax highlighting to code
    #[arg(long, alias = "no-highlight")]
    no_highlighting: bool,

    /// Minimum duration for running spinner in milliseconds
    ///
    /// The execution duration of executable nodes will be used for the
    /// spinner duration, but will be clamped to this minimum value.
    #[arg(long, default_value_t = MIN_RUNNING_DEFAULT)]
    min_running: u64,

    /// Maximum duration for running spinner in milliseconds
    ///
    /// The execution duration of executable nodes will be used for the
    /// spinner duration, but will be clamped to this maximum value.
    #[arg(long, default_value_t = MAX_RUNNING_DEFAULT)]
    max_running: u64,

    /// Arguments to pass through to `agg` when recoding to GIF
    ///
    /// See `agg --help`, or `stencila tools run agg --help`
    #[arg(last = true, allow_hyphen_values = true)]
    agg_args: Vec<String>,

    /// Specify which slides to demo
    ///
    /// Slides are delimited by thematic breaks (---). Examples:
    /// - "2" - only slide 2
    /// - "2-4" - slides 2 through 4
    /// - "2-" - slide 2 to the end
    /// - "-3" - slides 1 through 3
    /// - "1,3-5,7-" - slides 1, 3 through 5, and 7 to the end
    #[arg(long)]
    slides: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OutputFormat {
    Asciicast,
    Gif,
}

/// Represents which slides should be included in the demo
#[derive(Debug, Clone)]
struct SlideRanges {
    ranges: Vec<SlideRange>,
}

#[derive(Debug, Clone)]
enum SlideRange {
    Single(usize),
    Range(usize, usize),
    From(usize),
    To(usize),
}

impl SlideRanges {
    /// Parse a slide range string like "1,3-5,7-"
    fn parse(input: &str) -> Result<Self> {
        let mut ranges = Vec::new();

        for part in input.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            if let Some((start, end)) = part.split_once('-') {
                let start = start.trim();
                let end = end.trim();

                if start.is_empty() && end.is_empty() {
                    bail!("Invalid range: '-' without numbers");
                } else if start.is_empty() {
                    // "-N" format
                    let n = end
                        .parse::<usize>()
                        .wrap_err_with(|| format!("Invalid slide number: {end}"))?;
                    if n == 0 {
                        bail!("Slide numbers start from 1");
                    }
                    ranges.push(SlideRange::To(n));
                } else if end.is_empty() {
                    // "N-" format
                    let n = start
                        .parse::<usize>()
                        .wrap_err_with(|| format!("Invalid slide number: {start}"))?;
                    if n == 0 {
                        bail!("Slide numbers start from 1");
                    }
                    ranges.push(SlideRange::From(n));
                } else {
                    // "N-M" format
                    let start_num = start
                        .parse::<usize>()
                        .wrap_err_with(|| format!("Invalid slide number: {start}"))?;
                    let end_num = end
                        .parse::<usize>()
                        .wrap_err_with(|| format!("Invalid slide number: {end}"))?;
                    if start_num == 0 || end_num == 0 {
                        bail!("Slide numbers start from 1");
                    }
                    if start_num > end_num {
                        bail!("Invalid range: {} > {}", start_num, end_num);
                    }
                    ranges.push(SlideRange::Range(start_num, end_num));
                }
            } else {
                // Single slide number
                let n = part
                    .parse::<usize>()
                    .wrap_err_with(|| format!("Invalid slide number: {part}"))?;
                if n == 0 {
                    bail!("Slide numbers start from 1");
                }
                ranges.push(SlideRange::Single(n));
            }
        }

        if ranges.is_empty() {
            bail!("No valid slide ranges specified");
        }

        Ok(Self { ranges })
    }

    /// Check if a slide number is included in the ranges
    fn contains(&self, slide: usize) -> bool {
        self.ranges.iter().any(|range| match range {
            SlideRange::Single(n) => *n == slide,
            SlideRange::Range(start, end) => slide >= *start && slide <= *end,
            SlideRange::From(start) => slide >= *start,
            SlideRange::To(end) => slide <= *end,
        })
    }
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

    /// The terminal size
    terminal_size: (u16, u16),

    /// Parsed slide ranges to demo
    slide_ranges: Option<SlideRanges>,

    /// Current slide number (starts at 1)
    current_slide: usize,

    /// Whether we're currently in an active slide that should be shown
    in_active_slide: bool,
}

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const ITALIC: &str = "\x1b[3m";
const UNDERLINE: &str = "\x1b[4m";
const STRIKETHROUGH: &str = "\x1b[9m";

const FG_RED: &str = "\x1b[31m";
const FG_GREEN: &str = "\x1b[32m";
const FG_YELLOW: &str = "\x1b[33m";
const FG_BLUE: &str = "\x1b[34m";
const FG_MAGENTA: &str = "\x1b[35m";
const FG_CYAN: &str = "\x1b[36m";
const FG_ORANGE: &str = "\x1b[38;5;208m"; // Orange using 256-color palette

const SHOW_CURSOR: &str = "\x1b[?25h";
const HIDE_CURSOR: &str = "\x1b[?25l";

// Unicode box-drawing characters
const TOP_LEFT: &str = "╭";
const TOP_RIGHT: &str = "╮";
const BOTTOM_LEFT: &str = "╰";
const BOTTOM_RIGHT: &str = "╯";
const HORIZONTAL: &str = "─";
const VERTICAL: &str = "│";

impl Walker {
    /// Create a new walker
    fn new(mut options: DemoOptions) -> Result<Self> {
        // Apply preset defaults if specified
        match options.preset {
            DemoPreset::Slow => {
                // Slow typing with some typos and hesitation
                if options.speed == SPEED_DEFAULT {
                    options.speed = 80.0;
                }
                if options.speed_variance == SPEED_VARIANCE_DEFAULT {
                    options.speed_variance = 0.4;
                }
                if options.punctuation_pause == PUNCTUATION_PAUSE_DEFAULT {
                    options.punctuation_pause = 300;
                }
                if options.typo_rate == TYPO_RATE_DEFAULT {
                    options.typo_rate = 0.05;
                }
                if options.typo_pause == TYPO_PAUSE_DEFAULT {
                    options.typo_pause = 600;
                }
                if options.hesitation_rate == HESITATION_RATE_DEFAULT {
                    options.hesitation_rate = 0.1;
                }
                if options.hesitation_duration == HESITATION_DURATION_DEFAULT {
                    options.hesitation_duration = 200;
                }
                if options.min_running == MIN_RUNNING_DEFAULT {
                    options.min_running = 800;
                }
                if options.max_running == MAX_RUNNING_DEFAULT {
                    options.max_running = 6000;
                }
            }
            DemoPreset::Natural => {
                // Average typing speed with occasional typos and hesitation
                if options.speed == SPEED_DEFAULT {
                    options.speed = 100.0;
                }
                if options.speed_variance == SPEED_VARIANCE_DEFAULT {
                    options.speed_variance = 0.3;
                }
                if options.punctuation_pause == PUNCTUATION_PAUSE_DEFAULT {
                    options.punctuation_pause = 250;
                }
                if options.typo_rate == TYPO_RATE_DEFAULT {
                    options.typo_rate = 0.02;
                }
                if options.typo_pause == TYPO_PAUSE_DEFAULT {
                    options.typo_pause = 500;
                }
                if options.hesitation_rate == HESITATION_RATE_DEFAULT {
                    options.hesitation_rate = 0.05;
                }
                if options.hesitation_duration == HESITATION_DURATION_DEFAULT {
                    options.hesitation_duration = 150;
                }
                if options.min_running == MIN_RUNNING_DEFAULT {
                    options.min_running = 500;
                }
                if options.max_running == MAX_RUNNING_DEFAULT {
                    options.max_running = 5000;
                }
            }
            DemoPreset::Fast => {
                // Fast typing with no typos or hesitation
                if options.speed == SPEED_DEFAULT {
                    options.speed = 200.0;
                }
                if options.speed_variance == SPEED_VARIANCE_DEFAULT {
                    options.speed_variance = 0.0;
                }
                if options.punctuation_pause == PUNCTUATION_PAUSE_DEFAULT {
                    options.punctuation_pause = 200;
                }
                if options.typo_rate == TYPO_RATE_DEFAULT {
                    options.typo_rate = 0.0;
                }
                if options.typo_pause == TYPO_PAUSE_DEFAULT {
                    options.typo_pause = 0;
                }
                if options.hesitation_rate == HESITATION_RATE_DEFAULT {
                    options.hesitation_rate = 0.0;
                }
                if options.hesitation_duration == HESITATION_DURATION_DEFAULT {
                    options.hesitation_duration = 0;
                }
                if options.min_running == MIN_RUNNING_DEFAULT {
                    options.min_running = 1000;
                }
                if options.max_running == MAX_RUNNING_DEFAULT {
                    options.max_running = 1000;
                }
            }
            DemoPreset::Instant => {
                // Very fast typing with minimal running time
                if options.speed == SPEED_DEFAULT {
                    options.speed = f64::MAX;
                }
                if options.punctuation_pause == PUNCTUATION_PAUSE_DEFAULT {
                    options.punctuation_pause = 0;
                }
                if options.speed_variance == SPEED_VARIANCE_DEFAULT {
                    options.speed_variance = 0.0;
                }
                if options.typo_rate == TYPO_RATE_DEFAULT {
                    options.typo_rate = 0.0;
                }
                if options.typo_pause == TYPO_PAUSE_DEFAULT {
                    options.typo_pause = 0;
                }
                if options.hesitation_rate == HESITATION_RATE_DEFAULT {
                    options.hesitation_rate = 0.0;
                }
                if options.hesitation_duration == HESITATION_DURATION_DEFAULT {
                    options.hesitation_duration = 0;
                }
                if options.min_running == MIN_RUNNING_DEFAULT {
                    options.min_running = 0;
                }
                if options.max_running == MAX_RUNNING_DEFAULT {
                    options.max_running = 0;
                }
            }
        }

        // Clamp option values to ensure valid values and prevent random sampling panics
        options.speed = options.speed.max(1.0); // Minimum 1 WPM
        options.speed_variance = options.speed_variance.clamp(0.0, 1.0);
        options.typo_rate = options.typo_rate.clamp(0.0, 1.0);
        options.hesitation_rate = options.hesitation_rate.clamp(0.0, 1.0);
        options.max_running = options.max_running.max(options.min_running);

        let rng = rng();

        let start_time = SystemTime::now();

        let terminal_size = terminal_size()
            .map(|(width, height)| (width.0, height.0))
            .unwrap_or_else(|| {
                // Fall back to environment variables if terminal size detection fails
                let width = std::env::var("COLUMNS")
                    .ok()
                    .and_then(|s| s.parse::<u16>().ok())
                    .unwrap_or(80);
                let height = std::env::var("LINES")
                    .ok()
                    .and_then(|s| s.parse::<u16>().ok())
                    .unwrap_or(24);
                (width, height)
            });

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
                // Setting width is important because crates like comfy-table use that to determine
                // column widths.
                // See https://docs.asciinema.org/manual/asciicast/v2/
                let (width, height) = terminal_size;
                let header = common::serde_json::json!({
                    "version": 2,
                    "width": width,
                    "height": height
                });
                writeln!(&mut writer, "{header}").wrap_err("Failed to write asciicast header")?;

                (
                    Some(writer),
                    Some(asciicast_path),
                    Some(path.clone()),
                    format,
                )
            } else {
                (None, None, None, OutputFormat::Asciicast)
            };

        // Parse slide ranges if provided
        let slide_ranges = if let Some(ref slides_str) = options.slides {
            Some(SlideRanges::parse(slides_str)?)
        } else {
            None
        };

        // Check if we should start in an active slide
        let in_active_slide = slide_ranges
            .as_ref()
            .map(|ranges| ranges.contains(1))
            .unwrap_or(true);

        Ok(Self {
            options,
            rng,
            asciicast_file: output,
            asciicast_path,
            output_path,
            output_format,
            start_time,
            terminal_size,
            slide_ranges,
            current_slide: 1,
            in_active_slide,
        })
    }

    /// Finish the recording by flushing the output and if necessary and, if the
    /// output is GIF, convert using `agg` tool.
    async fn finish(&mut self) -> Result<()> {
        // Always ensure cursor is visible when finishing
        self.controls(&[SHOW_CURSOR, RESET]);

        if let Some(ref mut asciicast_file) = self.asciicast_file {
            asciicast_file
                .flush()
                .wrap_err("Failed to flush asciicast file")?;

            // Handle conversion to GIF
            if matches!(self.output_format, OutputFormat::Gif)
                && let (Some(asciicast_path), Some(final_output_path)) =
                    (&self.asciicast_path, &self.output_path)
            {
                let status = Agg
                    .async_command()
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

        Ok(())
    }

    /// Write text to stdout and record to asciicast if configured
    #[allow(clippy::print_stdout)]
    fn write(&mut self, text: &str) -> &mut Self {
        // Only write if we're in an active slide
        if !self.in_active_slide {
            return self;
        }

        // Print to stdout
        print!("{text}");
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
        // Skip spinner if not in active slide
        if !self.in_active_slide {
            return self;
        }

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

    /// Wrap content with a dim border and rounded unicode corners
    fn boxed(&mut self, content: &str) -> &mut Self {
        let width = self.terminal_size.0 as usize;
        let lines: Vec<&str> = content.lines().collect();
        let mut boxed = String::new();

        // Top border
        boxed.push_str(TOP_LEFT);
        boxed.push_str(&HORIZONTAL.repeat(width.saturating_sub(2)));
        boxed.push_str(TOP_RIGHT);
        boxed.push_str(RESET);
        boxed.push('\n');

        // Content lines with side borders
        for line in &lines {
            boxed.push_str(VERTICAL);
            boxed.push_str(RESET);
            boxed.push(' ');
            boxed.push_str(line);

            // Calculate visible length (excluding ANSI escape sequences)
            let visible_len = strip_ansi_escapes(line).chars().count() + 3; // +3 for left border, space, and right space

            // Pad to width
            if visible_len < width {
                boxed.push_str(&" ".repeat(width - visible_len));
            }
            boxed.push_str(RESET);
            boxed.push_str(VERTICAL);
            boxed.push('\n');
        }

        // Bottom border
        boxed.push_str(RESET);
        boxed.push_str(BOTTOM_LEFT);
        boxed.push_str(&HORIZONTAL.repeat(width.saturating_sub(2)));
        boxed.push_str(BOTTOM_RIGHT);
        boxed.push_str(RESET);

        self.write(&boxed)
    }

    /// Simulate manual typing
    fn typing(&mut self, text: &str) -> &mut Self {
        let DemoOptions {
            speed: wpm,
            speed_variance,
            typo_rate,
            typo_pause,
            punctuation_pause,
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
            let variance = if speed_variance > 0.0 {
                self.rng.random_range(-speed_variance..speed_variance)
            } else {
                0.0
            };
            let delay_ms = ((base_delay_ms as f64) * (1.0 + variance)) as u64;

            // Maybe make a typo
            if self.rng.random::<f64>() < typo_rate && ch.is_alphabetic() {
                // Type a wrong character
                let typo_ch = self.typo(ch).to_string();
                self.write(&typo_ch);

                // Pause to "notice" the mistake
                thread::sleep(Duration::from_millis(typo_pause));

                // Backspace and correct
                self.write(&format!("\x08 \x08{ch}"));
            } else {
                self.write(&ch.to_string());
            }

            stdout().flush().ok();

            // Pause after punctuation
            if matches!(ch, ',' | '.' | '!' | '?' | ';' | ':') && punctuation_pause > 0 {
                thread::sleep(Duration::from_millis(punctuation_pause));
            }

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

impl Drop for Walker {
    #[allow(clippy::print_stdout)]
    fn drop(&mut self) {
        // Always restore cursor visibility when Walker is dropped
        // This handles cleanup even when interrupted (e.g., Ctrl+C)
        // Note: We bypass the in_active_slide check here to ensure cleanup
        print!("{SHOW_CURSOR}{RESET}");
        stdout().flush().ok();
    }
}

impl Visitor for Walker {
    fn visit_node(&mut self, node: &schema::Node) -> WalkControl {
        // Just continue walk for root level nodes
        if matches!(node, Node::Article(..)) {
            return WalkControl::Continue;
        }

        // Node types that are commonly output from code chunks
        match node {
            Node::Datatable(node) => self.write(&node.to_terminal().to_string()),
            Node::String(string) => {
                let string = string.trim();

                // Unless the string looks like like it already has a box around
                // it (has >=10 horizontals on first and last lines) then box it
                let lines: Vec<&str> = string.lines().collect();
                let has_box = if lines.len() >= 2 {
                    // Count horizontal box characters in first and last lines
                    let first_horizontals = lines
                        .first()
                        .unwrap_or(&"")
                        .chars()
                        .filter(|&c| c == '─' || c == '━' || c == '-' || c == '═')
                        .count();
                    let last_horizontals = lines
                        .last()
                        .unwrap_or(&"")
                        .chars()
                        .filter(|&c| c == '─' || c == '━' || c == '-' || c == '═')
                        .count();

                    first_horizontals >= 10 && last_horizontals >= 10
                } else {
                    false
                };

                if has_box {
                    self.write(string)
                } else {
                    self.boxed(string)
                }
            }
            _ => {
                let node_type = node.node_type();
                if node_type.is_primitive() {
                    self.boxed(&Code::new_from(Format::Json, node).map_or_else(
                        |_| String::from("??"),
                        |code| {
                            code.to_terminal()
                                .to_string()
                                .trim_end_matches(&["\n", RESET].concat())
                                .to_string()
                        },
                    ))
                } else {
                    self.boxed(&["[", &node_type.to_string(), "]"].concat())
                }
            }
        };

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

            Inline::Link(link) => {
                if link.target == to_text(&link.content) {
                    self.control(FG_BLUE).typing(&link.target).control(RESET);
                } else {
                    self.write("[")
                        .walk(&link.content)
                        .write("](")
                        .control(FG_BLUE)
                        .typing(&link.target)
                        .control(RESET)
                        .write(")");
                }
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
                self.controls(&[BOLD, FG_ORANGE])
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

                self.typing("```")
                    .control(FG_CYAN)
                    .typing(&lang)
                    .control(RESET)
                    .newline()
                    .typing(&code);
                if !(code.ends_with("\n") || code.ends_with(&["\n", RESET].concat())) {
                    self.newline();
                }
                self.typing("```").newlines(2);

                return WalkControl::Break;
            }

            Block::CodeChunk(block) => {
                let is_echoed = block.is_echoed.unwrap_or(false);
                let is_hidden = block.is_hidden.unwrap_or(false);

                if is_echoed {
                    let lang = block.programming_language.clone().unwrap_or_default();

                    let code = if self.options.no_highlighting {
                        block.code.to_string()
                    } else {
                        Code::new(Format::from_name(&lang), &block.code)
                            .to_terminal()
                            .to_string()
                    };

                    self.typing("```")
                        .control(FG_CYAN)
                        .typing(&lang)
                        .control(RESET)
                        .typing(cstr!(" <bold,magenta>exec"));

                    if is_hidden {
                        self.typing(cstr!(" <bold,magenta>hide"));
                    }

                    self.newline().typing(&code);
                    if !(code.ends_with("\n") || code.ends_with(&["\n", RESET].concat())) {
                        self.newline();
                    }
                    self.typing("```").newlines(2);
                }

                if is_hidden {
                    return WalkControl::Break;
                }

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

                if clamped_duration > 0 {
                    self.spinner(clamped_duration, "");
                }

                // Show any messages
                let msg = |level: MessageLevel, message: &str| match level {
                    MessageLevel::Info => format!("{FG_GREEN}Info{RESET}: {message}"),
                    MessageLevel::Warning => format!("{FG_YELLOW}Warning{RESET}: {message}"),
                    MessageLevel::Error => format!("{FG_RED}Error{RESET}: {message}"),
                    MessageLevel::Exception => {
                        format!("{FG_MAGENTA}Exception{RESET}: {message}")
                    }
                    _ => String::new(),
                };
                for message in block
                    .options
                    .compilation_messages
                    .iter()
                    .flatten()
                    .filter(|msg| msg.level >= MessageLevel::Info)
                {
                    self.boxed(&msg(message.level, &message.message))
                        .newlines(2);
                }
                for message in block
                    .options
                    .execution_messages
                    .iter()
                    .flatten()
                    .filter(|msg| msg.level >= MessageLevel::Info)
                {
                    self.boxed(&msg(message.level, &message.message))
                        .newlines(2);
                }

                // Display outputs if not hidden with blank line between them
                if let (false, Some(outputs)) = (is_hidden, &block.outputs) {
                    for output in outputs {
                        self.walk(output).newlines(2);
                    }
                }

                return WalkControl::Break;
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

            Block::ThematicBreak(..) => {
                // Move to next slide
                self.current_slide += 1;

                // Update active slide status
                let was_active = self.in_active_slide;
                self.in_active_slide = self
                    .slide_ranges
                    .as_ref()
                    .map(|ranges| ranges.contains(self.current_slide))
                    .unwrap_or(true);

                // Only clear terminal if we're entering an active slide
                if self.in_active_slide && (!was_active || self.slide_ranges.is_none()) {
                    clear_terminal();
                }
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
        // Skip waiting for input if not in active slide
        if !self.in_active_slide {
            return WalkControl::Continue;
        }

        // Ensure cursor is showing to indicate that waiting
        self.control(SHOW_CURSOR);

        // Wait for input
        let mut input = String::new();
        stdin().read_line(&mut input).ok();

        // Add a marker for the asciicast
        self.marker("");

        WalkControl::Continue
    }
}
