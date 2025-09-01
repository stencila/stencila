use itertools::Itertools;

use codec::schema::{Citation, CitationGroup, CitationOptions, Inline, Superscript, Text};
use codec_text_trait::to_text;

/// Normalize a vector of Stencila inlines to:
///
/// - remove parentheses and square brackets around citations,
///
/// - group citations into citation groups and remove parentheses around those
pub(super) fn normalize_inlines(input: Vec<Inline>) -> Vec<Inline> {
    let mut output = Vec::with_capacity(input.len());
    for mut inline in input.into_iter() {
        if let Inline::Citation(current) = &mut inline {
            if let Some(Inline::Text(Text { value, .. })) = output.last_mut() {
                let trimmed = value.trim().to_string();
                if value.ends_with("(") || value.ends_with("[") {
                    // Remove opening parenthesis/bracket before citation/s
                    value.pop();
                } else if trimmed == ";"
                    || trimmed == ","
                        && matches!(
                            output.iter().rev().nth(1),
                            Some(Inline::Citation(..) | Inline::CitationGroup(..))
                        )
                {
                    // Pop off semicolon or comma between citations/groups
                    output.pop();
                } else if trimmed == "-"
                    || trimmed == "–"
                        && matches!(
                            output.iter().rev().nth(1),
                            Some(Inline::Citation(..) | Inline::CitationGroup(..))
                        )
                {
                    let previous = match output.iter().rev().nth(1) {
                        Some(Inline::Citation(previous)) => Some((previous, false)),
                        Some(Inline::CitationGroup(previous)) => {
                            previous.items.last().map(|c| (c, true))
                        }
                        _ => None,
                    };

                    if let Some((previous, previous_is_group)) = previous {
                        let mut target_prefix = previous.target.chars().collect_vec();
                        while target_prefix
                            .last()
                            .map(|c| c.is_ascii_digit())
                            .unwrap_or_default()
                        {
                            target_prefix.pop();
                        }
                        let target_prefix: String = target_prefix.into_iter().collect();

                        if let (Ok(start), Ok(end)) = (
                            to_text(&previous.options.content).parse::<u32>(),
                            to_text(&current.options.content).parse::<u32>(),
                        ) && end > start
                        {
                            // Dash between two numeric citations

                            // Pop off dash
                            output.pop();

                            // Generate citations over numeric range
                            let mut items = (start..=end)
                                .map(|target| Citation {
                                    target: [target_prefix.clone(), target.to_string()].concat(),
                                    options: Box::new(CitationOptions {
                                        content: Some(vec![Inline::Text(Text::new(
                                            target.to_string().into(),
                                        ))]),
                                        ..Default::default()
                                    }),
                                    ..Default::default()
                                })
                                .collect_vec();

                            if !previous_is_group {
                                // Dash separated pair of citations so pop off the
                                // first citation and replace with a citation group with range
                                output.pop();
                                output.push(Inline::CitationGroup(CitationGroup {
                                    items,
                                    ..Default::default()
                                }));
                            } else if let Some(Inline::CitationGroup(group)) = output.last_mut() {
                                // Citation after an existing citation group so extend
                                // the group with the new items (removing the last existing first
                                // since it is the start of the new range of items)
                                group.items.pop();
                                group.items.append(&mut items);
                            }

                            continue;
                        }
                    };
                }
            }

            if matches!(output.last(), Some(Inline::Citation(..)))
                && let Some(Inline::Citation(mut previous)) = output.pop()
            {
                // Put adjacent citations into a group
                previous.citation_mode = None;
                current.citation_mode = None;
                output.push(Inline::CitationGroup(CitationGroup {
                    items: vec![previous, current.clone()],
                    ..Default::default()
                }));
                continue;
            };

            if let Some(Inline::CitationGroup(CitationGroup { items, .. })) = output.last_mut() {
                // Add citation to previous citation group
                current.citation_mode = None;
                items.push(current.clone());
                continue;
            }
        } else if let Inline::CitationGroup(current) = &mut inline {
            if let Some(Inline::Text(Text { value, .. })) = output.last()
                && value.trim() == ","
                && let Some(Inline::Citation(citation)) = output.iter().rev().nth(1).cloned()
            {
                // Comma between a citation and a citation group so pop off both and add
                // the citation to the current group
                output.pop();
                output.pop();
                current.items.push(citation);
                continue;
            }
        } else if let Inline::Superscript(Superscript { content, .. }) = &inline {
            if let (1, Some(Inline::Citation(..) | Inline::CitationGroup(..))) =
                (content.len(), content.first())
            {
                // Superscript with only a citation or citation group: replace with content
                if let Some(Inline::Text(Text { value, .. })) = output.last_mut()
                    && !value.ends_with(" ")
                {
                    value.push(' ');
                }
                output.push(content[0].clone());
                continue;
            } else if let (
                3,
                Some(Inline::Text(Text { value: before, .. })),
                Some(Inline::Citation(..) | Inline::CitationGroup(..)),
                Some(Inline::Text(Text { value: after, .. })),
            ) = (
                content.len(),
                content.first(),
                content.get(1),
                content.last(),
            ) && before.trim().is_empty()
                && after.trim().is_empty()
            {
                // Superscript with only a citation or citation group surrounded by whitespace: replace with content
                if let Some(Inline::Text(Text { value, .. })) = output.last_mut()
                    && !value.ends_with(" ")
                {
                    value.push(' ');
                }
                output.push(content[1].clone());
                continue;
            }
        } else if let Inline::Text(Text { value, .. }) = &inline
            && matches!(
                output.last(),
                Some(Inline::Citation(..) | Inline::CitationGroup(..))
            )
            && (value.starts_with(")") || value.starts_with("]"))
        {
            // Remove closing parentheses/brackets after citation/s
            output.push(Inline::Text(Text::new(value[1..].into())));
            continue;
        }

        output.push(inline)
    }

    output
}
