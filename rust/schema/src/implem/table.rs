use stencila_codec_info::{lost_options, lost_options_of};

use crate::{HorizontalAlignment, Table, TableRow, TableRowType, prelude::*};

use super::utils::{
    EmptyJatsCaption, caption_to_dom, caption_to_markdown, encode_jats_figure_table_header,
    ensure_markdown_blankline,
};

impl JatsCodec for Table {
    fn to_jats(&self, context: &mut JatsEncodeContext) {
        context.enter_elem("table-wrap");
        context
            .merge_losses(lost_options!(self, authors, provenance))
            .merge_losses(lost_options_of!(
                "Table",
                self.options,
                alternate_names,
                description,
                identifiers,
                name,
                url,
                about,
                contributors,
                editors,
                maintainers,
                comments,
                date_created,
                date_received,
                date_accepted,
                date_modified,
                date_published,
                funders,
                funded_by,
                genre,
                keywords,
                is_part_of,
                licenses,
                parts,
                publisher,
                bibliography,
                references,
                text,
                title,
                repository,
                path,
                commit,
                worktree_status,
                version
            ));

        encode_jats_figure_table_header(
            self.id.as_deref(),
            self.doi.as_deref(),
            self.label.as_deref(),
            self.caption.as_deref(),
            EmptyJatsCaption::Keep,
            context,
        );

        // A table given only as an image has no rows to emit
        if self.rows.is_empty() {
            for image in self.options.images.iter().flatten() {
                image.to_jats(context);
            }
        } else {
            context.enter_elem("table");
            for (row_type, name) in [
                (Some(TableRowType::HeaderRow), "thead"),
                (None, "tbody"),
                (Some(TableRowType::FooterRow), "tfoot"),
            ] {
                context.enter_elem(name);
                for row in &self.rows {
                    let matches_group = match row_type {
                        Some(expected) => row.row_type == Some(expected),
                        None => !matches!(
                            row.row_type,
                            Some(TableRowType::HeaderRow | TableRowType::FooterRow)
                        ),
                    };
                    if matches_group {
                        row.to_jats(context);
                    }
                }
                context.exit_elem_omit_empty();
            }
            context.exit_elem();
        }

        if let Some(notes) = &self.notes {
            context.enter_elem("table-wrap-foot");
            notes.to_jats(context);
            context.exit_elem();
        }
        context.exit_elem();
    }
}

impl Table {
    pub fn to_html_special(&self, context: &mut HtmlEncodeContext) -> String {
        use stencila_codec_html_trait::encode::{attr, elem};

        let label = self
            .label
            .as_ref()
            .map(|label| elem("span", &[attr("slot", "label")], &[label.to_html(context)]));

        let caption = self.caption.as_ref().map(|caption| {
            elem(
                "span",
                &[attr("slot", "caption")],
                &[caption.to_html(context)],
            )
        });

        let caption = if label.is_some() && caption.is_some() {
            elem(
                "caption",
                &[],
                &[label.unwrap_or_default(), caption.unwrap_or_default()],
            )
        } else {
            String::new()
        };

        let body = elem("tbody", &[], &[self.rows.to_html(context)]);

        elem("table", &[], &[caption, body])
    }
}

impl DomCodec for Table {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        if let Some(label) = &self.label {
            context.push_attr("label", label);
        }

        if let Some(label_automatically) = &self.label_automatically {
            context.push_attr("label-automatically", &label_automatically.to_string());
        }

        if let Some(id_automatically) = &self.id_automatically {
            context.push_attr("id-automatically", &id_automatically.to_string());
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

        context.push_slot_fn("table", "rows", |context| {
            if (self.label.is_some() && matches!(self.label_automatically, Some(false)))
                || self.caption.is_some()
            {
                // The HTML spec requires <caption> to be within <table>. But slotted elements must be direct children
                // of the custom element (in this case, <stencila-table>). For those reasons, the caption is not
                // assigned to a slot
                context.enter_elem("caption");
                caption_to_dom(context, "table-label", "Table", &self.label, &self.caption);
                context.exit_elem();
            }

            self.rows.to_dom(context)
        });

        if self.rows.is_empty()
            && let Some(images) = &self.options.images
        {
            context.push_slot_fn("div", "images", |context| images.to_dom(context));
        }

        if let Some(notes) = &self.notes {
            context.push_slot_fn("aside", "notes", |context| notes.to_dom(context));
        }

        context.exit_node();
    }
}

