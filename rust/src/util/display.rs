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
pub fn content(format: String, content: String) -> Result {
    Ok(Display {
        format: Some(format),
        content: Some(content),
        ..Default::default()
    })
}
