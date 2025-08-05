use node_store::{
    ReadNode, WriteNode, WriteStore,
    automerge::{ObjId, Prop, ScalarValue, transaction::Transactable},
};

use crate::{Null, prelude::*};

impl StripNode for Null {}

impl PatchNode for Null {}

impl ReadNode for Null {
    fn load_null() -> Result<Self> {
        Ok(Self {})
    }
}

impl WriteNode for Null {
    fn insert_prop(&self, store: &mut WriteStore, obj_id: &ObjId, prop: Prop) -> Result<()> {
        match prop {
            Prop::Map(key) => store.put(obj_id, key, ScalarValue::Null)?,
            Prop::Seq(index) => store.insert(obj_id, index, ScalarValue::Null)?,
        };
        Ok(())
    }

    fn put_prop(&self, store: &mut WriteStore, obj_id: &ObjId, prop: Prop) -> Result<()> {
        Ok(store.put(obj_id, prop, ScalarValue::Null)?)
    }
}

impl DomCodec for Null {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        context
            .enter_elem("stencila-null")
            .push_text("null")
            .exit_elem();
    }
}

impl HtmlCodec for Null {
    fn to_html_parts(&self, _context: &mut HtmlEncodeContext) -> (&str, Vec<String>, Vec<String>) {
        ("stencila-null", vec![], vec!["null".to_string()])
    }

    fn to_html_attr(&self, _context: &mut HtmlEncodeContext) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

impl JatsCodec for Null {
    fn to_jats_parts(&self) -> (String, Vec<(String, String)>, String, Losses) {
        (String::new(), Vec::new(), self.to_text(), Losses::none())
    }
}

impl LatexCodec for Null {
    fn to_latex(&self, context: &mut LatexEncodeContext) {
        context.str(&self.to_text());
    }
}

impl MarkdownCodec for Null {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context.push_str(&self.to_text());
    }
}

impl TextCodec for Null {
    fn to_text(&self) -> String {
        self.to_string()
    }
}
