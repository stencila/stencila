//! Handling of custom requests and notifications related to prompts

use async_lsp::lsp_types::request::Request;

use stencila_prompts::PromptInstance;

pub use stencila_prompts::list;

pub struct ListPrompts;

impl Request for ListPrompts {
    const METHOD: &'static str = "stencila/listPrompts";
    type Params = ();
    type Result = Vec<PromptInstance>;
}
