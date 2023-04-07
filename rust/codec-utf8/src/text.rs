use common::eyre::Result;
use schema::Text;

use crate::{FromUtf8, ToUtf8};

impl FromUtf8 for Text {
    fn from_utf8(utf8: &str) -> Result<Self> {
        Ok(Text {
            value: utf8.to_string(),
            ..Default::default()
        })
    }
}

impl ToUtf8 for Text {
    fn to_utf8(&self) -> Result<String> {
        Ok(self.value.clone())
    }
}
