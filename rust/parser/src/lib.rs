use std::hash::{Hash, Hasher};

use common::{once_cell::sync::Lazy, regex::Regex, seahash::SeaHasher};
use format::Format;
use schema::{CompilationDigest, ExecutionTag};

// Re-exports for the convenience of internal crates implementing
// the `Parser` trait
pub use common;
pub use format;
pub use schema;

/// Parse information
pub struct ParseInfo {
    /// The compilation digest of the code
    pub compilation_digest: CompilationDigest,

    /// Tags parsed from comments in the code
    pub execution_tags: Option<Vec<ExecutionTag>>,

    /// Whether, and how, the code has changed since last time it was parsed
    pub changed: ParseChange,
}

pub enum ParseChange {
    No,
    State,
    Semantics,
}

impl ParseChange {
    pub fn yes(&self) -> bool {
        !matches!(self, Self::No)
    }

    pub fn no(&self) -> bool {
        matches!(self, Self::No)
    }

    pub fn state(&self) -> bool {
        matches!(self, Self::State)
    }

    pub fn semantics(&self) -> bool {
        matches!(self, Self::Semantics)
    }
}

/// A parser of code in a programming language
pub trait Parser: Sync + Send {
    /// Get the name of the parser
    fn name(&self) -> String;

    /// Get the languages supported by the parser
    fn supports_languages(&self) -> Vec<Format> {
        Vec::new()
    }

    /// Does the parser support a particular language?
    fn supports_language(&self, format: &Format) -> bool {
        self.supports_languages().contains(format)
    }

    /// Calculate the state digest of the code and language
    fn state_digest(&self, code: &str, format: &Format) -> u64 {
        let mut hash = SeaHasher::new();
        code.hash(&mut hash);
        format.to_string().hash(&mut hash);
        hash.finish()
    }

    /// Extract execution tags from some code
    fn execution_tags(&self, code: &str) -> Option<Vec<ExecutionTag>> {
        static REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(
                r"@(pure|impure|assigns|changes|uses|imports|reads|writes|watches)\s*([^\n]*)",
            )
            .expect("Invalid regex")
        });

        let tags = REGEX
            .captures_iter(code)
            .fold(Vec::new(), |mut tags, captures| {
                for value in captures[2].split_whitespace() {
                    tags.push(ExecutionTag {
                        name: captures[1].to_string(),
                        value: value.to_string(),
                        ..Default::default()
                    });
                }
                tags
            });

        if !tags.is_empty() { Some(tags) } else { None }
    }

    /// Calculate a [`ParseChange`] from the current and previous [`CompilationDigest`] of some code
    fn change(
        &self,
        current: &CompilationDigest,
        previous: &Option<CompilationDigest>,
    ) -> ParseChange {
        let Some(previous) = previous else {
            return ParseChange::Semantics;
        };

        if current.semantic_digest != previous.semantic_digest {
            return ParseChange::Semantics;
        }

        if current.state_digest != previous.state_digest {
            return ParseChange::State;
        }

        ParseChange::No
    }

    /// Parse code in a language
    fn parse(
        &self,
        code: &str,
        format: &Format,
        previous_compilation_digest: &Option<CompilationDigest>,
    ) -> ParseInfo;
}

/// A default parser
///
/// Calculates language independent `ParseInfo` properties such
/// the state digest and execution tags.
#[derive(Default)]
pub struct DefaultParser;

impl Parser for DefaultParser {
    fn name(&self) -> String {
        "default".to_string()
    }

    fn parse(
        &self,
        code: &str,
        format: &Format,
        previous_compilation_digest: &Option<CompilationDigest>,
    ) -> ParseInfo {
        let state_digest = self.state_digest(code, format);
        let execution_tags = self.execution_tags(code);

        let compilation_digest = CompilationDigest {
            state_digest,
            ..Default::default()
        };

        let change = self.change(&compilation_digest, previous_compilation_digest);

        ParseInfo {
            compilation_digest,
            execution_tags,
            changed: change,
        }
    }
}
