/// Parsing of command line arguments (beyond that done by clap)
pub mod args {
    /// Parse a vector of command line arguments into parameters of a method call
    pub fn params(params: &[String]) -> serde_json::Value {
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
}

/// Displaying results of commands on the command line
pub mod display {
    use serde::Serialize;

    /// A result which should be displayed, usually in the console
    pub type Result = eyre::Result<Display>;

    // A calue or content to be displayed
    #[derive(Debug, Default)]
    pub struct Display {
        /// The value to be displayed
        pub value: Option<serde_json::Value>,

        /// Content representing the value
        pub content: Option<String>,

        /// Format of the content
        pub format: Option<String>,
    }

    /// A result with nothing to be displayed
    pub fn nothing() -> Result {
        Ok(Display {
            ..Default::default()
        })
    }

    /// A result with a value to be displayed
    pub fn value<Type>(value: Type) -> Result
    where
        Type: Serialize,
    {
        Ok(Display {
            value: Some(serde_json::to_value(&value)?),
            ..Default::default()
        })
    }

    /// A result with content to be displayed
    pub fn content(format: &str, content: &str) -> Result {
        Ok(Display {
            format: Some(format.into()),
            content: Some(content.into()),
            ..Default::default()
        })
    }

    /// A result with content or value to be displayed
    pub fn new<Type>(format: &str, content: &str, value: Option<Type>) -> Result
    where
        Type: Serialize,
    {
        Ok(Display {
            format: Some(format.into()),
            content: Some(content.into()),
            value: Some(serde_json::to_value(&value)?),
        })
    }
}
