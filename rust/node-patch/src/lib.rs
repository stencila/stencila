use similar::TextDiff;

use common::{
    eyre::{bail, Result},
    tracing,
};
use node_address::Address;
use node_pointer::{find_mut, resolve_mut, Pointable, PointerMut};
use stencila_schema::Node;

use crate::differ::Differ;

/// Generate a [`Patch`] describing the difference between two nodes of the same type.
#[tracing::instrument(skip(node1, node2))]
pub fn diff<Type>(node1: &Type, node2: &Type) -> Patch
where
    Type: Patchable,
{
    let mut differ = Differ::default();
    node1.diff(node2, &mut differ);
    Patch::from_ops(differ.ops)
}

/// Generate a [`Patch`] describing the difference between two nodes of the same type
/// at a specific id.
#[tracing::instrument(skip(node1, node2))]
pub fn diff_id<Type>(id: &str, node1: &Type, node2: &Type) -> Patch
where
    Type: Patchable,
{
    let mut patch = diff(node1, node2);
    patch.target = Some(id.to_string());
    patch
}

/// Generate a [`Patch`] describing the difference between two nodes of the same type
/// at a specific address.
#[tracing::instrument(skip(node1, node2))]
pub fn diff_address<Type>(address: &Address, node1: &Type, node2: &Type) -> Patch
where
    Type: Patchable,
{
    let mut patch = diff(node1, node2);
    patch.address = Some(address.clone());
    patch
}

/// Generate a [`Patch`] using a recipe function
///
/// Inspired by [Immer](https://immerjs.github.io/immer/produce/)'s `produce` function.
pub fn produce<T: Clone + Patchable, F: Fn(&mut T)>(
    node: &T,
    node_id: Option<String>,
    node_address: Option<Address>,
    recipe: F,
) -> Patch {
    let mut draft = node.clone();
    recipe(&mut draft);

    let mut patch = diff(node, &draft);
    patch.target = node_id;
    patch.address = node_address;
    patch
}

pub fn produce_address<T: Clone + Patchable, F: Fn(&mut T)>(
    node: &T,
    address: &Address,
    recipe: F,
) -> Patch {
    produce(node, None, Some(address.clone()), recipe)
}

/// Generate a [`Patch`] using a mutating function
///
/// Like [`produce`] but mutates the node as well as generating a patch.
pub fn mutate<T: Clone + Patchable, F: Fn(&mut T)>(
    node: &mut T,
    node_id: Option<String>,
    node_address: Option<Address>,
    recipe: F,
) -> Patch {
    let before = node.clone();
    recipe(node);

    let mut patch = diff(&before, node);
    patch.target = node_id;
    patch.address = node_address;
    patch
}

/// Display the difference between two nodes as a "unified diff" of the nodes
/// converted to a given format.
///
/// This can provide a more intuitive way of visualizing the differences between the
/// nodes than the raw [`Operation`]s. Note that this is slightly different from first
/// converting each node and then taking the diff in that this generates and applies a
/// patch. This means any change operations not generated or applied by the functions
/// in this module will not appear in the difference.
pub async fn diff_display(node1: &Node, node2: &Node, format: &str) -> Result<String> {
    let patch = diff(node1, node2);
    let patched = apply_new(node1, patch)?;

    let old = codecs::to_string(node1, format, None).await?;
    let new = codecs::to_string(&patched, format, None).await?;

    let mut bytes = Vec::new();
    TextDiff::from_lines(&old, &new)
        .unified_diff()
        .to_writer(&mut bytes)
        .unwrap();

    let display = String::from_utf8(bytes)?;
    Ok(display)
}