impl MarkdownCodec for Table {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        use stencila_codec_markdown_trait::to_markdown;

        context
            .enter_node(self.node_type(), self.node_id())
            .merge_losses(lost_options!(self, authors, provenance));

        if matches!(context.mode, MarkdownEncodeMode::Render) {
            if self.label.is_some() || self.caption.is_some() {
                caption_to_markdown(context, "Table", &self.label, &self.caption);

                if !self.rows.is_empty() {
                    ensure_markdown_blankline(context);
                }
            }

            context.push_prop_fn(NodeProperty::Rows, |context| {
                encode_rows_to_markdown(&self.rows, context)
            });

            if let Some(notes) = &self.notes {
                ensure_markdown_blankline(context);
                context.push_prop_fn(NodeProperty::Notes, |context| notes.to_markdown(context));
            }

            ensure_markdown_blankline(context);
            context.exit_node();
            return;
        }

        if matches!(context.format, Format::Myst) {
            if self.label.is_some() || self.caption.is_some() {
                context.myst_directive(
                    ':',
                    "table",
                    |context| {
                        if let Some(caption) = &self.caption {
                            // Note: caption must be a single line
                            let caption = to_markdown(caption).replace('\n', " ");
                            context
                                .push_str(" ")
                                .push_prop_str(NodeProperty::Caption, &caption);
                        }
                    },
                    |context| {
                        if let Some(label) = &self.label {
                            context.myst_directive_option(NodeProperty::Label, None, label);
                        }
                    },
                    |context| {
                        encode_rows_to_markdown(&self.rows, context);
                        context.newline();
                    },
                );
            } else {
                encode_rows_to_markdown(&self.rows, context);
            }

            context.exit_node().newline();
        } else {
            let wrapped = if (self.label.is_some() && !self.label_automatically.unwrap_or(true))
                || (self.id.is_some() && !self.id_automatically.unwrap_or(false))
                || self.caption.is_some()
                || self.notes.is_some()
            {
                context.push_colons().push_str(" table");

                if !self.label_automatically.unwrap_or(true)
                    && let Some(label) = &self.label
                {
                    context.push_str(" ");
                    context.push_prop_str(NodeProperty::Label, label);
                }

                if !self.id_automatically.unwrap_or(false)
                    && let Some(id) = &self.id
                {
                    context.push_str(" #").push_prop_str(NodeProperty::Id, id);
                }

                context.push_str("\n\n");

                true
            } else {
                false
            };

            if let Some(caption) = &self.caption {
                context.push_prop_fn(NodeProperty::Caption, |context| {
                    caption.to_markdown(context)
                });
            }

            encode_rows_to_markdown(&self.rows, context);

            if let Some(notes) = &self.notes {
                context
                    .newline()
                    .push_prop_fn(NodeProperty::Notes, |context| notes.to_markdown(context));
            }

            if wrapped {
                if self.notes.is_none() {
                    context.newline();
                }
                context.push_colons().newline();
            }

            context.exit_node().newline();
        }
    }
}

