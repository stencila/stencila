//! Configuration for site route access restrictions
//!
//! Access restrictions control which users can access specific routes on a site.
//! Access levels form a cumulative hierarchy: public < subscriber < password < team.

use std::{borrow::Cow, collections::HashMap};

use eyre::{Result, bail};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Access level for site routes
///
/// Levels form a cumulative hierarchy where higher levels include access
/// to all lower levels:
/// - `public`: Anyone can access (default)
/// - `subscriber`: Subscribers to the site
/// - `password`: Users with the site password
/// - `team`: Team members only
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum AccessLevel {
    /// Anyone can access (default)
    #[default]
    Public,
    /// Subscribers to the site
    Subscriber,
    /// Users with the site password
    Password,
    /// Team members only
    Team,
}

impl AccessLevel {
    /// Check if a user with this access level can access a route requiring the given level
    ///
    /// Returns true if `self >= required` in the access hierarchy.
    pub fn can_access(&self, required: AccessLevel) -> bool {
        *self >= required
    }
}

/// Configuration for route access restrictions
///
/// Restricts access to specific routes based on user access level.
/// Uses path prefix matching with longest match wins.
///
/// Route keys must start AND end with "/" (except "/" itself).
/// This matches the post-redirect form since the Stencila worker
/// redirects paths without trailing slash to paths with trailing slash.
///
/// ```toml
/// [site.access]
/// default = "public"
/// "/data/" = "password"
/// "/internal/" = "team"
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq, JsonSchema)]
pub struct SiteAccessConfig {
    /// Default access level for routes not explicitly configured
    ///
    /// Default: "public"
    #[serde(default)]
    pub default: AccessLevel,

    /// Route path to access level mappings
    ///
    /// Keys are route path prefixes (must start and end with "/").
    /// Values are the minimum access level required.
    /// Uses longest prefix match - more specific paths take precedence.
    #[serde(flatten)]
    pub routes: HashMap<String, AccessLevel>,
}

impl SiteAccessConfig {
    /// Validate the access configuration
    ///
    /// Checks that all route keys:
    /// - Start with "/"
    /// - End with "/" (required to match post-redirect form)
    /// - Are not less restrictive than any parent prefix (monotonic restriction)
    pub fn validate(&self) -> Result<()> {
        for key in self.routes.keys() {
            if !key.starts_with('/') {
                bail!("Invalid access config: route key \"{key}\" must start with '/'");
            }
            // Require trailing slashes to match post-redirect form
            // (Stencila worker redirects paths to trailing slash)
            if !key.ends_with('/') {
                bail!(
                    "Invalid access config: route key \"{key}\" must end with '/' (use \"{key}/\" instead)"
                );
            }
        }

        // Check monotonic restriction: child routes cannot be less restrictive than parents
        // This ensures the "top-most badge only" UI logic works correctly
        for (route, level) in &self.routes {
            // Check against default level first
            if *level < self.default {
                bail!(
                    "Invalid access config: route \"{route}\" has access level \"{level:?}\" which is less restrictive than the default \"{:?}\". Child routes cannot be less restrictive than their parent.",
                    self.default
                );
            }

            // Check against all parent prefixes
            for (other_route, other_level) in &self.routes {
                // Skip self and non-parent routes
                if other_route == route || !route.starts_with(other_route.as_str()) {
                    continue;
                }

                // other_route is a prefix of route (parent)
                // Child level must be >= parent level
                if *level < *other_level {
                    bail!(
                        "Invalid access config: route \"{route}\" has access level \"{level:?}\" which is less restrictive than its parent \"{other_route}\" with level \"{other_level:?}\". Child routes cannot be less restrictive than their parent."
                    );
                }
            }
        }

        Ok(())
    }

    /// Get the access level required for a given route path
    ///
    /// Uses longest prefix match to find the most specific route configuration.
    /// Falls back to the default access level if no route matches.
    ///
    /// Note: Paths are normalized to ensure they start with "/" and directories
    /// end with "/". This is defensive to prevent access bypass.
    pub fn get_access_level(&self, path: &str) -> AccessLevel {
        // Normalize path to ensure it starts with /
        // This prevents access bypass if callers pass unnormalized routes like "docs/page"
        let path: Cow<str> = if path.starts_with('/') {
            Cow::Borrowed(path)
        } else {
            Cow::Owned(format!("/{path}"))
        };

        // Find the longest matching prefix
        // Since route keys always end with '/', we just need prefix matching
        let mut best_match: Option<(&str, AccessLevel)> = None;

        for (route, level) in &self.routes {
            // Check if route is a prefix of path
            // Routes end with '/' so this naturally matches at path boundaries
            // e.g., "/data/" matches "/data/" and "/data/file.csv" but not "/database/"
            if path.starts_with(route.as_str()) {
                match &best_match {
                    None => best_match = Some((route, *level)),
                    Some((best_route, _)) if route.len() > best_route.len() => {
                        best_match = Some((route, *level))
                    }
                    _ => {}
                }
            }
        }

        best_match.map(|(_, level)| level).unwrap_or(self.default)
    }

