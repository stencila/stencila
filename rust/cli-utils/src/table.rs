//! Display functions for table cells
//!
//! Reusable functions for using on attributes of `Table` structs e.g.
//!
//! ```ignore
//! #[table(title = "Created", display_fn = "date_time_ago")]
//! created_at
//! ```

pub use cli_table::Table;

use common::{
    chrono::{DateTime, Utc},
    chrono_humanize::HumanTime,
};

pub fn date_time_ago(value: &DateTime<Utc>) -> String {
    HumanTime::from(*value).to_string()
}

pub fn option_date_time_ago(value: &Option<DateTime<Utc>>) -> String {
    value
        .as_ref()
        .map(date_time_ago)
        .unwrap_or_else(|| "*Never*".to_string())
}

pub fn option_u64(value: &Option<u64>) -> String {
    value.map(|value| value.to_string()).unwrap_or_default()
}

pub fn option_string(value: &Option<String>) -> &str {
    value.as_deref().unwrap_or("")
}

pub fn bool_true_check(value: &bool) -> &str {
    match value {
        true => "✓",
        false => "",
    }
}
