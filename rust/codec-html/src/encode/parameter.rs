use stencila_schema::Parameter;

use super::{
    attr_slot, elem, elem_placeholder, elem_slot, validators::validator_tag_name, EncodeContext,
    ToHtml,
};

impl ToHtml for Parameter {
    fn to_html(&self, context: &mut EncodeContext) -> String {
        let id = self.id.to_attr("id");
        let name = self.name.to_attr("name");
        let label = self.label.to_attr("label");
        let default = self.default.to_attr("default");
        let value = self.value.to_attr("value");
        let derived_from = self.derived_from.to_attr("derived-from");
        let hidden = self.hidden.to_attr("hidden");

        let errors = elem_placeholder("span", &[attr_slot("errors")], &self.errors, context);

        let validator = elem_slot(
            &validator_tag_name(self.validator.as_deref()),
            "validator",
            &self.validator,
            context,
        );

        elem(
            "stencila-parameter",
            &[id, name, label, default, value, derived_from, hidden],
            &[String::new(), errors, validator].concat(),
        )
    }
}
