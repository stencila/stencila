//! Run generation for different spread modes

use std::fmt;

use indexmap::IndexMap;
use itertools::Itertools;
use stencila_cli_utils::color_print::cformat;

use super::parsing::{CaseParameters, ParameterValues, Parameters};
use super::{Result, SpreadError, SpreadMode};

/// One concrete run with resolved parameter values.
///
/// Represents a single execution with a specific set of parameter values.
/// Parameters are sorted lexicographically by key for deterministic ordering.
#[derive(Debug, Clone)]
pub struct Run {
    /// 1-based index of this run.
    pub index: usize,
    /// Parameter values for this run, sorted lexicographically by key.
    pub values: IndexMap<String, String>,
}

impl Run {
    /// Create a new Run with values sorted lexicographically by key.
    #[must_use]
    pub fn new(index: usize, values: IndexMap<String, String>) -> Self {
        let sorted: IndexMap<String, String> = values
            .into_iter()
            .sorted_by(|(a, _), (b, _)| a.cmp(b))
            .collect();
        Run {
            index,
            values: sorted,
        }
    }

    pub fn to_terminal(&self) -> String {
        self.values
            .iter()
            .map(|(k, v)| cformat!("<c>{k}</><dim>=</><g>{v}</g>"))
            .join(" ")
    }
}

impl fmt::Display for Run {
    /// Format parameter values for display (e.g., "region=north species=ABC").
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let formatted = self
            .values
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .join(" ");
        write!(f, "{formatted}")
    }
}

/// Generate runs for grid mode (Cartesian product).
///
/// Parameters are sorted lexicographically for deterministic ordering.
/// Scalar parameters (with a single value) are included in all runs.
///
/// # Examples
///
/// With `region=north,south` and `species=ABC,DEF`, generates 4 runs:
/// 1. region=north, species=ABC
/// 2. region=north, species=DEF
/// 3. region=south, species=ABC
/// 4. region=south, species=DEF
#[must_use]
pub fn generate_runs_grid(params: &Parameters) -> Vec<Run> {
    // Separate multi-valued and scalar parameters
    let mut multi_params: Vec<(&String, &ParameterValues)> =
        params.iter().filter(|(_, v)| v.is_multi()).collect();
    let scalar_params: Vec<(&String, &ParameterValues)> =
        params.iter().filter(|(_, v)| !v.is_multi()).collect();

    // Sort multi-valued parameters lexicographically by key
    multi_params.sort_by(|(a, _), (b, _)| a.cmp(b));

    // If no multi-valued params, return single run with all scalars
    if multi_params.is_empty() {
        let values: IndexMap<String, String> = params
            .iter()
            .map(|(k, v)| (k.clone(), v.first_or_default()))
            .collect();
        return vec![Run::new(1, values)];
    }

    // Generate Cartesian product of multi-valued parameters
    let multi_values: Vec<Vec<(&String, &String)>> = multi_params
        .iter()
        .map(|(name, list)| list.values().iter().map(|v| (*name, v)).collect::<Vec<_>>())
        .collect();

    let product = multi_values
        .into_iter()
        .multi_cartesian_product()
        .collect::<Vec<_>>();

    // Build runs
    let mut runs = Vec::with_capacity(product.len());
    for (index, combo) in product.into_iter().enumerate() {
        let mut values = IndexMap::new();

        // Add scalar parameters
        for (name, list) in &scalar_params {
            values.insert((*name).clone(), list.first_or_default());
        }

        // Add this combination's values
        for (name, value) in combo {
            values.insert(name.clone(), value.clone());
        }

        runs.push(Run::new(index + 1, values));
    }

    runs
}

