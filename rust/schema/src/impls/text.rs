use crate::types::Text;

impl From<&str> for Text {
    fn from(value: &str) -> Self {
        Self {
            value: value.to_string(),
            ..Default::default()
        }
    }
}
