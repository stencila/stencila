//! Parameter spread functionality for multi-variant execution.
//!
//! This module provides support for executing documents multiple times with
//! different parameter sets, producing multiple outputs from a single CLI invocation.
//!
//! # Overview
//!
//! When rendering parameterized documents, you often need to generate multiple
//! variants with different parameter combinations. This module supports three
//! spread modes:
//!
//! - **Grid**: Cartesian product of all multi-valued parameters
//! - **Zip**: Positional pairing of multi-valued parameters (all must have same length)
//! - **Cases**: Explicit parameter sets defined via `--case` arguments
//!
//!
//! # Examples
//!
//! ## Grid Mode (Cartesian Product)
//!
//! ```
//! use stencila_spread::{SpreadConfig, SpreadMode};
//!
//! let config = SpreadConfig::from_arguments(
//!     SpreadMode::Grid,
//!     &[("region", "north,south"), ("year", "2024,2025")],
//!     &[],
//!     100,
//! )?;
//!
//! let runs = config.generate_runs()?;
//! assert_eq!(runs.len(), 4); // 2 regions × 2 years
//! # Ok::<(), stencila_spread::SpreadError>(())
//! ```
//!
//! ## Zip Mode (Positional Pairing)
//!
//! ```
//! use stencila_spread::{SpreadConfig, SpreadMode};
//!
//! let config = SpreadConfig::from_arguments(
//!     SpreadMode::Zip,
//!     &[("region", "north,south"), ("code", "N,S")],
//!     &[],
//!     100,
//! )?;
//!
//! let runs = config.generate_runs()?;
//! assert_eq!(runs.len(), 2); // (north, N) and (south, S)
//! # Ok::<(), stencila_spread::SpreadError>(())
//! ```
//!
//! ## Cases Mode (Explicit Sets)
//!
//! ```
//! use stencila_spread::{SpreadConfig, SpreadMode};
//!
//! let config = SpreadConfig::from_arguments(
//!     SpreadMode::Cases,
//!     &[("currency", "USD")], // Global parameter
//!     &[
//!         "region=north species=ABC".to_string(),
//!         "region=south species=DEF".to_string(),
//!     ],
//!     100,
//! )?;
//!
//! let runs = config.generate_runs()?;
//! assert_eq!(runs.len(), 2);
//! # Ok::<(), stencila_spread::SpreadError>(())
//! ```

mod generation;
mod parsing;
mod template;
mod warnings;

use std::path::Path;

use clap::ValueEnum;
use strum::Display;
use thiserror::Error;

/// Errors that can occur during spread operations.
#[derive(Debug, Error)]
pub enum SpreadError {
    /// Zip mode requires all multi-valued parameters to have equal length.
    #[error("`--spread=zip` requires all multi-valued parameters to have equal length ({details})")]
    ZipLengthMismatch {
        /// Details about the parameter lengths.
        details: String,
    },

    /// Cases mode requires at least one `--case` parameter set.
    #[error("`--spread=cases` requires at least one --case parameter set")]
    NoCases,

    /// Unknown placeholder in output template.
    #[error("Unknown placeholder '{{{name}}}' in output template. Available: {{i}}, {available}")]
    UnknownPlaceholder {
        /// The unknown placeholder name.
        name: String,
        /// Comma-separated list of available placeholders.
        available: String,
    },

    /// Run count exceeds the configured limit.
    #[error(
        "Spread would generate {count} runs, exceeding limit of {max}. Use `--spread-max=N` to override or reduce parameters."
    )]
    TooManyRuns {
        /// The number of runs that would be generated.
        count: usize,
        /// The maximum allowed number of runs.
        max: usize,
    },

    /// Invalid `--case` syntax.
    #[error("Invalid `--case` syntax: {message}")]
    InvalidCaseSyntax {
        /// Description of the syntax error.
        message: String,
    },

    /// Duplicate parameter name in `--case`.
    #[error("Parameter `{name}` specified multiple times in `--case`")]
    DuplicateCaseParameter {
        /// The duplicated parameter name.
        name: String,
    },
}

