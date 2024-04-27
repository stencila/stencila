use common::tokio;

/// An executable to testing the language server:
/// 
/// cargo run -p lsp
#[tokio::main]
async fn main() {
    lsp::run().await
}
