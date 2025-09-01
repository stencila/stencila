use dirs::{DirType, get_app_dir};
use eyre::{Result, eyre};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

/// Generate embeddings for more that one text
///
/// TODO: add model as an argument
fn embed(texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
    let options = InitOptions::new(EmbeddingModel::BGESmallENV15)
        .with_cache_dir(get_app_dir(DirType::Models, true)?)
        .with_show_download_progress(false);

    let mut model = TextEmbedding::try_new(options).map_err(|error| eyre!(error))?;

    let vecs = model.embed(texts, None).map_err(|error| eyre!(error))?;
    Ok(vecs)
}

/// Generate embeddings for passages
///
/// Adds the "passage:" prefix which is recommended by `fastembed`.
pub fn passages(texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
    embed(
        texts
            .into_iter()
            .map(|text| format!("passage: {text}"))
            .collect(),
    )
}

/// Generate embeddings for a query
///
/// Adds the "query:" prefix which is recommended by `fastembed`.
pub fn query(text: &str) -> Result<Vec<f32>> {
    let mut vecs = embed(vec![format!("query: {text}")])?;
    Ok(vecs.swap_remove(0))
}
