use crate::{prelude::*, Datatable};

impl Datatable {
    /// Get the number of rows in the `Datatable`
    pub fn rows(&self) -> usize {
        self.columns
            .iter()
            .fold(0usize, |rows, column| rows.max(column.values.len()))
    }
}

impl DomCodec for Datatable {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .push_id(&self.id)
            .enter_elem("table");

        // Create a <thead><tr> elem with a <th> row describing each column
        context.enter_elem("thead").enter_elem("tr");
        for column in &self.columns {
            context.enter_elem("th");
            column.to_dom(context);
            context.exit_elem();
        }
        context.exit_elem().exit_elem();

        // Create a <tbody> elem with a <td> for each value in each column
        context.enter_elem("tbody");
        for row in 0..self.rows().min(context.max_datatable_rows) {
            context.enter_elem("tr");
            for column in &self.columns {
                context.enter_elem("td");
                if let Some(value) = column.values.get(row) {
                    let text = serde_json::to_string(value).unwrap_or_default();
                    context.push_text(&text);
                }
                context.exit_elem();
            }
            context.exit_elem();
        }
        context.exit_elem();

        context.exit_elem().exit_node();
    }
}
