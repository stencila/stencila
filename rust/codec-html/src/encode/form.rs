use stencila_schema::{Form, FormDeriveAction, FormDeriveItem};

use crate::{EncodeContext, ToHtml};

use super::{attr, attr_slot, elem, elem_placeholder};

impl ToHtml for Form {
    fn to_html(&self, context: &mut EncodeContext) -> String {
        let id = self.id.to_attr("id");
        let derive_from = self.derive_from.to_attr("derive-from");
        let derive_action = self.derive_action.to_attr("derive-action");
        let derive_item = self.derive_item.to_attr("derive-item");

        let errors = elem_placeholder("div", &[attr_slot("errors")], &self.errors, context);

        let content = elem(
            "div",
            &[attr_slot("content")],
            &self.content.to_html(context),
        );

        elem(
            "stencila-form",
            &[id, derive_from, derive_action, derive_item],
            &[errors, content].concat(),
        )
    }
}

impl ToHtml for FormDeriveAction {
    fn to_attr(&self, name: &str) -> String {
        attr(name, self.as_ref())
    }
}

impl ToHtml for FormDeriveItem {
    fn to_attr(&self, name: &str) -> String {
        attr(
            name,
            &match self {
                FormDeriveItem::Integer(int) => int.to_string(),
                FormDeriveItem::String(str) => str.to_string(),
            },
        )
    }
}
