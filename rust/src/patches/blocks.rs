use super::prelude::*;
use stencila_schema::{BlockContent, NodeTrait, Paragraph};

impl Diffable for Paragraph {
    diffable_is_same!(Paragraph);

    fn is_equal(&self, other: &Self) -> Result<()> {
        self.content.is_equal(&other.content)
    }

    diffable_diff!(Paragraph);

    fn diff_same(&self, differ: &mut Differ, other: &Self) {
        differ.field("content", &self.content, &other.content)
    }

    fn apply_add(&mut self, keys: &mut Keys, value: &Box<dyn Any>) {
        if let Some(Key::Name(name)) = keys.pop_front() {
            match name.as_str() {
                "content" => self.content.apply_add(keys, value),
                _ => invalid_name!(name),
            }
        } else {
            invalid_keys!(keys)
        }
    }

    fn apply_remove(&mut self, keys: &mut Keys, items: usize) {
        if let Some(Key::Name(name)) = keys.pop_front() {
            match name.as_str() {
                "content" => self.content.apply_remove(keys, items),
                _ => invalid_name!(name),
            }
        } else {
            invalid_keys!(keys)
        }
    }

    fn apply_replace(&mut self, keys: &mut Keys, items: usize, value: &Box<dyn Any>) {
        if let Some(Key::Name(name)) = keys.pop_front() {
            match name.as_str() {
                "content" => self.content.apply_replace(keys, items, value),
                _ => invalid_name!(name),
            }
        } else {
            invalid_keys!(keys)
        }
    }

    fn apply_transform(&mut self, keys: &mut Keys, from: &str, to: &str) {
        if let Some(Key::Name(name)) = keys.pop_front() {
            match name.as_str() {
                "content" => self.content.apply_transform(keys, from, to),
                _ => invalid_name!(name),
            }
        } else {
            invalid_keys!(keys)
        }
    }
}

impl Diffable for BlockContent {
    diffable_is_same!(BlockContent);

    fn is_equal(&self, other: &Self) -> Result<()> {
        match (self, other) {
            //(BlockContent::Claim(me), BlockContent::Claim(other)) => me.is_equal(other),
            //(BlockContent::CodeBlock(me), BlockContent::CodeBlock(other)) => me.is_equal(other),
            //(BlockContent::CodeChunk(me), BlockContent::CodeChunk(other)) => me.is_equal(other),
            //(BlockContent::Collection(me), BlockContent::Collection(other)) => me.is_equal(other),
            //(BlockContent::Figure(me), BlockContent::Figure(other)) => me.is_equal(other),
            //(BlockContent::Heading(me), BlockContent::Heading(other)) => me.is_equal(other),
            //(BlockContent::List(me), BlockContent::List(other)) => me.is_equal(other),
            //(BlockContent::MathBlock(me), BlockContent::MathBlock(other)) => me.is_equal(other),
            (BlockContent::Paragraph(me), BlockContent::Paragraph(other)) => me.is_equal(other),
            //(BlockContent::QuoteBlock(me), BlockContent::QuoteBlock(other)) => me.is_equal(other),
            //(BlockContent::Table(me), BlockContent::Table(other)) => me.is_equal(other),
            //(BlockContent::ThematicBreak(me), BlockContent::ThematicBreak(other)) => me.is_equal(other)
            _ => bail!(Error::NotEqual),
        }
    }

    diffable_diff!(BlockContent);

    fn diff_same(&self, differ: &mut Differ, other: &Self) {
        match (self, other) {
            //(BlockContent::Claim(me), BlockContent::Claim(other)) => me.diff_same(other),
            //(BlockContent::CodeBlock(me), BlockContent::CodeBlock(other)) => me.diff_same(other),
            //(BlockContent::CodeChunk(me), BlockContent::CodeChunk(other)) => me.diff_same(other),
            //(BlockContent::Collection(me), BlockContent::Collection(other)) => me.diff_same(other),
            //(BlockContent::Figure(me), BlockContent::Figure(other)) => me.diff_same(other),
            //(BlockContent::Heading(me), BlockContent::Heading(other)) => me.diff_same(other),
            //(BlockContent::List(me), BlockContent::List(other)) => me.diff_same(other),
            //(BlockContent::MathBlock(me), BlockContent::MathBlock(other)) => me.diff_same(other),
            (BlockContent::Paragraph(me), BlockContent::Paragraph(other)) => me.diff_same(differ, other),
            //(BlockContent::QuoteBlock(me), BlockContent::QuoteBlock(other)) => me.diff_same(other),
            //(BlockContent::Table(me), BlockContent::Table(other)) => me.diff_same(other),
            //(BlockContent::ThematicBreak(me), BlockContent::ThematicBreak(other)) => me.diff_same(other),
            _ => todo!("transform types"),
        }
    }
}
