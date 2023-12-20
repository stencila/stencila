use codec_html_trait::encode::{attr, elem};

use crate::{prelude::*, Table};

impl Table {
    pub fn to_html_special(&self, context: &mut HtmlEncodeContext) -> String {
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

impl MarkdownCodec for Table {
    fn to_markdown(&self, _context: &mut MarkdownEncodeContext) {
        /*
        let mut losses = Losses::none();

        let mut column_widths: Vec<usize> = Vec::new();
        let mut rows: Vec<Vec<String>> = Vec::new();
        for row in &self.rows {
            let mut cells: Vec<String> = Vec::new();
            for (column, cell) in row.cells.iter().enumerate() {
                let (content_md, content_losses) = cell.content.to_markdown(context);

                // Trim and replace inner newlines with <br> (because content is blocks, but in
                // Markdown tables must be a single line)
                let content_md = content_md.trim().replace('\n', "<br><br>");

                let width = content_md.len();
                match column_widths.get_mut(column) {
                    Some(column_width) => {
                        if width > *column_width {
                            *column_width = width
                        }
                    }
                    None => column_widths.push(3.max(width)),
                }

                cells.push(content_md);
                losses.merge(content_losses);
            }
            rows.push(cells);
        }

        let row_to_md = |cells: &[String]| -> String {
            cells
                .iter()
                .enumerate()
                .map(|(column, content)| {
                    format!(
                        "{:width$}",
                        // Ensure cell has no newlines or pipes which will break table
                        content
                            .replace("\r\n", " ")
                            .replace('\n', " ")
                            .replace('|', "\\|"),
                        width = column_widths[column]
                    )
                })
                .join(" | ")
        };

        let (first, rest) = if rows.is_empty() {
            // If there are no rows then just return an empty string
            return (String::new(), losses);
        } else if rows.len() == 1 {
            (
                row_to_md(&vec!["".to_string(); column_widths.len()]),
                row_to_md(&rows[0]),
            )
        } else {
            (
                row_to_md(&rows[0]),
                rows[1..].iter().map(|row| row_to_md(row)).join(" |\n| "),
            )
        };

        let dashes = column_widths
            .iter()
            .map(|width| "-".repeat(*width))
            .join(" | ");

        let md = [
            "| ", &first, " |\n", "| ", &dashes, " |\n", "| ", &rest, " |\n\n",
        ]
        .concat();

        // TODO add losses for creative work properties

        (md, losses)
        */
    }
}
