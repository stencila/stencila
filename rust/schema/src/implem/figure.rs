use stencila_codec_info::lost_options;
use stencila_node_type::NodeType;

use crate::{
    Block, CodeChunk, Figure, ImageObject, Inline, LabelType,
    prelude::*,
    shortcuts::{p, stg, t},
    transforms::blocks_to_inlines,
};

use super::utils::caption_to_dom;

/// A subfigure caption with its alphabetic label
struct SubfigureCaption {
    alpha: String,
    caption: Vec<Block>,
}

/// Extract the alphabetic suffix from a subfigure label like "1A" -> "A"
fn subfigure_label_to_alpha(label: &Option<String>) -> Option<String> {
    label.as_ref().and_then(|l| {
        let alpha: String = l.chars().skip_while(|c| !c.is_ascii_uppercase()).collect();
        if alpha.is_empty() { None } else { Some(alpha) }
    })
}

/// Convert a 1-based index to an alphabetic label (1→"A", 2→"B", …, 27→"AA")
fn subfigure_index_to_alpha(num: u32) -> String {
    let mut label = String::new();
    let mut n = num;
    while n > 0 {
        let remainder = (n - 1) % 26;
        label.insert(0, (b'A' + remainder as u8) as char);
        n = (n - 1) / 26;
    }
    label
}

/// Collect subfigure captions from a figure's content blocks.
///
/// Uses the alphabetic suffix from the subfigure label when available,
/// falling back to positional lettering (A, B, C, …) for uncompiled documents.
fn collect_subfigure_captions(content: &[Block]) -> Vec<SubfigureCaption> {
    let mut result = Vec::new();
    let mut position: u32 = 0;
    for block in content {
        match block {
            Block::Figure(fig) => {
                if let Some(caption) = &fig.caption {
                    position += 1;
                    let alpha = subfigure_label_to_alpha(&fig.label)
                        .unwrap_or_else(|| subfigure_index_to_alpha(position));
                    result.push(SubfigureCaption {
                        alpha,
                        caption: caption.clone(),
                    });
                }
            }
            Block::CodeChunk(CodeChunk {
                label_type: Some(LabelType::FigureLabel),
                label,
                caption: Some(caption),
                ..
            }) => {
                position += 1;
                let alpha = subfigure_label_to_alpha(label)
                    .unwrap_or_else(|| subfigure_index_to_alpha(position));
                result.push(SubfigureCaption {
                    alpha,
                    caption: caption.clone(),
                });
            }
            _ => {}
        }
    }
    result
}

impl DomCodec for Figure {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        let is_subfigure = context.has_ancestor(NodeType::Figure);

        context.enter_node(self.node_type(), self.node_id());

        if let Some(label) = &self.label {
            context.push_attr("label", label);
        }

        if let Some(label_automatically) = &self.label_automatically {
            context.push_attr("label-automatically", &label_automatically.to_string());
        }

        if let Some(layout) = &self.layout {
            context.push_attr("layout", layout);
        }

        if let Some(authors) = &self.authors {
            context.push_slot_fn("div", "authors", |context| authors.to_dom(context));
        }

        if let Some(provenance) = &self.provenance {
            context.push_slot_fn("div", "provenance", |context| provenance.to_dom(context));
        }

        if let Some(id) = &self.id {
            context
                .enter_slot("div", "id")
                .push_attr("id", id)
                .exit_slot();
        }

        let subcaptions = if is_subfigure {
            Vec::new()
        } else {
            collect_subfigure_captions(&self.content)
        };

        context.push_slot_fn("figure", "content", |context| {
            self.content.to_dom(context);

            // Subfigures do not render their own figcaption; their captions
            // are appended to the parent figure's caption instead.
            if is_subfigure {
                return;
            }

            if (self.label.is_some() && matches!(self.label_automatically, Some(false)))
                || self.caption.is_some()
                || !subcaptions.is_empty()
            {
                // Append subcaptions to caption
                let subcaptions = subcaptions
                    .iter()
                    .flat_map(|subcaption| {
                        let mut inlines = blocks_to_inlines(subcaption.caption.clone());
                        inlines.insert(0, stg([t([" (", &subcaption.alpha, ") "].concat())]));
                        inlines
                    })
                    .collect_vec();
                let caption = match self.caption.clone() {
                    Some(mut blocks) => {
                        match blocks.last_mut() {
                            Some(Block::Paragraph(para)) => para.content.extend(subcaptions),
                            _ => blocks.push(p(subcaptions)),
                        };
                        blocks
                    }
                    None => vec![p(subcaptions)],
                };

                // The HTML spec requires <figcaption> to be within <figure>. But slotted elements must be direct children
                // of the custom element (in this case, <stencila-figure>). For those reasons, the caption is not
                // assigned to a slot
                context.enter_elem("figcaption");
                caption_to_dom(
                    context,
                    "figure-label",
                    "Figure",
                    &self.label,
                    &Some(caption),
                );
                context.exit_elem();
            }
        });

        context.exit_node();
    }
}

impl MarkdownCodec for Figure {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, id, authors, provenance));

        if matches!(context.format, Format::Myst) {
            context
                .myst_directive(
                    ':',
                    "figure",
                    |context| {
                        let inlines = blocks_to_inlines(self.content.clone());
                        let mut urls = inlines.iter().filter_map(|inline| match inline {
                            Inline::ImageObject(ImageObject { content_url, .. }) => {
                                Some(content_url)
                            }
                            _ => None,
                        });
                        if let Some(url) = urls.next() {
                            context.push_str(" ").push_str(url);
                        }
                    },
                    |context| {
                        if let Some(label) = &self.label {
                            context.myst_directive_option(NodeProperty::Label, None, label);
                        }
                    },
                    |context| {
                        if let Some(caption) = &self.caption {
                            caption.to_markdown(context);
                        }
                    },
                )
                .exit_node()
                .newline();
        } else {
            context.push_colons().push_str(" figure");

            if !self.label_automatically.unwrap_or(true)
                && let Some(label) = &self.label
            {
                context.push_str(" ");
                context.push_prop_str(NodeProperty::Label, label);
            }

            if let Some(layout) = &self.layout {
                context.push_str(" [");
                context.push_prop_str(NodeProperty::Layout, layout);
                context.push_str("]");
            }

            context.push_str("\n\n");

            context
                .increase_depth()
                .push_prop_fn(NodeProperty::Content, |context| {
                    self.content.to_markdown(context)
                })
                .decrease_depth();

            // Place caption after content, following normal layout convention
            if let Some(caption) = &self.caption {
                context.push_prop_fn(NodeProperty::Caption, |context| {
                    caption.to_markdown(context)
                });
            }

            context.push_colons().newline().exit_node().newline();
        }
    }
}