// Encode the rows of the table to Markdown
fn encode_rows_to_markdown(self_rows: &[TableRow], context: &mut MarkdownEncodeContext) {
    // Do a first iteration over rows and cells to generate the Markdown
    // for each cell and determine column widths and alignments
    let mut column_widths: Vec<usize> = Vec::new();
    let mut column_alignments: Vec<Option<HorizontalAlignment>> = Vec::new();
    let mut rows: Vec<Vec<String>> = Vec::new();
    for row in self_rows {
        let mut cells: Vec<String> = Vec::new();
        for (column, cell) in row.cells.iter().enumerate() {
            let mut cell_context = MarkdownEncodeContext::default();
            cell.content.to_markdown(&mut cell_context);

            // Trim, replace inner newlines with <br> (because content is blocks, but in
            // Markdown tables must be a single line), & ensure cell has no carriage returns or pipes
            // which will break table
            let cell_md = cell_context
                .content
                .trim()
                .replace('\n', "<br><br>")
                .replace('\r', " ")
                .replace('|', "\\|");

            let width = cell_md.chars().count();
            match column_widths.get_mut(column) {
                Some(column_width) => {
                    if width > *column_width {
                        *column_width = width
                    }
                }
                None => column_widths.push(3.max(width)),
            }

            // Column alignment determined by the first cell with a non-None alignment
            match column_alignments.get_mut(column) {
                Some(column_alignment) => {
                    if column_alignment.is_none() && cell.options.horizontal_alignment.is_some() {
                        *column_alignment = cell.options.horizontal_alignment;
                    }
                }
                None => column_alignments.push(cell.options.horizontal_alignment),
            }

            cells.push(cell_md);
            context.merge_losses(cell_context.losses);
        }
        rows.push(cells);
    }

    // Rows
    let divider_row = |context: &mut MarkdownEncodeContext| {
        // Add indentation for SMD format
        if matches!(context.format, Format::Smd) {
            context.push_indent();
        }
        context.push_str("|");
        for (width, alignment) in column_widths.iter().zip(column_alignments.iter()) {
            match alignment {
                Some(HorizontalAlignment::AlignLeft) => {
                    context
                        .push_str(" :")
                        .push_str(&"-".repeat(width.saturating_sub(1)))
                        .push_str(" |");
                }
                Some(HorizontalAlignment::AlignCenter) => {
                    context
                        .push_str(" :")
                        .push_str(&"-".repeat(width.saturating_sub(2)))
                        .push_str(": |");
                }
                Some(HorizontalAlignment::AlignRight) => {
                    context
                        .push_str(" ")
                        .push_str(&"-".repeat(width.saturating_sub(1)))
                        .push_str(": |");
                }
                _ => {
                    context
                        .push_str(" ")
                        .push_str(&"-".repeat(*width))
                        .push_str(" |");
                }
            }
        }
        context.newline();
    };
    let empty_row = |context: &mut MarkdownEncodeContext| {
        // Add indentation for SMD format
        if matches!(context.format, Format::Smd) {
            context.push_indent();
        }
        context.push_str("|");
        for width in &column_widths {
            context.push_str(&" ".repeat(width + 2)).push_str("|");
        }
        context.newline();
    };
    for (row_index, row) in self_rows.iter().enumerate() {
        // If this is the first and only row then add an empty header if not
        // a header row and and empty body otherwise
        let (empty_header, empty_body) = if row_index == 0 && self_rows.len() == 1 {
            let empty_header = !matches!(row.row_type, Some(TableRowType::HeaderRow));
            (empty_header, !empty_header)
        } else {
            (false, false)
        };

        if empty_header {
            empty_row(context)
        }

        if empty_header || row_index == 1 {
            divider_row(context)
        }

        context.enter_node(row.node_type(), row.node_id());

        let cells = &rows[row_index];
        for (cell_index, cell) in row.cells.iter().enumerate() {
            if cell_index == 0 {
                // Add indentation for SMD format
                if matches!(context.format, Format::Smd) {
                    context.push_indent();
                }
                context.push_str("|");
            }

            context.enter_node(cell.node_type(), cell.node_id());

            let content = &cells[cell_index];
            let width = column_widths[cell_index];

            let aligned_cell = match column_alignments.get(cell_index).unwrap_or(&None) {
                Some(HorizontalAlignment::AlignLeft) => format!(" {content:<width$} "),
                Some(HorizontalAlignment::AlignCenter) => format!(" {content:^width$} "),
                Some(HorizontalAlignment::AlignRight) => format!(" {content:>width$} "),
                _ => format!(" {content:<width$} "),
            };

            context.push_str(&aligned_cell).exit_node().push_str("|");
        }
        context.newline().exit_node();

        if empty_body {
            divider_row(context);
            empty_row(context);
        }
    }
}

#[cfg(test)]
mod tests {
    use stencila_codec_markdown_trait::to_markdown_with;

    use crate::{
        Article, Block,
        shortcuts::{p, t, td, th, tr},
    };

    use super::*;

    #[test]
    fn render_markdown() {
        let mut table = Table::new(vec![tr([th([t("A")])]), tr([td([t("B")])])]);
        table.label = Some("1".to_string());
        table.caption = Some(vec![p([t("A table.")])]);

        let article = Article::new(vec![Block::Table(table), p([t("Next.")])]);
        let markdown = to_markdown_with(&article, Format::Markdown, MarkdownEncodeMode::Render);

        assert_eq!(
            markdown,
            "Table 1: A table.\n\n| A   |\n| --- |\n| B   |\n\nNext."
        );
    }
}
