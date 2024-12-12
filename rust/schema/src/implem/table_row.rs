use codec_info::lost_options;

use crate::{prelude::*, TableCellType, TableRow, TableRowType};

impl TableRow {
    pub fn to_jats_special(&self) -> (String, Losses) {
        use codec_jats_trait::encode::{elem, elem_no_attrs};

        let mut losses = lost_options!(self, id);

        let mut cells = String::new();
        for cell in &self.cells {
            let tag = if matches!(self.row_type, Some(TableRowType::HeaderRow))
                || matches!(cell.cell_type, Some(TableCellType::HeaderCell))
            {
                "th"
            } else {
                "td"
            };

            let mut attrs = Vec::new();
            if let Some(value) = &cell.options.row_span {
                attrs.push(("rowspan", value));
            }
            if let Some(value) = &cell.options.column_span {
                attrs.push(("colspan", value));
            }

            let (cell_content, cell_losses) = cell.content.to_jats();

            let cell = elem(tag, attrs, cell_content);
            cells.push_str(&cell);

            losses.merge(cell_losses);
        }

        (elem_no_attrs("tr", cells), losses)
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
