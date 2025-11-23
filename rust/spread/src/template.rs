//! Template substitution for output paths

use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use itertools::Itertools;
use regex::Regex;

use super::generation::Run;
use super::parsing::{CaseParameters, Parameters};
use super::{Result, SpreadError};

/// Regex pattern for matching placeholders like `{param}` or `{i}`.
static PLACEHOLDER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{([a-zA-Z_][a-zA-Z0-9_]*)\}").expect("valid placeholder regex"));

/// Apply template substitution to a string, replacing `{param}` and `{i}` placeholders.
///
/// # Placeholders
///
/// - `{i}` - The 1-based run index
/// - `{paramName}` - The value of the named parameter
///
/// # Errors
///
/// Returns `SpreadError::UnknownPlaceholder` if an unknown placeholder is found.
///
/// # Examples
///
/// ```
/// use stencila_spread::{Run, apply_template};
/// use indexmap::IndexMap;
///
/// let mut values = IndexMap::new();
/// values.insert("region".to_string(), "north".to_string());
/// values.insert("species".to_string(), "ABC".to_string());
/// let run = Run::new(1, values);
///
/// let result = apply_template("report-{region}-{species}.pdf", &run)?;
/// assert_eq!(result, "report-north-ABC.pdf");
///
/// let result = apply_template("report-{i}-{region}.pdf", &run)?;
/// assert_eq!(result, "report-1-north.pdf");
/// # Ok::<(), stencila_spread::SpreadError>(())
/// ```
pub fn apply_template(template: &str, run: &Run) -> Result<String> {
    let mut result = template.to_string();

    // Replace {i} with the run index
    result = result.replace("{i}", &run.index.to_string());

    // Replace {paramName} placeholders
    for cap in PLACEHOLDER_RE.captures_iter(template) {
        let Some(full_match) = cap.get(0) else {
            continue;
        };
        let Some(param_match) = cap.get(1) else {
            continue;
        };
        let full_match = full_match.as_str();
        let param_name = param_match.as_str();

        // Skip {i} since we already handled it
        if param_name == "i" {
            continue;
        }

        if let Some(value) = run.values.get(param_name) {
            result = result.replace(full_match, value);
        } else {
            let available: String = run.values.keys().join(", ");
            return Err(SpreadError::UnknownPlaceholder {
                name: param_name.to_string(),
                available,
            });
        }
    }

    Ok(result)
}

/// Check if a template contains any placeholders.
///
/// # Examples
///
/// ```
/// use stencila_spread::has_placeholders;
///
/// assert!(has_placeholders("report-{region}.pdf"));
/// assert!(has_placeholders("output-{i}.html"));
/// assert!(!has_placeholders("report.pdf"));
/// ```
#[must_use]
pub fn has_placeholders(template: &str) -> bool {
    PLACEHOLDER_RE.is_match(template)
}

/// Auto-append placeholders to an output path if none are present.
///
/// Inserts placeholders for all multi-valued parameters before the file extension,
/// sorted lexicographically.
///
/// **Note**: This function only considers global parameters. For cases mode,
/// use [`auto_append_placeholders_for_spread`] instead.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use stencila_spread::{Parameters, ParameterValues, auto_append_placeholders};
///
/// let mut params = Parameters::new();
/// params.insert("region".to_string(), ParameterValues::parse("north,south"));
/// params.insert("species".to_string(), ParameterValues::parse("ABC,DEF"));
///
/// let output = PathBuf::from("report.pdf");
/// let result = auto_append_placeholders(&output, &params);
///
/// // Multi-valued params are sorted: region, species
/// assert_eq!(result, PathBuf::from("report-{region}-{species}.pdf"));
/// ```
#[must_use]
pub fn auto_append_placeholders(output: &Path, params: &Parameters) -> PathBuf {
    auto_append_placeholders_from_names(output, get_multi_valued_param_names(params))
}

