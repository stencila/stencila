//! Project author role lists into author units without changing the source tree.

use stencila_schema::ValueKind;

use super::*;

enum AuthorUnit {
    Plain(Item),
    Roles {
        author: OccurrenceId,
        names: Vec<ScalarValue>,
    },
}

/// Whether a property is the role names aggregated onto an author unit
///
/// Only a person or organization in `authors` has this comparison-only property.
pub(crate) fn is_author_role_names(node_type: NodeType, property: NodeProperty) -> bool {
    property == NodeProperty::RoleName
        && matches!(node_type, NodeType::Person | NodeType::Organization)
}

impl Projection {
    pub(super) fn project_authors(
        &mut self,
        nodes: Vec<&dyn InspectNode>,
        path: &NodePath,
        parent: OccurrenceId,
        depth: usize,
    ) -> CompareResult<Vec<Item>> {
        let mut units = Vec::new();
        for (index, node) in nodes.into_iter().enumerate() {
            let mut path = path.clone();
            path.push_back(NodeSlot::Index(index));

            let Some((author, role)) = plain_role(node) else {
                units.push(AuthorUnit::Plain(self.project(
                    node,
                    path,
                    Some(parent),
                    Some(NodeProperty::Authors),
                    depth + 1,
                )?));
                continue;
            };

            // The wrapper adds no occurrence but still counts toward the depth limit
            self.check_depth(&path, depth + 1)?;
            path.push_back(NodeSlot::Property(NodeProperty::Author));
            let Item::Structured(author) = self.project(
                author,
                path,
                Some(parent),
                Some(NodeProperty::Authors),
                depth + 2,
            )?
            else {
                return Err(CompareError::Invariant {
                    message: "An author role's person or organization must be structured".into(),
                });
            };

            // Compare complete projected authors, not names or a fuzzy identity, so
            // differing affiliations, identifiers, etc. are never discarded. IDs are
            // ignored, as they are throughout comparison, so that an ID-only edit
            // cannot split or join a unit. Role properties are appended only after
            // this grouping pass.
            if let Some(AuthorUnit::Roles {
                author: previous,
                names,
            }) = units.last_mut()
                && self.eq_subtrees_identity_neutral(*previous, self, author)?
            {
                names.push(role);
                // The duplicate was just projected, so its entire subtree is the
                // arena's tail and nothing else can refer to it yet.
                self.occurrences.truncate(author.index());
            } else {
                units.push(AuthorUnit::Roles {
                    author,
                    names: vec![role],
                });
            }
        }

        Ok(units
            .into_iter()
            .map(|unit| {
                let (item, names) = match unit {
                    AuthorUnit::Plain(item) => (item, Vec::new()),
                    AuthorUnit::Roles { author, names } => (Item::Structured(author), names),
                };
                // Plain people and organizations get an absent property, so that a
                // missing set of roles compares as a value rather than a declaration
                if let Item::Structured(id) = &item {
                    let occurrence = &mut self.occurrences[id.index()];
                    if is_author_role_names(occurrence.node_type, NodeProperty::RoleName) {
                        occurrence.properties.push(role_names(names));
                    }
                }
                item
            })
            .collect())
    }
}

/// The comparison-only role names of an author unit, as a sorted set
fn role_names(mut names: Vec<ScalarValue>) -> ProjectedProperty {
    names.sort();
    names.dedup();
    ProjectedProperty {
        decl: PropertyDecl {
            property: NodeProperty::RoleName,
            required: false,
            repeated: true,
            kind: ValueKind::Scalar,
            slot: "AuthorRoleName",
        },
        presence: if names.is_empty() {
            Presence::Absent
        } else {
            Presence::Present
        },
        items: names.into_iter().map(Item::Scalar).collect(),
    }
}

/// Only unwrap roles whose complete information is their author and role name.
/// Other wrappers retain ordinary comparison so no role metadata is lost. A
/// wrapper's ID is not such metadata: comparison never reports ID changes.
fn plain_role(node: &dyn InspectNode) -> Option<(&dyn InspectNode, ScalarValue)> {
    if node.node_type() != Some(NodeType::AuthorRole) {
        return None;
    }
    let (mut author, mut name) = (None, None);
    for property in node.properties() {
        match (property.decl.property, property.value) {
            (NodeProperty::Author, InspectValue::One(value))
                if matches!(
                    value.node_type(),
                    Some(NodeType::Person | NodeType::Organization)
                ) =>
            {
                author = Some(value);
            }
            (NodeProperty::RoleName, InspectValue::One(value)) => {
                let ScalarRef::Enum {
                    schema_type,
                    variant,
                } = value.scalar()?
                else {
                    return None;
                };
                name = Some(ScalarValue::Enum {
                    schema_type: schema_type.into(),
                    variant: variant.into(),
                });
            }
            (NodeProperty::Id, _) | (_, InspectValue::Absent) => {}
            _ => return None,
        }
    }
    Some((author?, name?))
}