/// Generate runs for zip mode (positional pairing).
///
/// All multi-valued parameters must have the same length.
/// Values are paired positionally: first values together, second values together, etc.
///
/// # Errors
///
/// Returns `SpreadError::ZipLengthMismatch` if multi-valued parameters have different lengths.
///
/// # Examples
///
/// With `region=north,south` and `species=ABC,DEF`, generates 2 runs:
/// 1. region=north, species=ABC
/// 2. region=south, species=DEF
pub fn generate_runs_zip(params: &Parameters) -> Result<Vec<Run>> {
    // Separate multi-valued and scalar parameters
    let multi_params: Vec<(&String, &ParameterValues)> =
        params.iter().filter(|(_, v)| v.is_multi()).collect();
    let scalar_params: Vec<(&String, &ParameterValues)> =
        params.iter().filter(|(_, v)| !v.is_multi()).collect();

    // If no multi-valued params, return single run with all scalars
    if multi_params.is_empty() {
        let values: IndexMap<String, String> = params
            .iter()
            .map(|(k, v)| (k.clone(), v.first_or_default()))
            .collect();
        return Ok(vec![Run::new(1, values)]);
    }

    // Check that all multi-valued parameters have the same length
    let lengths: Vec<(&String, usize)> = multi_params
        .iter()
        .map(|(name, list)| (*name, list.len()))
        .collect();

    let expected_len = lengths[0].1;
    let mismatches: Vec<_> = lengths
        .iter()
        .filter(|(_, len)| *len != expected_len)
        .collect();

    if !mismatches.is_empty() {
        let details: String = lengths
            .iter()
            .map(|(name, len)| format!("{name}: {len}"))
            .join(", ");
        return Err(SpreadError::ZipLengthMismatch { details });
    }

    // Build runs by zipping
    let mut runs = Vec::with_capacity(expected_len);
    for i in 0..expected_len {
        let mut values = IndexMap::new();

        // Add scalar parameters
        for (name, list) in &scalar_params {
            values.insert((*name).clone(), list.first_or_default());
        }

        // Add values at position i from each multi-valued parameter
        for (name, list) in &multi_params {
            values.insert((*name).clone(), list.values()[i].clone());
        }

        runs.push(Run::new(i + 1, values));
    }

    Ok(runs)
}

/// Generate runs for cases mode (explicit parameter sets).
///
/// Global params are merged with each case, with case values taking precedence.
///
/// # Errors
///
/// Returns `SpreadError::NoCases` if no cases are provided.
pub fn generate_runs_cases(global: &Parameters, cases: &[CaseParameters]) -> Result<Vec<Run>> {
    if cases.is_empty() {
        return Err(SpreadError::NoCases);
    }

    let mut runs = Vec::with_capacity(cases.len());

    for (index, case) in cases.iter().enumerate() {
        let mut values = IndexMap::new();

        // Start with global params (use first value if multi-valued)
        for (key, list) in global.iter() {
            values.insert(key.clone(), list.first_or_default());
        }

        // Overlay case params (case overrides global)
        for (key, value) in case.iter() {
            values.insert(key.clone(), value.clone());
        }

        runs.push(Run::new(index + 1, values));
    }

    Ok(runs)
}

