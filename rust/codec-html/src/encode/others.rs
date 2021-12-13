use super::{
    attr, attr_itemtype, attr_prop, concat, concat_html, elem, elem_empty, nothing, EncodeContext,
    ToHtml,
};
use chrono::{DateTime, Datelike};
use stencila_schema::*;

/// Encode a `Datatable` to HTML
impl ToHtml for Datatable {
    fn to_html(&self, context: &EncodeContext) -> String {
        let columns = elem(
            "tr",
            &[attr_prop("columns")],
            &concat_html(&self.columns, context),
        );
        let rows = elem_empty("meta", &[attr_prop("rows"), attr("class", "proxy")]);
        let values = elem_empty("meta", &[attr_prop("values"), attr("class", "proxy")]);

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

/// Encode a `DatatableColumn` to HTML
impl ToHtml for DatatableColumn {
    fn to_html(&self, context: &EncodeContext) -> String {
        let name = elem("span", &[attr_prop("name")], &self.name.to_html(context));
        elem("th", &[attr_itemtype::<Self>()], &[name].concat())
    }
}

/// Encode a `Date` to HTML
///
/// Takes a similar approach to the encoding of `Cite` nodes in that it encodes parts
/// of the date as spans which the theme can choose to reorder and/or hide.
impl ToHtml for Date {
    fn to_html(&self, _context: &EncodeContext) -> String {
        let content = match DateTime::parse_from_rfc3339(&self.value) {
            Ok(datetime) => [
                elem("span", &[], &datetime.year().to_string()),
                elem("span", &[], &datetime.month().to_string()),
                elem("span", &[], &datetime.day().to_string()),
            ]
            .concat(),
            Err(error) => {
                tracing::warn!("While parsing date `{}`: {}", self.value, error);
                self.value.clone()
            }
        };
        elem("time", &[attr("datetime", &self.value)], &content)
    }
}
