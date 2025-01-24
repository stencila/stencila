//! Facilitates providing hints to users about how to help themselves

use std::io::IsTerminal;
use crate::ToStdout;

#[derive(Debug, Default)]
pub struct Hint {
    /// A message sent to all users
    ///
    /// Note: do not prefix the base with `hint:`
    base: String,

    /// Additions to the base hint that are intended for users who are using the CLI.
    ///
    ///  - "Remove the `--dry-run` flag to submit to Zenodo"
    //.  - "Retry with a file created with `stencila render --standalone`"
    cli_advice: String,

    /// Additions to the base hint for users who are using an editor through the LSP
    lsp_advice: String,
}

impl Hint {
    #[inline]
    #[must_use = "Users see nothing unless printed. Consider calling .to_stdout()"]
    pub fn new(hint: &str) -> Self {
        Hint {
            base: hint.to_string(),
            ..Default::default()
        }
    }

    #[inline]
    #[must_use = "Users see nothing unless printed. Consider calling .to_stdout()"]
    pub fn new_cli_only(hint: &str) -> Self {
        Hint {
            cli_advice: hint.to_string(),
            ..Default::default()
        }
    }

    #[inline]
    #[must_use = "Users see nothing unless printed. Consider calling .to_stdout()"]
    pub fn new_lsp_only(hint: &str) -> Self {
        Hint {
            lsp_advice: hint.to_string(),
            ..Default::default()
        }
    }
}

impl std::fmt::Display for Hint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let base = self.base.trim();
        let supplement = if std::io::stdout().is_terminal() {
            &self.cli_advice
        } else {
            &self.lsp_advice
        };

        if !base.is_empty() {
            writeln!(f, "hint: {}", self.base)?
        };

        if !supplement.is_empty() {
            writeln!(f, "hint: {}", supplement)?
        };

        Ok(())
    }
}

impl Into<Message> for Hint {
    fn into(self) -> Message {
        Message(format!("{}", self))
    }
}

impl ToStdout for Hint {
    fn to_terminal(&self) -> impl std::fmt::Display {
        let nl = if cfg!(windows) { "\r\n" } else { "\n" };
        let mut msg = String::with_capacity(128);

        if !self.base.is_empty() {
            msg.push_str("hint: ");
            msg.push_str(&self.base);
            msg.push_str(nl);
        }

        if !self.cli_advice.is_empty() {
            msg.push_str("hint: ");
            msg.push_str(&self.cli_advice);
            msg.push_str(nl);
        }

        msg
    }
}

#[macro_export]
macro_rules! hint {
    
    ($msg:expr, $($key:ident = $value:expr),* $(,)?) => {{
        let mut hint = $crate::Hint::new($msg);
        $(
            match stringify!($key) {
                "cli" => hint.cli_advice = $value.to_string(),
                "lsp" => hint.lsp_advice = $value.to_string(),
                _ => (), // Should n
            }
        )*
        hint
    }};
    
    ($str:literal, $($arg:tt)*) => {
        $crate::Hint::new(&format!($str, $($arg)*))
    };    
    
    ($msg:literal) => {
        $crate::Hint::new($msg)
    };
}

#[macro_export]
macro_rules! cli_hint {
    ($str:literal, $($arg:tt)*) => {
        $crate::Hint::new_cli_only(&format!($str, $($arg)*))
    };

    ($str:literal) => {
        $crate::Hint::new_cli_only($str)
    };
}

#[macro_export]
macro_rules! lsp_hint {
    ($str:literal, $($arg:tt)*) => {
        $crate::Hint::new_lsp_only(&format!($str, $($arg)*))
    };

    ($str:literal) => {
        $crate::Hint::new_lsp_only($str)
    };
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_hint() {
        let hint = hint!("basic message");
        assert_eq!(hint.base, "basic message");
        assert!(hint.cli_advice.is_empty());
        assert!(hint.lsp_advice.is_empty());
    }

    #[test]
    fn test_formatted_hint() {
        let value = 10_1000;
        let hint = hint!("formatted {}", value);
        assert_eq!(hint.base, "formatted 42");
    }

    #[test]
    fn test_hint_with_advice() {
        let hint = hint!("base message",
            cli = "cli advice",
            lsp = "lsp advice",
        );
        assert_eq!(hint.base, "base message");
        assert_eq!(hint.cli_advice, "cli advice");
        assert_eq!(hint.lsp_advice, "lsp advice");
    }

    #[test]
    fn test_cli_hint() {
        let hint = cli_hint!("cli only message");
        assert!(hint.base.is_empty());
        assert_eq!(hint.cli_advice, "cli only message");
        assert!(hint.lsp_advice.is_empty());
    }

    #[test]
    fn test_formatted_cli_hint() {
        let value = "test";
        let hint = cli_hint!("cli message: {}", value);
        assert!(hint.base.is_empty());
        assert_eq!(hint.cli_advice, "cli message: test");
    }

    #[test]
    fn test_lsp_hint() {
        let hint = lsp_hint!("lsp only message");
        assert!(hint.base.is_empty());
        assert!(hint.cli_advice.is_empty());
        assert_eq!(hint.lsp_advice, "lsp only message");
    }

    #[test]
    fn test_formatted_lsp_hint() {
        let value = 123;
        let hint = lsp_hint!("lsp message: {}", value);
        assert!(hint.base.is_empty());
        assert!(hint.cli_advice.is_empty());
        assert_eq!(hint.lsp_advice, "lsp message: 123");
    }

    #[test]
    #[should_panic(expected = "Invalid hint key")]
    fn test_invalid_hint_key() {
        hint!("message", invalid = "value".to_string());
    }

    #[test]
    fn test_display_formatting() {
        let hint = hint!("base",
            cli = "cli-specific info",
            lsp = "lsp-specific info"
        );
        let output = format!("{}", hint);
        assert!(output.contains("hint: base"));
        if std::io::stdout().is_terminal() {
            assert!(output.contains("cli-specific"));
            assert!(!output.contains("lsp-specific"));
        }
    }
}