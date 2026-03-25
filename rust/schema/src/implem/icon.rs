use crate::{Icon, prelude::*};

impl DomCodec for Icon {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        let mut class = if self.name.contains(':') {
            format!("i-{}", self.name)
        } else {
            format!("i-lucide:{}", self.name)
        };

        if let Some(style) = &self.style {
            class.push(' ');
            class.push_str(style);
        }

        context
            .enter_node(self.node_type(), self.node_id())
            .push_attr("role", "img")
            .push_attr("class", &class);

        if let Some(label) = &self.label {
            context.push_attr("aria-label", label);
        }

        if self.decorative.unwrap_or(false) {
            context.push_attr("aria-hidden", "true");
        }

        context.exit_node();
    }
}

impl MarkdownCodec for Icon {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .push_str(&format!("%[{}]", self.name));

        if self.label.is_some() || self.decorative.is_some() {
            context.push_str("{");

            let mut need_space = false;
            if let Some(label) = &self.label {
                context.push_str(&format!(r#"label="{label}""#));
                need_space = true;
            }
            if let Some(decorative) = self.decorative {
                if need_space {
                    context.push_str(" ");
                }
                context.push_str(&format!("decorative={decorative}"));
            }

            context.push_str("}");
        }

        context.exit_node();
    }
}
