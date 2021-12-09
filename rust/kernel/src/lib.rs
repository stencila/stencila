// use crate::utils::uuids;
use async_trait::async_trait;
use eyre::Result;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use stencila_schema::{CodeError, Node};
use strum::Display;

// Re-export for the convenience of crates that implement `KernelTrait`
pub use ::async_trait;
pub use eyre;
pub use serde;
pub use stencila_schema;

/// The type of kernel
///
/// At present this is mainly for informational purposes.
#[derive(Debug, Clone, Serialize, Deserialize, Display)]
pub enum KernelType {
    Builtin,
    Micro,
    Jupyter,
}

/// A specification for kernels
///
/// All kernels, including those implemented in plugins, should provide this
/// specification. Rust implementations return a `Kernel` instance from the
/// `spec` function of `KernelTrait`. Plugins provide a JSON or YAML serialization
/// as part of their manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Kernel {
    /// The name of the kernel
    ///
    /// This is used for informational purposes and to allow the user to specify
    /// which kernel they want to use (e.g. in instances that they have more than one kernel that
    /// is capable of executing a language).
    pub name: String,

    /// The type of kernel
    pub r#type: KernelType,

    /// The languages supported by the kernel
    ///
    /// These should be the `name` of one of the `Format`s defined in
    /// the `formats` crate. Many kernels only support one language.
    pub languages: Vec<String>,
}

impl Kernel {
    // Create a new kernel specification
    pub fn new(name: &str, r#type: KernelType, languages: &[&str]) -> Self {
        let languages = languages
            .iter()
            .map(|language| language.to_string())
            .collect();
        Self {
            name: name.to_string(),
            r#type,
            languages,
        }
    }

    // Does the kernel specification match against a kernel selector string?
    pub fn matches(&self, selector: &str) -> bool {
        KernelSelector::parse(selector).matches(self)
    }
}

/// The status of a running kernel
#[derive(Debug, PartialEq, Clone, Serialize, Display)]
#[allow(dead_code)]
pub enum KernelStatus {
    Pending,
    Starting,
    Idle,
    Busy,
    Unresponsive,
    Stopping,
    Finished,
    Failed,
    Unknown,
}

/// A selector used to choose amongst alternative kernels
pub struct KernelSelector {
    /// A string that will match against the kernel `name` or any of its `languages`
    pub any: Option<String>,

    /// A string that will match against the kernel `name`
    pub name: Option<String>,

    /// A string that will match against any of a kernel's `languages`
    pub lang: Option<String>,

    /// A string that will match against the kernel `type`
    pub r#type: Option<String>,
}

impl fmt::Display for KernelSelector {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let mut str = self.any.clone().unwrap_or_default();
        if let Some(name) = &self.name {
            str.push_str(" name:");
            str.push_str(name);
        }
        if let Some(lang) = &self.lang {
            str.push_str(" lang:");
            str.push_str(lang);
        }
        if let Some(r#type) = &self.r#type {
            str.push_str(" type:");
            str.push_str(r#type);
        }
        write!(formatter, "{}", str.trim())
    }
}

impl KernelSelector {
    /// Create a new `KernelSelector`
    pub fn new(name: Option<String>, lang: Option<String>, r#type: Option<String>) -> Self {
        Self {
            any: None,
            name,
            lang,
            r#type,
        }
    }

    /// Parse a kernel selector string into a `KernelSelector`
    pub fn parse(selector: &str) -> Self {
        static REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"(\b(name|lang|type)\s*:\s*(\w+)\b)|(\w+)").expect("Unable to create regex")
        });

        let mut any = None;
        let mut name = None;
        let mut lang = None;
        let mut r#type = None;
        for captures in REGEX.captures_iter(selector) {
            if let Some(tag) = captures.get(2) {
                let value = Some(captures[3].to_string());
                match tag.as_str() {
                    "name" => {
                        if name.is_none() {
                            name = value
                        } else {
                            tracing::warn!("Ignoring additional kernel `name` selector");
                        }
                    }
                    "lang" => {
                        if lang.is_none() {
                            lang = value
                        } else {
                            tracing::warn!("Ignoring additional kernel `lang` selector");
                        }
                    }
                    "type" => {
                        if r#type.is_none() {
                            r#type = value
                        } else {
                            tracing::warn!("Ignoring additional kernel `type` selector");
                        }
                    }
                    _ => (),
                }
            } else if any.is_none() {
                any = Some(captures[4].to_string())
            } else {
                tracing::warn!(
                    "Ignoring extraneous kernel selector: {}",
                    captures[0].to_string()
                );
            }
        }

        Self {
            any,
            name,
            lang,
            r#type,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.any.is_none() && self.name.is_none() && self.lang.is_none() && self.r#type.is_none()
    }

    /// Does a `Kernel` match this selector
    pub fn matches(&self, kernel: &Kernel) -> bool {
        let mut matched = true;

        if let (true, Some(name)) = (matched, &self.name) {
            matched = name.to_lowercase() == kernel.name.to_lowercase();
        } else if let Some(name) = &self.any {
            matched = name.to_lowercase() == kernel.name.to_lowercase();
        }

        if let (true, Some(lang)) = (matched, &self.lang) {
            let mut lang_matched = false;
            for kernel_lang in &kernel.languages {
                if lang.to_lowercase() == kernel_lang.to_lowercase() {
                    lang_matched = true;
                    break;
                }
            }
            matched &= lang_matched;
        } else if let (false, Some(lang)) = (matched, &self.any) {
            for kernel_lang in &kernel.languages {
                if lang.to_lowercase() == kernel_lang.to_lowercase() {
                    matched = true;
                    break;
                }
            }
        }

        if let (true, Some(r#type)) = (matched, &self.r#type) {
            matched = r#type.to_lowercase() == kernel.r#type.to_string().to_lowercase();
        }

        matched
    }
}

