use smol_str::SmolStr;

use node_store::{automerge::ObjId, get_node_type, ReadNode, ReadStore};

use crate::{prelude::*, *};

impl ReadNode for MessagePart {
    fn load_str(value: &SmolStr) -> Result<Self> {
        Ok(MessagePart::String(value.to_string()))
    }

    fn load_map<S: ReadStore>(store: &S, obj_id: &ObjId) -> Result<Self> {
        let Some(node_type) = get_node_type(store, obj_id)? else {
            bail!("Object in Automerge store is not a Stencila node");
        };

        Ok(match node_type {
            NodeType::AudioObject => {
                MessagePart::AudioObject(AudioObject::load_map(store, obj_id)?)
            }
            NodeType::ImageObject => {
                MessagePart::ImageObject(ImageObject::load_map(store, obj_id)?)
            }
            NodeType::VideoObject => {
                MessagePart::VideoObject(VideoObject::load_map(store, obj_id)?)
            }
            _ => bail!("Unexpected type `{node_type}` in Automerge store for `MessagePart`"),
        })
    }
}

impl<S> From<S> for MessagePart
where
    S: AsRef<str>,
{
    fn from(value: S) -> Self {
        Self::String(value.as_ref().to_string())
    }
}
