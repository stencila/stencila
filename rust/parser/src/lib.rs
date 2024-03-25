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

        if !tags.is_empty() {
            Some(tags)
        } else {
            None
        }
    }

    /// Parse code in a language
    fn parse(&self, code: &str, format: &Format) -> ParseInfo;
}

/// A default parser
///
/// Calculates language independent `ParseInfo` properties such
/// the state digest and execution tags.
#[derive(Default)]
pub struct DefaultParser {}

impl Parser for DefaultParser {
    fn name(&self) -> String {
        "default".to_string()
    }

    fn parse(&self, code: &str, format: &Format) -> ParseInfo {
        ParseInfo {
            compilation_digest: CompilationDigest {
                state_digest: self.state_digest(code, format),
                ..Default::default()
            },
            execution_tags: self.execution_tags(code),
        }
    }
}