/// A trait for kernels
///
/// This trait can be used by Rust implementations of kernels, allowing them to
/// be compiled into the Stencila binaries.
#[async_trait]
pub trait KernelTrait {
    /// Get the [`Kernel`] specification for this implementation
    fn spec(&self) -> Kernel;

    /// Is the kernel available on the current machine?
    async fn available(&self) -> bool {
        true
    }

    /// Start the kernel
    async fn start(&mut self) -> Result<()> {
        Ok(())
    }

    /// Stop the kernel
    async fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    /// Get the status of the kernel
    async fn status(&self) -> Result<KernelStatus>;

    /// Get a symbol from the kernel
    async fn get(&mut self, name: &str) -> Result<Node>;

    /// Set a symbol in the kernel
    async fn set(&mut self, name: &str, value: Node) -> Result<()>;

    /// Execute some code in the kernel
    async fn exec(&mut self, code: &str) -> Result<(Vec<Node>, Vec<CodeError>)>;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn kernel_selector_new() {
        let ks = KernelSelector::parse("name_lang");
        assert_eq!(ks.any, Some("name_lang".to_string()));
        assert_eq!(ks.name, None);
        assert_eq!(ks.lang, None);
        assert_eq!(ks.r#type, None);
        assert_eq!(ks.to_string(), "name_lang");

        let ks = KernelSelector::parse("name_lang foo bar");
        assert_eq!(ks.any, Some("name_lang".to_string()));
        assert_eq!(ks.name, None);
        assert_eq!(ks.lang, None);
        assert_eq!(ks.r#type, None);

        let ks = KernelSelector::parse("type:micro name_lang");
        assert_eq!(ks.any, Some("name_lang".to_string()));
        assert_eq!(ks.name, None);
        assert_eq!(ks.lang, None);
        assert_eq!(ks.r#type, Some("micro".to_string()));

        let ks = KernelSelector::parse("name_lang type:jupyter lang:py");
        assert_eq!(ks.any, Some("name_lang".to_string()));
        assert_eq!(ks.name, None);
        assert_eq!(ks.lang, Some("py".to_string()));
        assert_eq!(ks.r#type, Some("jupyter".to_string()));

        let ks = KernelSelector::parse("name:ir");
        assert_eq!(ks.any, None);
        assert_eq!(ks.name, Some("ir".to_string()));
        assert_eq!(ks.lang, None);
        assert_eq!(ks.r#type, None);

        let ks = KernelSelector::parse("type:jupyter lang:r name:ir");
        assert_eq!(ks.any, None);
        assert_eq!(ks.name, Some("ir".to_string()));
        assert_eq!(ks.lang, Some("r".to_string()));
        assert_eq!(ks.r#type, Some("jupyter".to_string()));
        assert_eq!(ks.to_string(), "name:ir lang:r type:jupyter");
    }

    #[test]
    fn kernel_selector_matches() {
        let k = Kernel::new("foo", KernelType::Builtin, &["bar", "baz"]);

        assert!(KernelSelector::parse("foo").matches(&k));
        assert!(KernelSelector::parse("bar").matches(&k));
        assert!(KernelSelector::parse("baz").matches(&k));
        assert!(KernelSelector::parse("name:foo").matches(&k));
        assert!(KernelSelector::parse("lang:bar").matches(&k));
        assert!(KernelSelector::parse("lang:baz").matches(&k));
        assert!(KernelSelector::parse("name:foo lang:bar type:builtin").matches(&k));
        assert!(KernelSelector::parse("foo type:builtin").matches(&k));

        assert!(!KernelSelector::parse("quax").matches(&k));
        assert!(!KernelSelector::parse("name:quax").matches(&k));
        assert!(!KernelSelector::parse("lang:quax").matches(&k));
        assert!(!KernelSelector::parse("name:foo lang:quax").matches(&k));
        assert!(!KernelSelector::parse("name:foo lang:bar type:quax").matches(&k));
        assert!(!KernelSelector::parse("foo type:quax").matches(&k));
    }
}
