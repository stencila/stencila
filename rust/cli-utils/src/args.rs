//! Parsing of command line arguments (beyond that done by clap)

use std::collections::HashMap;

use common::{regex::Regex, serde_json};

/// Parse a vector of command line arguments into parameters of a method call
pub fn params(params: &[String]) -> HashMap<String, serde_json::Value> {
    let re = Regex::new(r"(\w+)(:?=)(.+)").unwrap();
    let mut map = HashMap::new();
    for param in params {
        if let Some(captures) = re.captures(param.as_str()) {
            let (name, kind, value) = (&captures[1], &captures[2], &captures[3]);
            let value = if kind == ":=" {
                match serde_json::from_str(value) {
                    Ok(value) => value,
                    Err(_) => serde_json::Value::String(value.to_string()),
                }
            } else {
                serde_json::Value::from(value)
            };
            map.insert(name.to_string(), value);
        }
    }
    map
}
