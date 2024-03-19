use crate::{prelude::*, DatatableColumn};

impl DomCodec for DatatableColumn {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        context
            .enter_node(self.node_type(), self.node_id())
            .push_id(&self.id)
            .push_attr("name", &self.name);

        // This does not encode the `values`` of the column since that is done,
        // row-by-row in `impl DomCodec` for the parent `Datatable`.

        if let Some(validator) = &self.validator {
            let validator = serde_json::to_string(validator).unwrap_or_default();
            context.push_attr("validator", &validator);
        }

        context.exit_node();
    }
}