/// Compute the number of runs for a given configuration.
///
/// This is useful for validation before generating runs.
#[must_use]
pub fn count_runs(mode: SpreadMode, params: &Parameters, cases: &[CaseParameters]) -> usize {
    match mode {
        SpreadMode::Grid => {
            let multi_counts: Vec<usize> = params
                .iter()
                .filter(|(_, v)| v.is_multi())
                .map(|(_, v)| v.len())
                .collect();

            if multi_counts.is_empty() {
                1
            } else {
                multi_counts.iter().product()
            }
        }
        SpreadMode::Zip => {
            let multi_lens: Vec<usize> = params
                .iter()
                .filter(|(_, v)| v.is_multi())
                .map(|(_, v)| v.len())
                .collect();

            if multi_lens.is_empty() {
                1
            } else {
                // In zip mode, all multi-valued should have same length
                // Just return the first one (validation happens in generate_runs_zip)
                multi_lens[0]
            }
        }
        SpreadMode::Cases => cases.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_runs_grid() {
        let mut params = Parameters::new();
        params.insert("region".to_string(), ParameterValues::parse("north,south"));
        params.insert("species".to_string(), ParameterValues::parse("ABC,DEF"));
        params.insert(
            "year".to_string(),
            ParameterValues::scalar("2025".to_string()),
        );

        let runs = generate_runs_grid(&params);

        assert_eq!(runs.len(), 4);

        // Check first run
        assert_eq!(runs[0].index, 1);
        assert_eq!(runs[0].values.get("region"), Some(&"north".to_string()));
        assert_eq!(runs[0].values.get("species"), Some(&"ABC".to_string()));
        assert_eq!(runs[0].values.get("year"), Some(&"2025".to_string()));

        // Check last run
        assert_eq!(runs[3].index, 4);
        assert_eq!(runs[3].values.get("region"), Some(&"south".to_string()));
        assert_eq!(runs[3].values.get("species"), Some(&"DEF".to_string()));
    }

    #[test]
    fn test_generate_runs_zip() -> Result<()> {
        let mut params = Parameters::new();
        params.insert("region".to_string(), ParameterValues::parse("north,south"));
        params.insert("species".to_string(), ParameterValues::parse("ABC,DEF"));
        params.insert(
            "year".to_string(),
            ParameterValues::scalar("2025".to_string()),
        );

        let runs = generate_runs_zip(&params)?;

        assert_eq!(runs.len(), 2);

        assert_eq!(runs[0].values.get("region"), Some(&"north".to_string()));
        assert_eq!(runs[0].values.get("species"), Some(&"ABC".to_string()));

        assert_eq!(runs[1].values.get("region"), Some(&"south".to_string()));
        assert_eq!(runs[1].values.get("species"), Some(&"DEF".to_string()));
        Ok(())
    }

    #[test]
    fn test_generate_runs_zip_length_mismatch() {
        let mut params = Parameters::new();
        params.insert("region".to_string(), ParameterValues::parse("north,south"));
        params.insert("species".to_string(), ParameterValues::parse("ABC,DEF,GHI"));

        let result = generate_runs_zip(&params);
        assert!(matches!(result, Err(SpreadError::ZipLengthMismatch { .. })));
    }

    #[test]
    fn test_generate_runs_cases() -> Result<()> {
        use super::super::parsing::parse_case;

        let mut global = Parameters::new();
        global.insert(
            "currency".to_string(),
            ParameterValues::scalar("NZD".to_string()),
        );

        let cases = vec![
            parse_case("region=north species=ABC")?,
            parse_case("region=south species=DEF")?,
        ];

        let runs = generate_runs_cases(&global, &cases)?;

        assert_eq!(runs.len(), 2);

        assert_eq!(runs[0].values.get("currency"), Some(&"NZD".to_string()));
        assert_eq!(runs[0].values.get("region"), Some(&"north".to_string()));

        assert_eq!(runs[1].values.get("currency"), Some(&"NZD".to_string()));
        assert_eq!(runs[1].values.get("region"), Some(&"south".to_string()));
        Ok(())
    }

    #[test]
    fn test_count_runs() {
        let mut params = Parameters::new();
        params.insert("region".to_string(), ParameterValues::parse("north,south"));
        params.insert("species".to_string(), ParameterValues::parse("ABC,DEF"));

        // Grid: 2 * 2 = 4
        assert_eq!(count_runs(SpreadMode::Grid, &params, &[]), 4);

        // Zip: 2
        assert_eq!(count_runs(SpreadMode::Zip, &params, &[]), 2);

        // Cases: number of cases
        let cases = vec![
            CaseParameters::new(),
            CaseParameters::new(),
            CaseParameters::new(),
        ];
        assert_eq!(count_runs(SpreadMode::Cases, &params, &cases), 3);
    }

    #[test]
    fn test_run_display() {
        let mut values = IndexMap::new();
        values.insert("region".to_string(), "north".to_string());
        values.insert("species".to_string(), "ABC".to_string());
        let run = Run::new(1, values);

        assert_eq!(run.to_string(), "region=north species=ABC");
    }
}
