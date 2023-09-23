
/// Convenience function to find the first child with a tag name
pub(super) fn first_child_with_tag<'a, 'input>(
    node: &Node<'a, 'input>,
    tag: &str,
) -> Option<Node<'a, 'input>> {
    node.children().find(|child| child.has_tag_name(tag))
}
