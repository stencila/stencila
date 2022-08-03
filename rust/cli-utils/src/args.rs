//! Parsing of command line arguments (beyond that done by clap)

use std::collections::HashMap;

use common::regex::Regex;

/// Parse a vector of command line arguments into parameters of a method call
pub fn params(params: &[String]) -> HashMap<String, String> {
    let re = Regex::new(r"(\w+)(:?=)(.+)").unwrap();
    let mut map = HashMap::new();
    for param in params {
        if let Some(captures) = re.captures(param.as_str()) {
            let (name, _kind, value) = (&captures[1], &captures[2], &captures[3]);
            map.insert(name.to_string(), value.to_string());
        }
    }
    map
}