/// Result type for spread operations.
pub type Result<T> = std::result::Result<T, SpreadError>;

// Re-export main types
pub use generation::{Run, count_runs, generate_runs_cases, generate_runs_grid, generate_runs_zip};
pub use parsing::{CaseParameters, ParameterValues, Parameters, parse_case};
pub use template::{
    apply_template, auto_append_placeholders, auto_append_placeholders_for_spread,
    has_placeholders, infer_spread_mode,
};
pub use warnings::{SpreadWarning, check_warnings};

/// The mode of parameter spreading.
#[derive(Debug, Display, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
#[strum(serialize_all = "lowercase")]
pub enum SpreadMode {
    /// Cartesian product of multi-valued parameters (default).
    ///
    /// For example, with `region=north,south` and `year=2024,2025`,
    /// generates 4 runs: all combinations of region and year.
    #[default]
    Grid,

    /// Positional pairing of multi-valued parameters.
    ///
    /// All multi-valued parameters must have the same length.
    /// For example, with `region=north,south` and `code=N,S`,
    /// generates 2 runs: (north, N) and (south, S).
    Zip,

    /// Explicitly enumerated parameter sets via `--case`.
    ///
    /// Each `--case` argument defines one run with specific parameter values.
    /// Global parameters are merged with each case.
    Cases,
}

/// Configuration for spread execution.
#[derive(Debug, Clone)]
pub struct SpreadConfig {
    /// The spread mode.
    pub mode: SpreadMode,
    /// Global parameters.
    pub params: Parameters,
    /// Explicit cases (for cases mode).
    pub cases: Vec<CaseParameters>,
    /// Maximum number of runs allowed.
    pub max_runs: usize,
}

impl Default for SpreadConfig {
    fn default() -> Self {
        SpreadConfig {
            mode: SpreadMode::Grid,
            params: Parameters::new(),
            cases: Vec::new(),
            max_runs: 100,
        }
    }
}

impl SpreadConfig {
    /// Build a spread configuration from CLI-style arguments.
    ///
    /// This parses positional arguments into parameters, with comma expansion
    /// based on the spread mode.
    ///
    /// # Arguments
    ///
    /// * `mode` - The spread mode to use
    /// * `arguments` - Parameter name-value pairs
    /// * `case_strings` - Raw `--case` argument strings (for cases mode)
    /// * `max_runs` - Maximum number of runs to allow
    ///
    /// # Errors
    ///
    /// Returns an error if case strings cannot be parsed.
    pub fn from_arguments(
        mode: SpreadMode,
        arguments: &[(&str, &str)],
        case_strings: &[String],
        max_runs: usize,
    ) -> Result<Self> {
        let mut params = Parameters::new();
        let mut cases: Vec<CaseParameters> = Vec::new();

        // Parse global params from arguments
        for (name, value) in arguments {
            if mode == SpreadMode::Cases {
                // In cases mode, don't expand commas in global params
                params.insert(name.to_string(), ParameterValues::scalar(value.to_string()));
            } else {
                // In grid/zip mode, split on commas
                params.insert(name.to_string(), ParameterValues::parse(value));
            }
        }

        // Parse cases from --case argument strings
        for case_str in case_strings {
            let case_params = parse_case(case_str)?;
            cases.push(case_params);
        }

        Ok(SpreadConfig {
            mode,
            params,
            cases,
            max_runs,
        })
    }

    /// Validate the configuration and return the number of runs.
    ///
    /// # Errors
    ///
    /// - `SpreadError::NoCases`: Cases mode with no cases
    /// - `SpreadError::TooManyRuns`: Run count exceeds max_runs limit
    pub fn validate(&self) -> Result<usize> {
        let run_count = count_runs(self.mode, &self.params, &self.cases);

        if run_count > self.max_runs {
            return Err(SpreadError::TooManyRuns {
                count: run_count,
                max: self.max_runs,
            });
        }

        // Additional validation for cases mode
        if self.mode == SpreadMode::Cases && self.cases.is_empty() {
            return Err(SpreadError::NoCases);
        }

        Ok(run_count)
    }