/// Auto-append placeholders for spread mode, handling all spread modes correctly.
///
/// For grid/zip modes, uses multi-valued parameter names.
/// For cases mode, collects all unique parameter names from all cases.
/// Falls back to `{i}` (run index) if multiple runs exist but no parameters vary.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use stencila_spread::{
///     SpreadMode, Parameters, CaseParameters, ParameterValues,
///     auto_append_placeholders_for_spread, parse_case,
/// };
///
/// // Cases mode: uses parameters from cases
/// let params = Parameters::new();
/// let cases = vec![
///     parse_case("region=north").unwrap(),
///     parse_case("region=south").unwrap(),
/// ];
/// let output = PathBuf::from("report.pdf");
/// let result = auto_append_placeholders_for_spread(&output, SpreadMode::Cases, &params, &cases);
/// assert_eq!(result, PathBuf::from("report-{region}.pdf"));
/// ```
#[must_use]
pub fn auto_append_placeholders_for_spread(
    output: &Path,
    mode: super::SpreadMode,
    params: &Parameters,
    cases: &[CaseParameters],
) -> PathBuf {
    // Check if the path already has placeholders
    if has_placeholders(&output.to_string_lossy()) {
        return output.to_path_buf();
    }

    let placeholder_names: Vec<String> = match mode {
        super::SpreadMode::Grid | super::SpreadMode::Zip => {
            // For grid/zip, use multi-valued parameter names
            get_multi_valued_param_names(params)
        }
        super::SpreadMode::Cases => {
            // For cases mode, collect all unique parameter names across all cases
            let mut names: Vec<String> =
                cases.iter().flat_map(|case| case.keys().cloned()).collect();
            names.sort();
            names.dedup();
            names
        }
    };

    // If we have placeholder names, use them
    if !placeholder_names.is_empty() {
        return auto_append_placeholders_from_names(output, placeholder_names);
    }

    // Fallback: if there are multiple runs but no distinguishing parameters,
    // use {i} to ensure unique filenames
    let run_count = super::generation::count_runs(mode, params, cases);
    if run_count > 1 {
        return append_suffix_to_path(output, "-{i}");
    }

    output.to_path_buf()
}

/// Get multi-valued parameter names from a Parameters map, sorted.
fn get_multi_valued_param_names(params: &Parameters) -> Vec<String> {
    params
        .iter()
        .filter(|(_, v)| v.is_multi())
        .map(|(name, _)| name.clone())
        .sorted()
        .collect()
}

/// Append placeholders from a list of parameter names to a path.
fn auto_append_placeholders_from_names(output: &Path, names: Vec<String>) -> PathBuf {
    if names.is_empty() {
        return output.to_path_buf();
    }

    // Check if the path already has placeholders
    if has_placeholders(&output.to_string_lossy()) {
        return output.to_path_buf();
    }

    // Build the suffix from parameter names
    let suffix: String = names.iter().map(|name| format!("-{{{name}}}")).join("");

    append_suffix_to_path(output, &suffix)
}

/// Append a suffix to a path, inserting before the file extension.
fn append_suffix_to_path(output: &Path, suffix: &str) -> PathBuf {
    let stem = output
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let extension = output.extension().map(|e| e.to_string_lossy().to_string());

    let new_filename = match extension {
        Some(ext) => format!("{stem}{suffix}.{ext}"),
        None => format!("{stem}{suffix}"),
    };

    output.with_file_name(new_filename)
}

