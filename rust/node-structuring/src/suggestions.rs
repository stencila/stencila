use stencila_schema::{Block, Inline, SuggestionBlock, SuggestionInline, SuggestionType};

use crate::SuggestionMetadata;

trait SuggestionNode: Sized {
    type Content;

    fn suggestion_type(&self) -> Option<SuggestionType>;
    
    fn metadata(&self) -> SuggestionMetadata;

    fn into_parts(self) -> (Vec<Self::Content>, Vec<Self::Content>);

    fn from_parts(
        suggestion_type: SuggestionType,
        metadata: SuggestionMetadata,
        new: Vec<Self::Content>,
        old: Vec<Self::Content>,
    ) -> Self;
}

impl SuggestionNode for SuggestionInline {
    type Content = Inline;

    fn suggestion_type(&self) -> Option<SuggestionType> {
        self.suggestion_type
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

    fn into_parts(self) -> (Vec<Self::Content>, Vec<Self::Content>) {
        use SuggestionType::*;
        match self.suggestion_type {
            Some(Insert) => (self.content, Vec::new()),
            Some(Delete) => (Vec::new(), self.content),
            Some(Replace) | None => (self.content, self.original.unwrap_or_default()),
        }
    }

    fn from_parts(
        suggestion_type: SuggestionType,
        metadata: SuggestionMetadata,
        new: Vec<Self::Content>,
        old: Vec<Self::Content>,
    ) -> Self {
        use SuggestionType::*;
        let mut suggestion = match suggestion_type {
            Insert => SuggestionInline {
                suggestion_type: Some(suggestion_type),
                content: new,
                ..Default::default()
            },
            Delete => SuggestionInline {
                suggestion_type: Some(suggestion_type),
                content: old,
                ..Default::default()
            },
            Replace => SuggestionInline {
                suggestion_type: Some(suggestion_type),
                content: new,
                original: (!old.is_empty()).then_some(old),
                ..Default::default()
            },
        };

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

    fn into_parts(self) -> (Vec<Self::Content>, Vec<Self::Content>) {
        use SuggestionType::*;
        match self.suggestion_type {
            Some(Insert) => (self.content, Vec::new()),
            Some(Delete) => (Vec::new(), self.content),
            Some(Replace) | None => (self.content, self.original.unwrap_or_default()),
        }
    }

    fn from_parts(
        suggestion_type: SuggestionType,
        metadata: SuggestionMetadata,
        new: Vec<Self::Content>,
        old: Vec<Self::Content>,
    ) -> Self {
        use SuggestionType::*;
        let mut suggestion = match suggestion_type {
            Insert => SuggestionBlock {
                suggestion_type: Some(suggestion_type),
                content: new,
                ..Default::default()
            },
            Delete => SuggestionBlock {
                suggestion_type: Some(suggestion_type),
                content: old,
                ..Default::default()
            },
            Replace => SuggestionBlock {
                suggestion_type: Some(suggestion_type),
                content: new,
                original: (!old.is_empty()).then_some(old),
                ..Default::default()
            },
        };

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
    new: Vec<T::Content>,
    old: Vec<T::Content>,
}

impl<T: SuggestionNode> SuggestionAccumulator<T> {
    fn new(suggestion: T) -> Self {
        let metadata = suggestion.metadata();
        let mut accumulator = Self {
            metadata,
            new: Vec::new(),
            old: Vec::new(),
        };
        accumulator.push(suggestion);
        accumulator
    }

    fn is_compatible(&self, suggestion: &T) -> bool {
        self.metadata == suggestion.metadata()
    }

    fn push(&mut self, suggestion: T) {
        let (new, old) = suggestion.into_parts();
        self.old.extend(old);
        self.new.extend(new);
    }

    fn into_suggestion(self) -> T {
        let suggestion_type = if self.old.is_empty() {
            SuggestionType::Insert
        } else if self.new.is_empty() {
            SuggestionType::Delete
        } else {
            SuggestionType::Replace
        };

        T::from_parts(suggestion_type, self.metadata, self.new, self.old)
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
        if suggestion.suggestion_type().is_none() {
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
            accumulator.push(suggestion);
        } else {
            pending = Some(SuggestionAccumulator::new(suggestion));
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
