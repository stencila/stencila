use std::path::{Path, PathBuf};

use merge::Merge;

use common::{
    defaults::Defaults,
    serde::{Deserialize, Serialize},
    serde_with::skip_serializing_none,
    strum::AsRefStr,
};
use config_docs::Docs;

use crate::docs::Docs;

#[derive(Debug, Defaults, Merge, Deserialize, Serialize, Docs)]
#[serde(crate = "common::serde")]
#[skip_serializing_none]
pub struct Config {
    /// Options for building a project or directory
    pub build: Build,
}

#[derive(Debug, Defaults, Merge, Deserialize, Serialize, Docs)]
#[serde(crate = "common::serde")]
#[skip_serializing_none]
pub struct Build {
    /// Options for building a web site
    ///
    /// Set to `null` to not build a site.
    #[def = "Some(Site::default())"]
    pub site: Option<Site>,
}

/// Configuration options for the site
#[derive(Debug, Defaults, Merge, Deserialize, Serialize, Docs)]
#[serde(crate = "common::serde")]
#[skip_serializing_none]
pub struct Site {
    /// The pages to generate for the current directory
    ///
    /// Use this option to...
    ///
    /// Defaults to using all files in a directory in file/directory name order.
    /// Override this by providing a list of pages (and optionally their labels etc)
    /// in the desired order.
    #[def = "None"]
    pub pages: Option<Vec<Page>>,

    /// A list of file name patterns to use as the "index" page for the current directory
    ///
    /// Patterns use `glob` syntax and are case insensitive.
    /// Defaults to `["index.*", "main.*", "readme.*"]`.
    /// Set to `null` to not generate an `index.html` file for a directory.
    pub index: Option<Vec<String>>,

    /// A list of file/directory name patterns to include
    ///
    /// Ignored if `pages` is not empty
    pub include: Option<Vec<String>>,

    /// A list of file/directory name patterns to exclude
    ///
    /// Ignored if `pages` is not empty
    pub exclude: Option<Vec<String>>,

    /// The casing to use to transform file and directory names into breadcrumb labels
    ///
    /// Set to `null` to not perform any case transformation.
    #[def = "Some(Case::Title)"]
    pub label_case: Option<Case>,

    /// Options for site navigation breadcrumbs
    ///
    /// By default, breadcrumbs are shown in the toolbar of each page except those in the
    /// root directory of the site.
    /// Set to `null` to never show breadcrumbs.
    #[def = "Some(Breadcrumbs::default())"]
    pub breadcrumbs: Option<Breadcrumbs>,

    /// Options for site image optimizations
    ///
    /// By default, images in documents, including images that are the output of code
    /// chunks are optimized.
    /// Set to `null` to never perform image optimization.
    #[def = "Some(Images::default())"]
    pub images: Option<Images>,
}

#[derive(Debug, Defaults, Deserialize, Serialize, Docs)]
#[serde(crate = "common::serde", rename_all = "camelCase")]
#[skip_serializing_none]

pub struct Page {
    pub source: PathBuf,

    pub slug: Option<String>,

    pub label: Option<String>,
}

/// Alternative case transformations
#[derive(Debug, Defaults, Deserialize, Serialize, Docs)]
#[def = "Title"]
#[serde(crate = "common::serde", rename_all = "camelCase")]
pub enum Case {
    /// Camel case e.g. `camelCase`
    #[serde(alias = "Camel", alias = "camelCase")]
    Camel,

    /// Kebab case e.g. `kebab-case`
    #[serde(alias = "Kebab", alias = "kebab-case")]
    Kebab,

    /// Pascal case e.g. `PascalCase`
    #[serde(alias = "Pascal", alias = "PascalCase")]
    Pascal,

    /// Screaming snake case e.g. `SCREAMING_SNAKE_CASE`
    #[serde(
        alias = "ScreamingSnake",
        alias = "SCREAMING_SNAKE",
        alias = "SCREAMING_SNAKE_CASE"
    )]
    ScreamingSnake,

    /// Sentence case e.g. `Sentence case`
    #[serde(alias = "Sentence", alias = "Sentence case")]
    Sentence,

    /// Snake case e.g. `snake_case`
    #[serde(alias = "Snake", alias = "snake_case")]
    Snake,

    /// Title case e.g. `Title Case`
    #[serde(alias = "Title", alias = "Title Case")]
    Title,

    /// Train case e.g. `Train-Case`
    #[serde(alias = "Train", alias = "Train-Case")]
    Train,
}

/// Configuration options for site navigation breadcrumbs shown in the toolbar of each page
#[derive(Debug, Defaults, Merge, Deserialize, Serialize, Docs)]
#[serde(crate = "common::serde")]
#[skip_serializing_none]
pub struct Breadcrumbs {
    /// The label for the first breadcrumb pointing to the root directory of the site
    ///
    /// Set to `null` for no label.
    #[def = "None"]
    pub root_label: Option<String>,

    /// The icon for the first breadcrumb pointing to the root directory of the site
    ///
    /// Set to `null` for no icon.
    #[def = "Some(\"home\".to_string())"]
    pub root_icon: Option<String>,

    /// The separator between each breadcrumb
    ///
    /// Usually a character such as `/` but may be an emoji or
    /// a multi-character string.
    #[def = "\">\".to_string()"]
    #[merge(strategy = replace)]
    pub separator: String,
}

#[derive(Debug, Defaults, Merge, Deserialize, Serialize, Docs)]
#[serde(crate = "common::serde")]
#[skip_serializing_none]
pub struct Images {
    /// A list of image widths to generate for each of the formats
    ///
    /// Set to `null` not only generate images of the same size as
    /// the original.
    #[def = "Some(vec![640, 750, 828, 1080, 1200, 1920, 2048, 3840])"]
    pub sizes: Option<Vec<u32>>,

    /// The quality of optimized images
    ///
    /// An integer between 1 and 100 where 100 is the best quality and
    /// thus has the largest file size.
    #[def = "75"]
    #[merge(strategy = replace)]
    pub quality: u8,

    /// A list of formats to generate images for
    ///
    /// Set to `null` to only generate images in the original format
    /// of the source image.
    ///
    /// Note: support for AVIF is alpha and may not be enabled
    /// in the version of Stencila you are using.
    #[def = "Some(vec![ImageFormat::WebP])"]
    pub formats: Option<Vec<ImageFormat>>,
}

/// Alternative image formats
#[derive(Debug, Defaults, Clone, Deserialize, Serialize, AsRefStr, Docs)]
#[def = "Avif"]
#[serde(crate = "common::serde", rename_all = "camelCase")]
pub enum ImageFormat {
    #[serde(alias = "avif", alias = "image/avif", alias = "AVIF")]
    Avif,
    #[serde(alias = "webp", alias = "image/webp", alias = "WebP")]
    WebP,
}

impl Config {
    /// Resolve a config for a given path
    #[allow(dead_code)]
    fn resolve<P: AsRef<Path>>(_path: &Path) {
        todo!()
    }
}

/// A merge strategy which simply replaces the existing value
pub fn replace<T>(left: &mut T, right: T) {
    *left = right;
}
