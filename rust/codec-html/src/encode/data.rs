//! HTML encoding for node types in the "data" category

use stencila_schema::*;

use super::{
    attr_itemtype, attr_prop, concat, concat_html, elem, elem_meta,
    nothing, EncodeContext, ToHtml,
};

/// Encode a `Datatable`
impl ToHtml for Datatable {
    fn to_html(&self, context: &mut EncodeContext) -> String {
        let columns = elem(
            "tr",
            &[attr_prop("columns")],
            &concat_html(&self.columns, context),
        );
        let rows = elem_meta("rows", "");
        let values = elem_meta("values", "");

        let head = elem("thead", &[], &[columns, rows, values].concat());

        let rows = self.columns.iter().fold(0, |mut rows, column| {
            let len = column.values.len();
            if len > rows {
                rows = len
            }
            rows
        });
        let rows = (0..rows)
            .into_iter()
            .map(|row| {
                let data = concat(&self.columns, |column| {
                    let data = if let Some(data) = column.values.get(row) {
                        data.to_html(context)
                    } else {
                        nothing()
                    };
                    elem("td", &[], &data)
                });
                elem("tr", &[], &data)
            })
            .collect::<Vec<String>>()
            .concat();
        let body = elem("tbody", &[], &rows);

        elem(
            "stencila-datatable",
            &[attr_itemtype::<Self>()],
            &elem("table", &[], &[head, body].concat()),
        )
    }
}

/// Encode a `DatatableColumn`
impl ToHtml for DatatableColumn {
    fn to_html(&self, context: &mut EncodeContext) -> String {
        let name = elem("span", &[attr_prop("name")], &self.name.to_html(context));
        elem("th", &[attr_itemtype::<Self>()], &[name].concat())
    }
}
