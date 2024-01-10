use codec_losses::{lost_exec_options, lost_options};

use crate::{prelude::*, CodeChunk, LabelType};

impl CodeChunk {
    pub fn to_markdown_special(&self, context: &mut MarkdownEncodeContext) -> (String, Losses) {
        let mut losses = lost_options!(self, id, outputs);
        losses.merge(lost_exec_options!(self));

        let fence = ":".repeat(3 + context.depth * 2);

        let (wrapped, mut md) =
            if self.label_type.is_some() || self.label.is_some() || self.caption.is_some() {
                let mut md = format!("{fence} ");

                if let Some(label_type) = &self.label_type {
                    md += match label_type {
                        LabelType::FigureLabel => "figure",
                        LabelType::TableLabel => "table",
                    }
                } else {
                    md += "chunk";
                }

                if let Some(label) = &self.label {
                    md += " ";
                    md += label;
                }

                md += "\n\n";

                (true, md)
            } else {
                Default::default()
            };

        if let Some(caption) = &self.caption {
            let (caption_md, caption_losses) = caption.to_markdown(context);
            md += &caption_md;
            losses.merge(caption_losses)
        }

        md += "```";

        if let Some(lang) = &self.programming_language {
            md.push_str(lang);
            md.push(' ');
        }

        md.push_str("exec");

        if let Some(auto) = &self.auto_exec {
            md.push_str(" auto=");
            md.push_str(&auto.to_string().to_lowercase())
        }

        md.push('\n');
        md.push_str(&self.code);

        if !self.code.ends_with('\n') {
            md.push('\n');
        }

        md.push_str("```\n\n");

        if wrapped {
            md += &fence;
            md += "\n\n";
        }

        (md, losses)
    }
}
