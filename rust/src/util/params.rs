/// Parse a vector of command line arguments into parameters of a method call
pub fn parse(params: &[String]) -> serde_json::Value {
    let re = regex::Regex::new(r"(\w+)(:?=)(.+)").unwrap();
    let mut object = serde_json::json!({});
    for param in params {
        if let Some(captures) = re.captures(param.as_str()) {
            let (name, kind, value) = (&captures[1], &captures[2], &captures[3]);
            if kind == ":=" {
                object[name] = match serde_json::from_str(value) {
                    Ok(value) => value,
                    Err(_) => serde_json::Value::String(value.to_string()),
                };
            } else {
                object[name] = serde_json::Value::from(value);
            }
        }
    }
    object
}
