/// Normalize an OpenAlex license string to a standardized license URL
///
/// Maps common OpenAlex license identifiers to their canonical URLs. For
/// Creative Commons licenses, maps to version 4.0 URLs. For software licenses,
/// maps to standard repository URLs. Returns the original string if no mapping
/// is found.
pub fn normalize_license(license: &str) -> String {
    match license {
        // Creative Commons licenses (most common)
        "CC BY" | "cc-by" => "https://creativecommons.org/licenses/by/4.0/".to_string(),
        "CC BY-NC-ND" | "cc-by-nc-nd" => {
            "https://creativecommons.org/licenses/by-nc-nd/4.0/".to_string()
        }
        "CC BY-NC" | "cc-by-nc" => "https://creativecommons.org/licenses/by-nc/4.0/".to_string(),
        "CC BY-NC-SA" | "cc-by-nc-sa" => {
            "https://creativecommons.org/licenses/by-nc-sa/4.0/".to_string()
        }
        "CC BY-SA" | "cc-by-sa" => "https://creativecommons.org/licenses/by-sa/4.0/".to_string(),
        "CC BY-ND" | "cc-by-nd" => "https://creativecommons.org/licenses/by-nd/4.0/".to_string(),
        "public domain (CC0)" | "cc0" | "CC0" => {
            "https://creativecommons.org/publicdomain/zero/1.0/".to_string()
        }

        // Software licenses
        "MIT" => "https://opensource.org/licenses/MIT".to_string(),
        "GNU GPLv3" | "GPL-3.0" | "gpl-3.0" => {
            "https://www.gnu.org/licenses/gpl-3.0.html".to_string()
        }
        "Apache License 2.0" | "Apache-2.0" | "apache-2.0" => {
            "https://www.apache.org/licenses/LICENSE-2.0".to_string()
        }

        // Generic/unspecified licenses - keep as-is since there's no standard URL
        "other open access" | "publisher specific open access" => license.to_string(),

        // Default: return original string if no mapping found
        _ => license.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creative_commons_licenses() {
        assert_eq!(
            normalize_license("CC BY"),
            "https://creativecommons.org/licenses/by/4.0/"
        );
        assert_eq!(
            normalize_license("cc-by"),
            "https://creativecommons.org/licenses/by/4.0/"
        );
        assert_eq!(
            normalize_license("CC BY-NC-ND"),
            "https://creativecommons.org/licenses/by-nc-nd/4.0/"
        );
        assert_eq!(
            normalize_license("public domain (CC0)"),
            "https://creativecommons.org/publicdomain/zero/1.0/"
        );
    }

    #[test]
    fn test_software_licenses() {
        assert_eq!(
            normalize_license("MIT"),
            "https://opensource.org/licenses/MIT"
        );
        assert_eq!(
            normalize_license("GNU GPLv3"),
            "https://www.gnu.org/licenses/gpl-3.0.html"
        );
        assert_eq!(
            normalize_license("Apache License 2.0"),
            "https://www.apache.org/licenses/LICENSE-2.0"
        );
    }

    #[test]
    fn test_generic_licenses() {
        assert_eq!(normalize_license("other open access"), "other open access");
        assert_eq!(
            normalize_license("publisher specific open access"),
            "publisher specific open access"
        );
    }

    #[test]
    fn test_unknown_license() {
        assert_eq!(normalize_license("custom-license"), "custom-license");
    }
}
