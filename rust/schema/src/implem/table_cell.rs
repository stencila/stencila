use crate::{prelude::*, TableCell, TableCellType};

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

        if let Some(row_span) = self.options.row_span {
            context.push_attr("rowspan", &row_span.to_string());
        }
        if let Some(column_span) = self.options.column_span {
            context.push_attr("colspan", &column_span.to_string());
        }

        self.content.to_dom(context);
        context.exit_node();
    }
}