    /// Check if any routes have non-public access restrictions
    pub fn has_restrictions(&self) -> bool {
        self.default != AccessLevel::Public
            || self
                .routes
                .values()
                .any(|level| *level != AccessLevel::Public)
    }

    /// Get all unique access levels used in the configuration
    ///
    /// Returns levels that have at least one route configured.
    pub fn used_access_levels(&self) -> Vec<AccessLevel> {
        let mut levels: Vec<AccessLevel> = self
            .routes
            .values()
            .copied()
            .chain(std::iter::once(self.default))
            .collect();
        levels.sort();
        levels.dedup();
        levels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_level_ordering() {
        assert!(AccessLevel::Public < AccessLevel::Subscriber);
        assert!(AccessLevel::Subscriber < AccessLevel::Password);
        assert!(AccessLevel::Password < AccessLevel::Team);
    }

    #[test]
    fn test_can_access() {
        assert!(AccessLevel::Team.can_access(AccessLevel::Public));
        assert!(AccessLevel::Team.can_access(AccessLevel::Subscriber));
        assert!(AccessLevel::Team.can_access(AccessLevel::Password));
        assert!(AccessLevel::Team.can_access(AccessLevel::Team));

        assert!(AccessLevel::Password.can_access(AccessLevel::Public));
        assert!(AccessLevel::Password.can_access(AccessLevel::Subscriber));
        assert!(AccessLevel::Password.can_access(AccessLevel::Password));
        assert!(!AccessLevel::Password.can_access(AccessLevel::Team));

        assert!(AccessLevel::Public.can_access(AccessLevel::Public));
        assert!(!AccessLevel::Public.can_access(AccessLevel::Subscriber));
    }

    #[test]
    fn test_get_access_level_default() {
        let config = SiteAccessConfig::default();
        assert_eq!(config.get_access_level("/"), AccessLevel::Public);
        assert_eq!(config.get_access_level("/any/path"), AccessLevel::Public);
    }

    #[test]
    fn test_get_access_level_prefix_match() {
        let mut routes = HashMap::new();
        routes.insert("/data/".to_string(), AccessLevel::Password);

        let config = SiteAccessConfig {
            default: AccessLevel::Public,
            routes,
        };

        assert_eq!(config.get_access_level("/data/"), AccessLevel::Password);
        assert_eq!(
            config.get_access_level("/data/file.csv"),
            AccessLevel::Password
        );
        assert_eq!(config.get_access_level("/other/"), AccessLevel::Public);
    }

    #[test]
    fn test_get_access_level_longest_prefix() {
        let mut routes = HashMap::new();
        routes.insert("/docs/".to_string(), AccessLevel::Subscriber);
        routes.insert("/docs/internal/".to_string(), AccessLevel::Team);

        let config = SiteAccessConfig {
            default: AccessLevel::Public,
            routes,
        };

        assert_eq!(config.get_access_level("/docs/"), AccessLevel::Subscriber);
        assert_eq!(
            config.get_access_level("/docs/guide/"),
            AccessLevel::Subscriber
        );
        assert_eq!(
            config.get_access_level("/docs/internal/"),
            AccessLevel::Team
        );
        assert_eq!(
            config.get_access_level("/docs/internal/secrets/"),
            AccessLevel::Team
        );
    }

    #[test]
    fn test_get_access_level_path_boundary() {
        let mut routes = HashMap::new();
        routes.insert("/data/".to_string(), AccessLevel::Password);

        let config = SiteAccessConfig {
            default: AccessLevel::Public,
            routes,
        };

        // Should match paths under /data/
        assert_eq!(
            config.get_access_level("/data/file.csv"),
            AccessLevel::Password
        );

        // Should NOT match /database/ (trailing slash in route acts as path boundary)
        assert_eq!(config.get_access_level("/database/"), AccessLevel::Public);
    }

    #[test]
    fn test_has_restrictions() {
        let config = SiteAccessConfig::default();
        assert!(!config.has_restrictions());

        let mut routes = HashMap::new();
        routes.insert("/data/".to_string(), AccessLevel::Password);
        let config = SiteAccessConfig {
            default: AccessLevel::Public,
            routes,
        };
        assert!(config.has_restrictions());

        let config = SiteAccessConfig {
            default: AccessLevel::Team,
            routes: HashMap::new(),
        };
        assert!(config.has_restrictions());
    }

    #[test]
    fn test_used_access_levels() {
        let mut routes = HashMap::new();
        routes.insert("/data/".to_string(), AccessLevel::Password);
        routes.insert("/internal/".to_string(), AccessLevel::Team);

        let config = SiteAccessConfig {
            default: AccessLevel::Public,
            routes,
        };

        let levels = config.used_access_levels();
        assert_eq!(
            levels,
            vec![
                AccessLevel::Public,
                AccessLevel::Password,
                AccessLevel::Team
            ]
        );
    }

    #[test]
    fn test_toml_deserialization() {
        let toml_str = r#"
            default = "public"
            "/data/" = "password"
            "/internal/" = "team"
        "#;

        let config: SiteAccessConfig = toml::from_str(toml_str).expect("Failed to parse TOML");
        assert_eq!(config.default, AccessLevel::Public);
        assert_eq!(config.routes.get("/data/"), Some(&AccessLevel::Password));
        assert_eq!(config.routes.get("/internal/"), Some(&AccessLevel::Team));
    }

    #[test]
    fn test_json_serialization() {
        let mut routes = HashMap::new();
        routes.insert("/data/".to_string(), AccessLevel::Password);

        let config = SiteAccessConfig {
            default: AccessLevel::Public,
            routes,
        };

        let json = serde_json::to_string(&config).expect("Failed to serialize");
        assert!(json.contains("\"default\":\"public\""));
        assert!(json.contains("\"/data/\":\"password\""));
    }

    #[test]
    fn test_get_access_level_normalizes_path() {
        // Security fix: paths without leading "/" should be normalized to prevent bypass
        let mut routes = HashMap::new();
        routes.insert("/data/".to_string(), AccessLevel::Password);

        let config = SiteAccessConfig {
            default: AccessLevel::Public,
            routes,
        };

        // Path with leading "/" should work as before
        assert_eq!(config.get_access_level("/data/"), AccessLevel::Password);

        // Path WITHOUT leading "/" should be normalized and still match
        // (Previously this would return default, potentially bypassing access control)
        assert_eq!(config.get_access_level("data/"), AccessLevel::Password);
        assert_eq!(
            config.get_access_level("data/file.csv"),
            AccessLevel::Password
        );
    }

    #[test]
    fn test_validate_accepts_valid_keys() {
        let mut routes = HashMap::new();
        routes.insert("/data/".to_string(), AccessLevel::Password);
        routes.insert("/internal/".to_string(), AccessLevel::Team);
        routes.insert("/".to_string(), AccessLevel::Public);

        let config = SiteAccessConfig {
            default: AccessLevel::Public,
            routes,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_rejects_missing_leading_slash() {
        let mut routes = HashMap::new();
        routes.insert("data".to_string(), AccessLevel::Password); // Missing leading "/"

        let config = SiteAccessConfig {
            default: AccessLevel::Public,
            routes,
        };

        let result = config.validate();
        let err = result.expect_err("validation should fail for missing leading slash");
        assert!(err.to_string().contains("must start with '/'"));
        assert!(err.to_string().contains("data"));
    }

    #[test]
    fn test_validate_rejects_missing_trailing_slash() {
        let mut routes = HashMap::new();
        routes.insert("/data".to_string(), AccessLevel::Password); // Missing trailing slash

        let config = SiteAccessConfig {
            default: AccessLevel::Public,
            routes,
        };

        let result = config.validate();
        let err = result.expect_err("validation should fail for missing trailing slash");
        assert!(err.to_string().contains("must end with '/'"));
        assert!(err.to_string().contains("/data"));
    }

    #[test]
    fn test_validate_accepts_root_slash() {
        // "/" is a valid route key (starts and ends with /)
        let mut routes = HashMap::new();
        routes.insert("/".to_string(), AccessLevel::Password);

        let config = SiteAccessConfig {
            default: AccessLevel::Public,
            routes,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_accepts_monotonic_restrictions() {
        // Child routes can be MORE restrictive than parents - this is valid
        let mut routes = HashMap::new();
        routes.insert("/docs/".to_string(), AccessLevel::Subscriber);
        routes.insert("/docs/internal/".to_string(), AccessLevel::Team);

        let config = SiteAccessConfig {
            default: AccessLevel::Public,
            routes,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_rejects_child_less_restrictive_than_parent() {
        // Child route is LESS restrictive than parent - this is invalid
        let mut routes = HashMap::new();
        routes.insert("/docs/".to_string(), AccessLevel::Password);
        routes.insert("/docs/public/".to_string(), AccessLevel::Public); // Less restrictive!

        let config = SiteAccessConfig {
            default: AccessLevel::Public,
            routes,
        };

        let result = config.validate();
        let err = result.expect_err("validation should fail for non-monotonic restriction");
        assert!(err.to_string().contains("/docs/public/"));
        assert!(err.to_string().contains("less restrictive"));
    }

    #[test]
    fn test_validate_rejects_route_less_restrictive_than_default() {
        // Route is LESS restrictive than default - this is invalid
        let mut routes = HashMap::new();
        routes.insert("/public/".to_string(), AccessLevel::Public);

        let config = SiteAccessConfig {
            default: AccessLevel::Password, // Default is password
            routes,
        };

        let result = config.validate();
        let err =
            result.expect_err("validation should fail when route is less restrictive than default");
        assert!(err.to_string().contains("/public/"));
        assert!(err.to_string().contains("less restrictive"));
    }

    #[test]
    fn test_validate_accepts_route_equal_to_default() {
        // Route equal to default is valid
        let mut routes = HashMap::new();
        routes.insert("/docs/".to_string(), AccessLevel::Password);

        let config = SiteAccessConfig {
            default: AccessLevel::Password,
            routes,
        };

        assert!(config.validate().is_ok());
    }
}
