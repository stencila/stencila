use stencila_codec_info::{lost_options, lost_options_of};

use crate::{TableCellType, TableRow, TableRowType, prelude::*};

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
                .merge_losses(lost_options_of!(
                    "TableCell",
                    cell.options,
                    name,
                    vertical_alignment,
                    horizontal_alignment,
                    horizontal_alignment_character
                ));
            if let Some(value) = &cell.options.row_span {
                context.push_attr("rowspan", value);
            }
            if let Some(value) = &cell.options.column_span {
                context.push_attr("colspan", value);
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
