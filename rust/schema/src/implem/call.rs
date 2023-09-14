use codec_json5_trait::Json5Codec;

use crate::{prelude::*, Call};

impl Call {
    pub fn to_markdown_special(&self) -> (String, Losses) {
        let mut md = ["/", &self.source, "("].concat();

        for arg in &self.arguments {
            md.push_str(&arg.name);
            md.push('=');
            md.push_str(&arg.code.to_json5().unwrap_or_default());
        }

        md.push_str(")\n\n");

        (md, Losses::todo())
    }
}
