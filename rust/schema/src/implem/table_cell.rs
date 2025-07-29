use crate::{Block, HorizontalAlignment, Paragraph, TableCell, TableCellType, prelude::*};

impl TableCell {
    pub fn to_jats_special(&self) -> (String, Losses) {
        // Empty implementation because needs to be handled by table row
        (String::new(), Losses::none())
    }
}

impl DomCodec for TableCell {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        // Can not use a custom element (i.e. <stencila-table-cell>) because only <th> or <td> elements
        // are allowed in a <tr>.
        let name = match self.cell_type {
            Some(TableCellType::HeaderCell) => "th",
            _ => "td",
        };
        context.enter_node_elem(name, self.node_type(), self.node_id());

        if let Some(align) = self.horizontal_alignment {
            let align = match align {
                HorizontalAlignment::AlignLeft => "left",
                HorizontalAlignment::AlignCenter => "center",
                HorizontalAlignment::AlignRight => "right",
                HorizontalAlignment::AlignJustify => "justify",
                HorizontalAlignment::AlignCharacter => "char",
            };
            context.push_attr("style", &["text-align:", align].concat());
        }

        if let Some(char) = &self.horizontal_alignment_character {
            context.push_attr("char", char);
        }

        if let Some(row_span) = self.options.row_span {
            context.push_attr("rowspan", &row_span.to_string());
        }
        if let Some(column_span) = self.options.column_span {
            context.push_attr("colspan", &column_span.to_string());
        }

        // If content is a single paragraph (true most of the time)
        // then unwrap it to avoid an unnecessary <stencila-paragraph> element
        // which amongst other things can interfere with horizontal alignment.
        if let (1, Some(Block::Paragraph(Paragraph { content, .. }))) =
            (self.content.len(), self.content.first())
        {
            content.to_dom(context);
        } else {
            self.content.to_dom(context);
        }

        context.exit_node();
    }
}