/// Apply a [`Patch`] to a node.
#[tracing::instrument(skip(node, patch))]
pub fn apply<Type>(node: &mut Type, patch: Patch) -> Result<()>
where
    Type: Patchable + Pointable,
{
    if patch.address.is_none() && patch.target.is_none() {
        node.apply_patch(patch)
    } else {
        let pointer = if let Some(address) = &patch.address {
            resolve_mut(node, address.clone())?
        } else if let Some(id) = &patch.target {
            find_mut(node, id)?
        } else {
            bail!("This should be unreachable!")
        };

        match pointer {
            PointerMut::Inline(node) => node.apply_patch(patch),
            PointerMut::Block(node) => node.apply_patch(patch),
            PointerMut::CallArgument(node) => node.apply_patch(patch),
            PointerMut::IfClause(node) => node.apply_patch(patch),
            PointerMut::Work(node) => node.apply_patch(patch),
            PointerMut::Node(node) => node.apply_patch(patch),
            PointerMut::None => bail!("Pointer is empty!"),
        }
    }
}

/// Apply a [`Patch`] to a clone of a node.
///
/// In contrast to `apply`, this does not alter the original node.
pub fn apply_new<Type>(node: &Type, patch: Patch) -> Result<Type>
where
    Type: Patchable + Clone,
{
    let mut node = node.clone();
    node.apply_patch(patch)?;
    Ok(node)
}

/// Merge changes from two or more derived versions of a node into
/// their common ancestor version.
///
/// This is equivalent to `git merge` except that there can be
/// more than two derived versions and conflicts are always resolved.
/// Conflicts are resolved by preferring the changes in 'later' derived
/// version (i.e. those that are later in the `derived` list).
///
/// # Arguments
///
/// - `ancestor`: The ancestor node
/// - `derived`: A list of derived nodes in ascending order of priority
///              when resolving merge conflicts i.e. the last in the list
///              will win over all other nodes that it conflicts with
#[tracing::instrument(skip(ancestor, derived))]
pub fn merge<Type>(ancestor: &mut Type, derived: &[&Type]) -> Result<()>
where
    Type: Patchable + Pointable,
{
    let patches: Vec<Patch> = derived.iter().map(|node| diff(ancestor, *node)).collect();

    // TODO transform operations (shift address based on other operations) and resolve conflicts
    tracing::warn!("Merging is work in progress");

    for patch in patches {
        apply(ancestor, patch)?;
    }
    Ok(())
}

mod differ;
mod errors;

mod operation;
pub use operation::Operation;

mod patch;
pub use patch::Patch;

mod patchable;
pub use patchable::Patchable;

pub mod value;
pub use value::Value;

mod prelude;

#[macro_use]
mod enums;
mod boxes;
mod options;
mod strings;
#[macro_use]
mod structs;
mod maps;
mod vecs;

mod blocks;
mod data;
mod inlines;
mod nodes;
mod others;
mod primitives;
mod works;

#[cfg(test)]
mod tests {
    use super::*;
    use stencila_schema::{
        Article, BlockContent, CodeChunk, Emphasis, ExecutionRequired, ExecutionStatus,
        InlineContent, Paragraph,
    };
    use test_utils::{assert_json_eq, assert_json_is};

    #[test]
    fn test_diff_apply() -> Result<()> {
        let empty = Paragraph::default();
        let a = Paragraph {
            content: vec![
                InlineContent::String("word1".to_string()),
                InlineContent::String("word2".to_string()),
            ],
            ..Default::default()
        };
        let b = Paragraph {
            content: vec![
                InlineContent::Emphasis(Emphasis {
                    content: vec![InlineContent::String("word1".to_string())],
                    ..Default::default()
                }),
                InlineContent::String("wotwo".to_string()),
            ],
            ..Default::default()
        };

        // Patching `empty` to `a` should return no difference

        let patch = diff(&empty, &empty);
        assert_json_is!(patch.ops, []);

        let mut patched = empty.clone();
        apply(&mut patched, patch)?;
        assert_json_eq!(patched, empty);

        // Patching `empty` to `a` should:
        // - replace all content with the content of `a`

        let patch = diff(&empty, &a);
        assert_json_is!(
            patch.ops,
            [{
                "type": "Add",
                "address": ["content", 0],
                "value": ["word1", "word2"],
                "length": 2
            }]
        );

        let mut patched = empty;
        apply(&mut patched, patch)?;
        assert_json_eq!(patched, a);

        // Patching `a` to `b` should:
        // - transform `content[0]` from a string to an `Emphasis`
        // - replace part of `content[1]`

        let patch = diff(&a, &b);
        assert_json_is!(
            patch.ops,
            [{
                "type": "Transform",
                "address": ["content", 0],
                "from": "String",
                "to": "Emphasis"
            },{
                "type": "Replace",
                "address": ["content", 1, 2],
                "items": 3,
                "value": "two",
                "length": 3
            }]
        );

        let mut patched = a;
        apply(&mut patched, patch)?;
        assert_json_eq!(patched, b);

        Ok(())
    }

