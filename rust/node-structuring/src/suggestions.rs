use stencila_schema::{Block, Inline, SuggestionBlock, SuggestionInline, SuggestionType};

use crate::SuggestionMetadata;

trait SuggestionNode: Sized {
    type Content;

    fn suggestion_type(&self) -> Option<SuggestionType>;
    fn into_parts(self) -> (Vec<Self::Content>, Option<Vec<Self::Content>>);
    fn metadata(&self) -> SuggestionMetadata;
    fn from_parts(
        content: Vec<Self::Content>,
        suggestion_type: SuggestionType,
        original: Option<Vec<Self::Content>>,
        metadata: SuggestionMetadata,
    ) -> Self;
}

impl SuggestionNode for SuggestionInline {
    type Content = Inline;

    fn suggestion_type(&self) -> Option<SuggestionType> {
        self.suggestion_type
    }

    fn into_parts(self) -> (Vec<Self::Content>, Option<Vec<Self::Content>>) {
        (self.content, self.original)
    }

    fn metadata(&self) -> SuggestionMetadata {
        SuggestionMetadata {
            suggestion_status: self.suggestion_status,
            authors: self.authors.clone(),
            provenance: self.provenance.clone(),
            execution_duration: self.execution_duration.clone(),
            execution_ended: self.execution_ended.clone(),
            feedback: self.feedback.clone(),
        }
    }

    fn from_parts(
        content: Vec<Self::Content>,
        suggestion_type: SuggestionType,
        original: Option<Vec<Self::Content>>,
        metadata: SuggestionMetadata,
    ) -> Self {
        let mut suggestion = SuggestionInline::new(content);
        suggestion.suggestion_type = Some(suggestion_type);
        suggestion.original = original;
        suggestion.suggestion_status = metadata.suggestion_status;
        suggestion.authors = metadata.authors;
        suggestion.provenance = metadata.provenance;
        suggestion.execution_duration = metadata.execution_duration;
        suggestion.execution_ended = metadata.execution_ended;
        suggestion.feedback = metadata.feedback;
        suggestion
    }
}

impl SuggestionNode for SuggestionBlock {
    type Content = Block;

    fn suggestion_type(&self) -> Option<SuggestionType> {
        self.suggestion_type
    }

    fn into_parts(self) -> (Vec<Self::Content>, Option<Vec<Self::Content>>) {
        (self.content, self.original)
    }

    fn metadata(&self) -> SuggestionMetadata {
        SuggestionMetadata {
            suggestion_status: self.suggestion_status,
            authors: self.authors.clone(),
            provenance: self.provenance.clone(),
            execution_duration: self.execution_duration.clone(),
            execution_ended: self.execution_ended.clone(),
            feedback: self.feedback.clone(),
        }
    }

    fn from_parts(
        content: Vec<Self::Content>,
        suggestion_type: SuggestionType,
        original: Option<Vec<Self::Content>>,
        metadata: SuggestionMetadata,
    ) -> Self {
        let mut suggestion = SuggestionBlock::new(content);
        suggestion.suggestion_type = Some(suggestion_type);
        suggestion.original = original;
        suggestion.suggestion_status = metadata.suggestion_status;
        suggestion.authors = metadata.authors;
        suggestion.provenance = metadata.provenance;
        suggestion.execution_duration = metadata.execution_duration;
        suggestion.execution_ended = metadata.execution_ended;
        suggestion.feedback = metadata.feedback;
        suggestion
    }
}

struct SuggestionAccumulator<T: SuggestionNode> {
    metadata: SuggestionMetadata,
    content: Vec<T::Content>,
    original: Vec<T::Content>,
}

impl<T: SuggestionNode> SuggestionAccumulator<T> {
    fn new(kind: SuggestionType, suggestion: T) -> Self {
        let metadata = suggestion.metadata();
        let mut accumulator = Self {
            metadata,
            content: Vec::new(),
            original: Vec::new(),
        };
        accumulator.push(kind, suggestion);
        accumulator
    }

    fn is_compatible(&self, suggestion: &T) -> bool {
        self.metadata == suggestion.metadata()
    }

    fn push(&mut self, kind: SuggestionType, suggestion: T) {
        let (mut content, original) = suggestion.into_parts();
        let mut original = original.unwrap_or_default();

        match kind {
            SuggestionType::Insert => self.content.append(&mut content),
            SuggestionType::Delete => {
                if original.is_empty() {
                    original = content;
                }
                self.original.append(&mut original);
            }
            SuggestionType::Replace => {
                self.original.append(&mut original);
                self.content.append(&mut content);
            }
        }
    }

    fn into_suggestion(self) -> T {
        let suggestion_type = if self.original.is_empty() {
            SuggestionType::Insert
        } else if self.content.is_empty() {
            SuggestionType::Delete
        } else {
            SuggestionType::Replace
        };

        T::from_parts(
            self.content,
            suggestion_type,
            (!self.original.is_empty()).then_some(self.original),
            self.metadata,
        )
    }
}

fn flush_pending<T: SuggestionNode>(
    pending: &mut Option<SuggestionAccumulator<T>>,
    normalized: &mut Vec<T>,
) {
    if let Some(accumulator) = pending.take() {
        normalized.push(accumulator.into_suggestion());
    }
}

fn normalize_suggestions<T: SuggestionNode>(suggestions: &mut Vec<T>) {
    let mut normalized = Vec::with_capacity(suggestions.len());
    let mut pending: Option<SuggestionAccumulator<T>> = None;

    for suggestion in std::mem::take(suggestions) {
        let Some(kind) = suggestion.suggestion_type() else {
            flush_pending(&mut pending, &mut normalized);
            normalized.push(suggestion);
            continue;
        };

        if pending
            .as_ref()
            .is_some_and(|accumulator| !accumulator.is_compatible(&suggestion))
        {
            flush_pending(&mut pending, &mut normalized);
        }

        if let Some(accumulator) = &mut pending {
            accumulator.push(kind, suggestion);
        } else {
            pending = Some(SuggestionAccumulator::new(kind, suggestion));
        }
    }

    flush_pending(&mut pending, &mut normalized);
    *suggestions = normalized;
}

pub(super) fn normalize_suggestion_inlines(inlines: &mut Vec<Inline>) {
    let mut normalized = Vec::with_capacity(inlines.len());
    let mut pending = Vec::new();

    for inline in std::mem::take(inlines) {
        match inline {
            Inline::SuggestionInline(suggestion) => pending.push(suggestion),
            other => {
                normalize_suggestions(&mut pending);
                normalized.extend(pending.drain(..).map(Inline::SuggestionInline));
                normalized.push(other);
            }
        }
    }

    normalize_suggestions(&mut pending);
    normalized.extend(pending.into_iter().map(Inline::SuggestionInline));
    *inlines = normalized;
}

pub(super) fn normalize_suggestion_blocks(blocks: &mut Vec<Block>) {
    let mut normalized = Vec::with_capacity(blocks.len());
    let mut pending = Vec::new();

    for block in std::mem::take(blocks) {
        match block {
            Block::SuggestionBlock(suggestion) => pending.push(suggestion),
            other => {
                normalize_suggestions(&mut pending);
                normalized.extend(pending.drain(..).map(Block::SuggestionBlock));
                normalized.push(other);
            }
        }
    }

    normalize_suggestions(&mut pending);
    normalized.extend(pending.into_iter().map(Block::SuggestionBlock));
    *blocks = normalized;
}
