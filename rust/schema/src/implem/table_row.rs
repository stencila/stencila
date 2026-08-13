use stencila_codec_info::{lost_options, lost_options_of};

use crate::{
    HorizontalAlignment, TableCellType, TableRow, TableRowType, VerticalAlignment, prelude::*,
};

impl JatsCodec for TableRow {
    fn to_jats(&self, context: &mut JatsEncodeContext) {
        context
            .enter_elem("tr")
            .merge_losses(lost_options!(self, id));
        for cell in &self.cells {
            let tag = if matches!(self.row_type, Some(TableRowType::HeaderRow))
                || matches!(cell.cell_type, Some(TableCellType::HeaderCell))
            {
                "th"
            } else {
                "td"
            };

            context
                .enter_elem(tag)
                .merge_losses(lost_options!(cell, id))
                .merge_losses(lost_options_of!("TableCell", cell.options, name));
            if let Some(value) = &cell.options.row_span {
                context.push_attr("rowspan", value);
            }
            if let Some(value) = &cell.options.column_span {
                context.push_attr("colspan", value);
            }
            if let Some(value) = &cell.options.horizontal_alignment {
                context.push_attr(
                    "align",
                    match value {
                        HorizontalAlignment::AlignLeft => "left",
                        HorizontalAlignment::AlignRight => "right",
                        HorizontalAlignment::AlignCenter => "center",
                        HorizontalAlignment::AlignJustify => "justify",
                        HorizontalAlignment::AlignCharacter => "char",
                    },
                );
            }
            if let Some(value) = &cell.options.horizontal_alignment_character {
                context.push_attr("char", value);
            }
            if let Some(value) = &cell.options.vertical_alignment {
                context.push_attr(
                    "valign",
                    match value {
                        VerticalAlignment::AlignTop => "top",
                        VerticalAlignment::AlignMiddle => "middle",
                        VerticalAlignment::AlignBottom => "bottom",
                        VerticalAlignment::AlignBaseline => "baseline",
                    },
                );
            }
            cell.content.to_jats(context);
            context.exit_elem();
        }
        context.exit_elem();
    }
}

impl DomCodec for TableRow {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        // Can not use a custom element (i.e. <stencila-table-row>) because only <tr> elements
        // are allowed in a <tbody>
        context.enter_node_elem("tr", self.node_type(), self.node_id());
        self.cells.to_dom(context);
        context.exit_node();
    }
}
