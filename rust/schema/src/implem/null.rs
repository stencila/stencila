use node_store::{
    automerge::{transaction::Transactable, ObjId, Prop, ScalarValue, Value},
    ReadNode, ReadStore, WriteNode, WriteStore, SIMILARITY_MAX,
};

use crate::{prelude::*, Null};

impl StripNode for Null {}

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

    fn similarity<S: ReadStore>(&self, store: &S, obj_id: &ObjId, prop: Prop) -> Result<usize> {
        if let Some((Value::Scalar(scalar), ..)) = store.get(obj_id, prop)? {
            if let ScalarValue::Null = *scalar {
                return Ok(SIMILARITY_MAX);
            }
        }
        Ok(0)
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
        let (content, losses) = self.to_text();
        (String::new(), Vec::new(), content, losses)
    }
}

impl MarkdownCodec for Null {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        let (text, losses) = self.to_text();
        context.push_str(&text);
        context.merge_losses(losses);
    }
}

impl TextCodec for Null {
    fn to_text(&self) -> (String, Losses) {
        (self.to_string(), Losses::one("Null@"))
    }
}
