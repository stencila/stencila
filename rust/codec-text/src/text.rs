use common::eyre::Result;
use schema::{Text, TextValue};

use crate::{FromText, ToText};

impl FromText for Text {
    fn from_text(text: &str) -> Result<Self> {
        Ok(Text {
            value: TextValue(text.to_string()),
            ..Default::default()
        })
    }
}

impl ToText for Text {
    fn to_text(&self) -> Result<String> {
        Ok(self.value.0.clone())
    }
}