/// Infer spread mode from a template string's placeholders and arguments.
///
/// Returns `Some(SpreadMode::Grid)` if:
/// - The template contains `{name}` placeholders (excluding `{i}`)
/// - At least one of those placeholder names appears in arguments with comma-separated values
///
/// This enables automatic spread mode detection without requiring the `--spread` flag.
/// Works with output path templates, route templates, or title templates.
///
/// # Examples
///
/// ```
/// use stencila_spread::{SpreadMode, infer_spread_mode};
///
/// // Placeholders with multi-valued args -> infer Grid mode
/// let args = [("region", "north,south"), ("year", "2024")];
/// assert_eq!(infer_spread_mode("report-{region}-{year}.pdf", &args), Some(SpreadMode::Grid));
///
/// // No placeholders -> None
/// assert_eq!(infer_spread_mode("report.pdf", &args), None);
///
/// // Placeholders but no multi-valued args -> None
/// let args = [("region", "north")];
/// assert_eq!(infer_spread_mode("report-{region}.pdf", &args), None);
///
/// // Only {i} placeholder -> None
/// let args = [("region", "north,south")];
/// assert_eq!(infer_spread_mode("report-{i}.pdf", &args), None);
/// ```
#[must_use]
pub fn infer_spread_mode(template: &str, arguments: &[(&str, &str)]) -> Option<super::SpreadMode> {
    // Extract placeholder names from template (excluding {i})
    let placeholder_names: std::collections::HashSet<&str> = PLACEHOLDER_RE
        .captures_iter(template)
        .filter_map(|c| c.get(1).map(|m| m.as_str()))
        .filter(|name| *name != "i")
        .collect();

    if placeholder_names.is_empty() {
        return None;
    }

    // Check if any placeholder has comma-separated values in arguments
    let has_multi_valued = arguments
        .iter()
        .any(|(name, value)| placeholder_names.contains(name) && value.contains(','));

    has_multi_valued.then_some(super::SpreadMode::Grid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;

    use super::super::parsing::ParameterValues;

    #[test]
    fn test_apply_template() -> Result<()> {
        let mut values = IndexMap::new();
        values.insert("region".to_string(), "north".to_string());
        values.insert("species".to_string(), "ABC".to_string());
        let run = Run::new(1, values);

        let result = apply_template("report-{region}-{species}.pdf", &run)?;
        assert_eq!(result, "report-north-ABC.pdf");

        let result = apply_template("report-{i}-{region}.pdf", &run)?;
        assert_eq!(result, "report-1-north.pdf");
        Ok(())
    }

    #[test]
    fn test_apply_template_unknown_placeholder() {
        let run = Run::new(1, IndexMap::new());
        let result = apply_template("report-{unknown}.pdf", &run);
        assert!(matches!(
            result,
            Err(SpreadError::UnknownPlaceholder { name, .. }) if name == "unknown"
        ));
    }

    #[test]
    fn test_has_placeholders() {
        assert!(has_placeholders("report-{region}.pdf"));
        assert!(has_placeholders("output-{i}.html"));
        assert!(has_placeholders("{a}{b}{c}"));
        assert!(!has_placeholders("report.pdf"));
        assert!(!has_placeholders("no-placeholders"));
    }

    #[test]
    fn test_auto_append_placeholders() {
        let mut params = Parameters::new();
        params.insert("region".to_string(), ParameterValues::parse("north,south"));
        params.insert("species".to_string(), ParameterValues::parse("ABC,DEF"));

        let output = PathBuf::from("report.pdf");
        let result = auto_append_placeholders(&output, &params);

        // Should be sorted: region, species
        assert_eq!(result, PathBuf::from("report-{region}-{species}.pdf"));
    }

    #[test]
    fn test_auto_append_placeholders_no_extension() {
        let mut params = Parameters::new();
        params.insert("region".to_string(), ParameterValues::parse("north,south"));

        let output = PathBuf::from("output");
        let result = auto_append_placeholders(&output, &params);

        assert_eq!(result, PathBuf::from("output-{region}"));
    }

    #[test]
    fn test_auto_append_placeholders_already_has_placeholder() {
        let mut params = Parameters::new();
        params.insert("region".to_string(), ParameterValues::parse("north,south"));

        let output = PathBuf::from("report-{i}.pdf");
        let result = auto_append_placeholders(&output, &params);

        // Should not modify since it already has placeholders
        assert_eq!(result, PathBuf::from("report-{i}.pdf"));
    }

    #[test]
    fn test_auto_append_placeholders_for_spread_cases_mode() -> super::super::Result<()> {
        use super::super::SpreadMode;
        use super::super::parsing::parse_case;

        let params = Parameters::new();
        let cases = vec![parse_case("region=north")?, parse_case("region=south")?];

        let output = PathBuf::from("report.pdf");
        let result =
            auto_append_placeholders_for_spread(&output, SpreadMode::Cases, &params, &cases);

        // Should use case parameter names
        assert_eq!(result, PathBuf::from("report-{region}.pdf"));
        Ok(())
    }

    #[test]
    fn test_auto_append_placeholders_for_spread_cases_multiple_params() -> super::super::Result<()>
    {
        use super::super::SpreadMode;
        use super::super::parsing::parse_case;

        let params = Parameters::new();
        let cases = vec![
            parse_case("region=north species=ABC")?,
            parse_case("region=south species=DEF")?,
        ];

        let output = PathBuf::from("report.pdf");
        let result =
            auto_append_placeholders_for_spread(&output, SpreadMode::Cases, &params, &cases);

        // Should be sorted alphabetically: region, species
        assert_eq!(result, PathBuf::from("report-{region}-{species}.pdf"));
        Ok(())
    }

    #[test]
    fn test_auto_append_placeholders_for_spread_grid_mode() {
        use super::super::SpreadMode;

        let mut params = Parameters::new();
        params.insert("region".to_string(), ParameterValues::parse("north,south"));

        let output = PathBuf::from("report.pdf");
        let result = auto_append_placeholders_for_spread(&output, SpreadMode::Grid, &params, &[]);

        // Should use multi-valued param names
        assert_eq!(result, PathBuf::from("report-{region}.pdf"));
    }

    #[test]
    fn test_auto_append_placeholders_for_spread_fallback_to_index() -> super::super::Result<()> {
        use super::super::SpreadMode;
        use super::super::parsing::parse_case;

        // Cases with no unique parameters (same params in both)
        let params = Parameters::new();
        let cases = vec![
            parse_case("x=1")?,
            parse_case("x=1")?, // Same param value
        ];

        let output = PathBuf::from("report.pdf");
        let result =
            auto_append_placeholders_for_spread(&output, SpreadMode::Cases, &params, &cases);

        // Should still have {x} since it's a parameter in the cases
        assert_eq!(result, PathBuf::from("report-{x}.pdf"));
        Ok(())
    }

    #[test]
    fn test_auto_append_placeholders_for_spread_already_has_placeholder() -> super::super::Result<()>
    {
        use super::super::SpreadMode;
        use super::super::parsing::parse_case;

        let params = Parameters::new();
        let cases = vec![parse_case("region=north")?, parse_case("region=south")?];

        let output = PathBuf::from("report-{i}.pdf");
        let result =
            auto_append_placeholders_for_spread(&output, SpreadMode::Cases, &params, &cases);

        // Should not modify since it already has placeholders
        assert_eq!(result, PathBuf::from("report-{i}.pdf"));
        Ok(())
    }

    #[test]
    fn test_infer_spread_mode_with_placeholders_and_multi_valued() {
        use super::super::SpreadMode;

        let args = [("region", "north,south"), ("year", "2024")];
        assert_eq!(
            infer_spread_mode("report-{region}-{year}.pdf", &args),
            Some(SpreadMode::Grid)
        );
    }

    #[test]
    fn test_infer_spread_mode_no_placeholders() {
        let args = [("region", "north,south")];
        assert_eq!(infer_spread_mode("report.pdf", &args), None);
    }

    #[test]
    fn test_infer_spread_mode_no_multi_valued() {
        let args = [("region", "north")];
        assert_eq!(infer_spread_mode("report-{region}.pdf", &args), None);
    }

    #[test]
    fn test_infer_spread_mode_only_index_placeholder() {
        let args = [("region", "north,south")];
        assert_eq!(infer_spread_mode("report-{i}.pdf", &args), None);
    }

    #[test]
    fn test_infer_spread_mode_placeholder_not_in_args() {
        let args = [("species", "cat,dog")];
        assert_eq!(infer_spread_mode("report-{region}.pdf", &args), None);
    }

    #[test]
    fn test_infer_spread_mode_nested_path() {
        use super::super::SpreadMode;

        let args = [("region", "north,south"), ("species", "cat,dog")];
        assert_eq!(
            infer_spread_mode("output/{region}/{species}/report.pdf", &args),
            Some(SpreadMode::Grid)
        );
    }

    #[test]
    fn test_infer_spread_mode_route_template() {
        use super::super::SpreadMode;

        // Route template (for push --route)
        let args = [("environ", "earth,sea"), ("region", "north,south")];
        assert_eq!(
            infer_spread_mode("/{environ}/{region}/", &args),
            Some(SpreadMode::Grid)
        );
    }

    #[test]
    fn test_infer_spread_mode_title_template() {
        use super::super::SpreadMode;

        // Title template (for push --title)
        let args = [("region", "north,south")];
        assert_eq!(
            infer_spread_mode("Report - {region}", &args),
            Some(SpreadMode::Grid)
        );
    }
}