    /// Generate all runs according to the configuration.
    ///
    /// # Errors
    ///
    /// - `SpreadError::ZipLengthMismatch`: Zip mode with mismatched parameter lengths
    /// - `SpreadError::NoCases`: Cases mode with no cases
    pub fn generate_runs(&self) -> Result<Vec<Run>> {
        match self.mode {
            SpreadMode::Grid => Ok(generate_runs_grid(&self.params)),
            SpreadMode::Zip => generate_runs_zip(&self.params),
            SpreadMode::Cases => generate_runs_cases(&self.params, &self.cases),
        }
    }

    /// Get names of parameters that have multiple values.
    ///
    /// This is useful for identifying which parameters will vary across runs.
    #[must_use]
    pub fn multi_valued_params(&self) -> Vec<&str> {
        self.params
            .iter()
            .filter(|(_, v)| v.is_multi())
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// Check for potential misuse and return warnings.
    ///
    /// # Arguments
    ///
    /// * `run_count` - The number of runs that will be generated
    /// * `output_template` - The output path template
    /// * `raw_arguments` - The original CLI arguments (for detecting commas)
    #[must_use]
    pub fn check_warnings(
        &self,
        run_count: usize,
        output_template: &Path,
        raw_arguments: &[String],
    ) -> Vec<SpreadWarning> {
        check_warnings(
            self.mode,
            &self.params,
            &self.cases,
            run_count,
            output_template,
            raw_arguments,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spread_config_validate() -> Result<()> {
        let mut config = SpreadConfig {
            mode: SpreadMode::Grid,
            params: Parameters::new(),
            cases: Vec::new(),
            max_runs: 10,
        };

        config
            .params
            .insert("a".to_string(), ParameterValues::parse("1,2,3,4,5"));
        config
            .params
            .insert("b".to_string(), ParameterValues::parse("1,2,3,4,5"));

        // 5 * 5 = 25 > 10
        let result = config.validate();
        assert!(matches!(
            result,
            Err(SpreadError::TooManyRuns { count: 25, max: 10 })
        ));

        // Increase limit
        config.max_runs = 100;
        let run_count = config.validate()?;
        assert_eq!(run_count, 25);
        Ok(())
    }

    #[test]
    fn test_spread_config_from_arguments_grid() -> Result<()> {
        let config = SpreadConfig::from_arguments(
            SpreadMode::Grid,
            &[("region", "north,south"), ("year", "2024")],
            &[],
            100,
        )?;

        assert_eq!(config.mode, SpreadMode::Grid);
        assert!(
            config
                .params
                .get("region")
                .expect("region param exists")
                .is_multi()
        );
        assert!(
            !config
                .params
                .get("year")
                .expect("year param exists")
                .is_multi()
        );
        Ok(())
    }

    #[test]
    fn test_spread_config_from_arguments_cases() -> Result<()> {
        let config = SpreadConfig::from_arguments(
            SpreadMode::Cases,
            &[("currency", "USD")],
            &["region=north".to_string(), "region=south".to_string()],
            100,
        )?;

        assert_eq!(config.mode, SpreadMode::Cases);
        assert_eq!(config.cases.len(), 2);
        Ok(())
    }

    #[test]
    fn test_multi_valued_params() {
        let mut config = SpreadConfig::default();
        config
            .params
            .insert("region".to_string(), ParameterValues::parse("north,south"));
        config.params.insert(
            "year".to_string(),
            ParameterValues::scalar("2024".to_string()),
        );

        let multi = config.multi_valued_params();
        assert_eq!(multi, vec!["region"]);
    }
}
