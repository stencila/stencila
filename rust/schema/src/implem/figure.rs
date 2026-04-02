use stencila_codec_info::lost_options;
use stencila_format::Format;
use stencila_layout_lang::{Columns, Layout, Placement, parse as parse_layout};
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

/// SSR-ready grid styles for a figure layout and its content items.
struct GridLayoutStyles {
    container: String,
    items: Vec<String>,
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

/// Resolve a figure layout string into container and item styles for SSR.
fn resolve_grid_layout_styles(layout: &str, item_count: usize) -> Option<GridLayoutStyles> {
    if item_count == 0 {
        return None;
    }

    let layout = parse_layout(layout).ok()?.resolve_row(item_count);

    match layout {
        Layout::Auto { columns } => {
            let container = columns_style(&columns);
            let column_count = columns.column_count();
            if column_count == 0 {
                return None;
            }

            let use_gap_tracks = has_explicit_gap_tracks(&columns);
            let items = (0..item_count)
                .map(|index| {
                    let col = index % column_count;
                    let row = index / column_count;
                    placement_style(col as u32, row as u32, 1, 1, use_gap_tracks)
                })
                .collect_vec();

            Some(GridLayoutStyles { container, items })
        }
        Layout::Map {
            columns,
            placements,
        } => {
            if placements.len() != item_count {
                return None;
            }

            let container = columns_style(&columns);
            let use_gap_tracks = has_explicit_gap_tracks(&columns);
            let items = placements
                .iter()
                .map(|placement| placement_to_style(placement, use_gap_tracks))
                .collect_vec();

            Some(GridLayoutStyles { container, items })
        }
        Layout::Row => None,
    }
}

/// Whether the column spec uses explicit inline gap tracks.
fn has_explicit_gap_tracks(columns: &Columns) -> bool {
    columns.gaps.iter().any(Option::is_some)
}

/// Generate the inline grid styles for the figure container.
fn columns_style(columns: &Columns) -> String {
    let mut styles = vec!["display:grid".to_string()];

    if has_explicit_gap_tracks(columns) {
        let template = columns
            .widths
            .iter()
            .enumerate()
            .flat_map(|(index, width)| {
                let column = std::iter::once(format!("minmax(0,{width}fr)"));
                let gap = columns.gaps.get(index).map(|gap| match gap {
                    Some(gap) => format!("minmax(var(--figure-subfigure-gap),{gap}fr)"),
                    None => "var(--figure-subfigure-gap)".to_string(),
                });
                column.chain(gap)
            })
            .collect_vec()
            .join(" ");

        styles.push(format!("grid-template-columns:{template}"));
    } else {
        let template = columns
            .widths
            .iter()
            .map(|width| format!("minmax(0,{width}fr)"))
            .join(" ");

        styles.push(format!("grid-template-columns:{template}"));
        styles.push("column-gap:var(--figure-subfigure-gap)".to_string());
    }

    styles.push("row-gap:var(--figure-subfigure-gap)".to_string());

    styles.join(";")
}

/// Generate the placement style for a parsed layout placement.
fn placement_to_style(placement: &Placement, use_gap_tracks: bool) -> String {
    placement_style(
        placement.col,
        placement.row,
        placement.col_span,
        placement.row_span,
        use_gap_tracks,
    )
}

/// Generate the grid placement style for a single rendered content item.
fn placement_style(
    col: u32,
    row: u32,
    col_span: u32,
    row_span: u32,
    use_gap_tracks: bool,
) -> String {
    let grid_column_start = if use_gap_tracks {
        (col * 2) + 1
    } else {
        col + 1
    };
    let grid_column_span = if use_gap_tracks {
        (col_span * 2) - 1
    } else {
        col_span
    };

    format!(
        "grid-column:{grid_column_start} / span {grid_column_span};grid-row:{} / span {row_span}",
        row + 1
    )
}

impl DomCodec for Figure {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        let is_subfigure = context.has_ancestor(NodeType::Figure);
        let grid_layout_styles = self
            .layout
            .as_deref()
            .and_then(|layout| resolve_grid_layout_styles(layout, self.content.len()));

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
            context.enter_elem_attrs("div", [("class", "figure-content-area")]);

            if let Some(grid_layout_styles) = &grid_layout_styles {
                context.push_attr("style", &grid_layout_styles.container);

                for (index, block) in self.content.iter().enumerate() {
                    let Some(style) = grid_layout_styles.items.get(index) else {
                        block.to_dom(context);
                        continue;
                    };

                    context.enter_elem_attrs(
                        "div",
                        [("class", "figure-content-item"), ("style", style)],
                    );
                    block.to_dom(context);
                    context.exit_elem();
                }
            } else {
                self.content.to_dom(context);
            }

            if let Some(overlay) = &self.overlay {
                context.push_slot_fn("div", "overlay", |context| {
                    context.push_html(overlay);
                });
            }

            context.exit_elem();

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

            if let Some(overlay) = &self.overlay
                && matches!(context.format, Format::Smd)
            {
                let backticks = context.enclosing_backticks(overlay);

                context
                    .push_str("\n")
                    .push_indent()
                    .push_str(&backticks)
                    .push_str("svg overlay\n")
                    .push_indent()
                    .push_prop_fn(NodeProperty::Overlay, |context| {
                        overlay.to_markdown(context)
                    });

                if !overlay.ends_with('\n') {
                    context.newline();
                }

                context.push_indent().push_str(&backticks).push_str("\n\n");
            }

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
