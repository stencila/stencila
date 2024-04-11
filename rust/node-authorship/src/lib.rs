use schema::{
    Author, AuthorRole, Block, CreativeWorkType, Inline, VisitorMut, WalkControl, WalkNode,
};

/// Walk over a node and add the author roles to the authors properties of the
/// node and child nodes
pub fn author_roles<T: WalkNode>(node: &mut T, author_roles: Vec<AuthorRole>) {
    let mut authorship = Authorship { author_roles };
    node.walk_mut(&mut authorship)
}

/// A visitor that adds authors to the `authors` property
struct Authorship {
    /// The list of author roles to attribute authorship to nodes
    author_roles: Vec<AuthorRole>,
}

impl Authorship {
    /// Add the authors to the current list of authors for a node
    ///
    /// If there is already an `AuthorRole` with the same `author` and `role_name`, then
    /// this will update the `last_modified` property of that role.
    fn apply(&self, current: &mut Option<Vec<Author>>) {
        match current {
            Some(current) => {
                for author_role in &self.author_roles {
                    let mut updated = false;
                    for current_author in current.iter_mut() {
                        if let Author::AuthorRole(current_author_role) = current_author {
                            if current_author_role.author == author_role.author
                                && current_author_role.role_name == author_role.role_name
                            {
                                current_author_role.last_modified =
                                    author_role.last_modified.clone();
                                updated = true;
                                break;
                            }
                        }
                    }

                    if !updated {
                        current.push(Author::AuthorRole(author_role.clone()))
                    }
                }
            }
            None => {
                *current = Some(
                    self.author_roles
                        .clone()
                        .into_iter()
                        .map(Author::AuthorRole)
                        .collect(),
                )
            }
        }
    }
}

impl VisitorMut for Authorship {
    fn visit_inline(&mut self, inline: &mut Inline) -> WalkControl {
        macro_rules! apply {
            ($($variant:ident),*) => {
                use Inline::*;
                match inline {
                    $($variant(node) => self.apply(&mut node.options.authors),)*
                    _ => {}
                }
            };
        }

        // Should be applied to all inlines with the `authors` property
        apply!(
            AudioObject,
            CodeExpression,
            CodeInline,
            ImageObject,
            InstructionInline,
            MathInline,
            MediaObject,
            StyledInline,
            VideoObject
        );

        WalkControl::Continue
    }

    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        macro_rules! apply {
            ($($variant:ident),*) => {
                use Block::*;
                match block {
                    $($variant(node) => self.apply(&mut node.options.authors),)*
                    _ => {}
                }
            };
        }

        // Should be applied to all blocks with the `authors` property
        apply!(
            Admonition,
            Claim,
            CodeBlock,
            CodeChunk,
            Figure,
            ForBlock,
            Heading,
            IfBlock,
            InstructionBlock,
            List,
            MathBlock,
            Paragraph,
            QuoteBlock,
            StyledBlock,
            Table
        );

        WalkControl::Continue
    }

    fn visit_work(&mut self, work: &mut CreativeWorkType) -> WalkControl {
        macro_rules! apply {
            ($($variant:ident),*) => {
                use CreativeWorkType::*;
                match work {
                    // For article and comment, the authors are required
                    Article(node) => self.apply(&mut node.authors),
                    Comment(node) => self.apply(&mut node.authors),
                    $($variant(node) => self.apply(&mut node.options.authors),)*
                }
            };
        }

        // Should be applied to all creative work types since they all the `authors` property
        apply!(
            AudioObject,
            Claim,
            Collection,
            Datatable,
            Figure,
            ImageObject,
            MediaObject,
            Periodical,
            PublicationIssue,
            PublicationVolume,
            Review,
            SoftwareApplication,
            SoftwareSourceCode,
            Table,
            VideoObject
        );

        WalkControl::Continue
    }
}
