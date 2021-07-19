use std::{fs, path::Path};

use crate::{files::File, sources::Elife, utils::http::download};
use eyre::{bail, Result};
use futures::future::join_all;
use minidom::Element;

/// Import an eLife article into a project
pub async fn import(
    project: &Path,
    source: &Elife,
    destination: Option<String>,
) -> Result<Vec<File>> {
    let url = format!("https://elifesciences.org/articles/{}.xml", source.article);
    let response = reqwest::get(&url).await?;
    let response = match response.error_for_status() {
        Ok(response) => response,
        Err(error) => {
            if let Some(status) = error.status() {
                if status == 404 {
                    bail!("Not found '{}', is the article number correct?", url)
                }
            }
            bail!(error)
        }
    };

    let xml = response.text().await?;
    // Parsing requires that the root element have a namespace, so insert an
    // arbitrary one.
    let xml = xml.replace("<article ", "<article xmlns=\"article\"");
    let mut root: Element = xml.parse()?;

    let destination =
        project.join(destination.unwrap_or_else(|| format!("elife-{}.xml", source.article)));
    let parent_dir = destination.parent().expect("Should always have a parent");
    let file_name = destination
        .file_name()
        .expect("Should always have a file name")
        .to_string_lossy()
        .to_string();
    let media_dir = format!("{}.media", file_name);

    // Walk the tree, collecting the URLs of images to download and rewriting
    // the `href` attributes of `graphic` elements to local JPG files (instead of TIFs).
    fn walk(article: u32, media_dir: &str, elem: &mut Element) -> Vec<(String, String)> {
        let mut images = Vec::new();
        for child in elem.children_mut() {
            if child.name() == "graphic" {
                let mut rewritten = false;
                for (name, value) in child.attrs_mut() {
                    if name == "xlink:href" && value.starts_with("elife") {
                        let href = if value.ends_with(".tif") {
                            value.clone()
                        } else {
                            [value, ".tif"].concat()
                        };
                        let url = format!(
                            "https://iiif.elifesciences.org/lax:{article}%2F{href}/full/full/0/default.jpg",
                            article = article,
                            href = href
                        );
                        let relative_path = [
                            media_dir,
                            "/",
                            &href.replace(&format!("elife-{}-", article), "")
                                .replace(".tif", ".jpg"),
                        ]
                        .concat();

                        *value = relative_path.clone();
                        images.push((url, relative_path));
                        rewritten = true;
                        break;
                    }
                }
                if rewritten {
                    child.set_attr("mime-subtype", "jpeg")
                }
            }
            images.append(&mut walk(article, media_dir, child));
        }
        images
    }
    let images = walk(source.article, &media_dir, &mut root);

    // Write the re-written XML
    let mut writer = fs::File::create(parent_dir.join(file_name))?;
    root.write_to(&mut writer)?;

    // If there are any images then ensure the media directory is created (and empty)
    if !images.is_empty() {
        let dir = parent_dir.join(media_dir);
        if dir.exists() {
            fs::remove_dir_all(&dir)?
        };
        fs::create_dir_all(&dir)?;
    }

    // Download images concurrently
    let results = join_all(
        images
            .iter()
            .map(|(url, path)| download(url, parent_dir.join(path))),
    )
    .await;
    results.iter().for_each(|result| {
        if let Err(error) = result {
            tracing::warn!("While downloading image {}: {}", url, error.to_string())
        };
    });

    Ok(vec![])
}