    #[test]
    fn test_serialization() {
        // Empty article
        let one = Node::Article(Article {
            content: Some(vec![]),
            ..Default::default()
        });

        // Add an empty paragraph
        let two = Node::Article(Article {
            content: Some(vec![BlockContent::Paragraph(Paragraph::default())]),
            ..Default::default()
        });

        // Add words to the paragraph
        let three = Node::Article(Article {
            content: Some(vec![BlockContent::Paragraph(Paragraph {
                content: vec![
                    InlineContent::String("first".to_string()),
                    InlineContent::String(" second".to_string()),
                ],
                ..Default::default()
            })]),
            ..Default::default()
        });

        // Modify a word
        let four = Node::Article(Article {
            content: Some(vec![BlockContent::Paragraph(Paragraph {
                content: vec![
                    InlineContent::String("foot".to_string()),
                    InlineContent::String(" second".to_string()),
                ],
                ..Default::default()
            })]),
            ..Default::default()
        });

        // Move words
        let five = Node::Article(Article {
            content: Some(vec![BlockContent::Paragraph(Paragraph {
                content: vec![
                    InlineContent::String(" second".to_string()),
                    InlineContent::String("foot".to_string()),
                ],
                ..Default::default()
            })]),
            ..Default::default()
        });

        // one to one -> empty patch
        let patch = diff(&one, &one);
        assert!(patch.ops.is_empty());

        // one to two -> `Add` operation on the article's content
        let mut patch = diff(&one, &two);
        patch.prepublish(0, &two);
        assert_json_is!(
            patch.ops,
            [{
                "type": "Add",
                "address": ["content", 0],
                "value": [{"type": "Paragraph", "content": []}],
                "length": 1,
            }]
        );

        // two to three -> `Add` operation on the paragraph's content
        let mut patch = diff(&two, &three);
        patch.prepublish(0, &three);
        assert_json_is!(
            patch.ops,
            [{
                "type": "Add",
                "address": ["content", 0, "content", 0],
                "value": ["first", " second"],
                "length": 2,
            }]
        );

        // three to four -> `Replace` operation on a word
        let mut patch = diff(&three, &four);
        patch.prepublish(0, &four);
        assert_json_is!(
            patch.ops,
            [{
                "type": "Replace",
                "address": ["content", 0, "content", 0, 1],
                "items": 3,
                "value": "oo",
                "length": 2
                // No `html` because same as `value`
            }]
        );

        // four to five -> `Move` operation on the word
        let mut patch = diff(&four, &five);
        patch.prepublish(0, &five);
        assert_json_is!(
            patch.ops,
            [{
                "type": "Move",
                "from": ["content", 0, "content", 1],
                "items": 1,
                "to": ["content", 0, "content", 0],
            }]
        );
    }

    /// A regression test of serialization of an patch replacing execution status etc
    #[test]
    fn test_serialize_execute_enums() -> Result<()> {
        let patch = diff(
            &CodeChunk {
                execution_status: None,
                execution_required: Some(ExecutionRequired::NeverExecuted),
                ..Default::default()
            },
            &CodeChunk {
                execution_status: Some(ExecutionStatus::Scheduled),
                execution_required: Some(ExecutionRequired::SemanticsChanged),
                ..Default::default()
            },
        );

        assert_json_is!(patch, {
            "ops": [
                {
                    "type": "Replace",
                    "address": ["executionRequired"],
                    "items": 1,
                    "value": "SemanticsChanged",
                    "length": 1
                },
                {
                    "type": "Add",
                    "address": ["executionStatus"],
                    "value": "Scheduled",
                    "length": 1
                },
            ]
        });

        Ok(())
    }
}
