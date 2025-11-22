//! Warnings for spread execution

use std::fmt;
use std::path::Path;

use super::SpreadMode;
use super::parsing::{CaseParameters, Parameters};

/// Warnings that may be emitted during spread execution.
///
/// These warnings indicate potential misconfigurations or suboptimal usage
/// that won't prevent execution but may not produce the intended results.
#[derive(Debug, Clone)]
pub enum SpreadWarning {
    /// Comma found in a parameter value when using cases mode.
    ///
    /// In cases mode, commas are not expanded into multiple values.
    /// This warning suggests the user may have intended to use grid mode.
    CommaInCasesMode(String),

    /// Spread mode is active but only one run will be generated.
    ///
    /// This suggests the spread configuration may be unnecessary or
    /// the parameters may not be configured correctly.
    SingleRunSpread,

    /// A parameter in a case is not referenced in the output template.
    ///
    /// This may indicate a typo or forgotten placeholder.
    UnusedParameter {
        /// 1-based index of the case containing the unused parameter.
        case_index: usize,
        /// Name of the unused parameter.
        param_name: String,
    },
}

impl SpreadWarning {
    /// Format the warning message for display.
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            SpreadWarning::CommaInCasesMode(param) => {
                format!(
                    "Parameter '{param}' contains comma but --spread=cases doesn't expand lists (use --spread=grid?)"
                )
            }
            SpreadWarning::SingleRunSpread => {
                "Spread mode active but only 1 run will be generated".to_string()
            }
            SpreadWarning::UnusedParameter {
                case_index,
                param_name,
            } => {
                format!(
                    "Parameter '{param_name}' in case {case_index} not referenced in output template"
                )
            }
        }
    }
}

impl fmt::Display for SpreadWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}

/// Check for potential misuse and return warnings.
///
/// # Arguments
///
/// * `mode` - The spread mode being used
/// * `params` - The global parameters
/// * `cases` - The case parameter sets (for cases mode)
/// * `run_count` - The number of runs that will be generated
/// * `output_template` - The output path template
/// * `raw_arguments` - The original CLI arguments (for detecting commas in cases mode)
#[must_use]
pub fn check_warnings(
    mode: SpreadMode,
    params: &Parameters,
    cases: &[CaseParameters],
    run_count: usize,
    output_template: &Path,
    raw_arguments: &[String],
) -> Vec<SpreadWarning> {
    let mut warnings = Vec::new();

    // Warning: comma in cases mode
    if mode == SpreadMode::Cases {
        for (name, _) in params.iter() {
            // Check original arguments for commas
            for arg in raw_arguments {
                // Parse key=value format
                if let Some((arg_name, arg_value)) = arg.split_once('=') {
                    let arg_name = arg_name.trim_start_matches('-').trim();
                    if arg_name == name && arg_value.contains(',') {
                        warnings.push(SpreadWarning::CommaInCasesMode(name.clone()));
                    }
                }
            }
        }
    }

    // Warning: single run spread
    if run_count == 1 {
        warnings.push(SpreadWarning::SingleRunSpread);
    }

    // Warning: unused parameters in cases (check against output template)
    let template_str = output_template.to_string_lossy();
    for (case_idx, case) in cases.iter().enumerate() {
        for param_name in case.keys() {
            let placeholder = format!("{{{param_name}}}");
            if !template_str.contains(&placeholder) && !params.contains_key(param_name) {
                warnings.push(SpreadWarning::UnusedParameter {
                    case_index: case_idx + 1,
                    param_name: param_name.clone(),
                });
            }
        }
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Result;
    use crate::parsing::{ParameterValues, parse_case};
    use std::path::PathBuf;

    #[test]
    fn test_warning_messages() {
        let w1 = SpreadWarning::CommaInCasesMode("region".to_string());
        assert!(w1.message().contains("region"));
        assert!(w1.message().contains("--spread=grid"));

        let w2 = SpreadWarning::SingleRunSpread;
        assert!(w2.message().contains("1 run"));

        let w3 = SpreadWarning::UnusedParameter {
            case_index: 2,
            param_name: "species".to_string(),
        };
        assert!(w3.message().contains("species"));
        assert!(w3.message().contains("case 2"));
    }

    #[test]
    fn test_check_warnings_comma_in_cases_mode() -> Result<()> {
        let mut params = Parameters::new();
        params.insert(
            "region".to_string(),
            ParameterValues::scalar("north,south".to_string()),
        );

        let warnings = check_warnings(
            SpreadMode::Cases,
            &params,
            &[parse_case("x=1")?],
            1,
            &PathBuf::from("output.pdf"),
            &["region=north,south".to_string()],
        );

        assert!(
            warnings
                .iter()
                .any(|w| matches!(w, SpreadWarning::CommaInCasesMode(_)))
        );
        Ok(())
    }

    #[test]
    fn test_check_warnings_single_run() {
        let params = Parameters::new();

        let warnings = check_warnings(
            SpreadMode::Grid,
            &params,
            &[],
            1,
            &PathBuf::from("output.pdf"),
            &[],
        );

        assert!(
            warnings
                .iter()
                .any(|w| matches!(w, SpreadWarning::SingleRunSpread))
        );
    }

    #[test]
    fn test_check_warnings_unused_parameter() -> Result<()> {
        let params = Parameters::new();
        let cases = vec![parse_case("unused_param=value")?];

        let warnings = check_warnings(
            SpreadMode::Cases,
            &params,
            &cases,
            1,
            &PathBuf::from("output.pdf"), // No {unused_param} placeholder
            &[],
        );

        assert!(warnings.iter().any(|w| matches!(
            w,
            SpreadWarning::UnusedParameter { param_name, .. } if param_name == "unused_param"
        )));
        Ok(())
    }
}
