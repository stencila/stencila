use std::{
    fs::{create_dir_all, write},
    path::PathBuf,
};

use common::eyre::Result;

/// Trait for generating docs for configurations items
pub trait Docs {
    fn docs(&self) -> String {
        String::new()
    }
}

// Trait is implemented as needed

impl Docs for i8 {}
impl Docs for i16 {}
impl Docs for i32 {}
impl Docs for i64 {}
impl Docs for i128 {}

impl Docs for u8 {}
impl Docs for u16 {}
impl Docs for u32 {}
impl Docs for u64 {}
impl Docs for u128 {}

impl Docs for f32 {}
impl Docs for f64 {}

impl Docs for String {}

impl Docs for PathBuf {}

impl<T: Default + Docs> Docs for Option<T> {
    fn docs(&self) -> String {
        T::default().docs()
    }
}

impl<T: Default + Docs> Docs for Vec<T> {
    fn docs(&self) -> String {
        T::default().docs()
    }
}

/// Generate a reference documentation Markdown file
pub fn generate() -> Result<()> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("reference")
        .join("config");
    create_dir_all(&dir)?;

    let path = dir.join("README.md");

    let md = crate::Config::default().docs();

    write(path, md)?;

    Ok(())
}
