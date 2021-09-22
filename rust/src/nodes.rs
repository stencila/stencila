use stencila_schema::{BlockContent, InlineContent};

pub type NodeId = String;

#[derive(Debug)]
pub enum NodePointer {
    Inline(*mut InlineContent),
    Block(*mut BlockContent),
}
