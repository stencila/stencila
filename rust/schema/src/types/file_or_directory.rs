//! Generated file, do not edit

use crate::prelude::*;

use super::directory::Directory;
use super::file::File;

/// [`File`] or [`Directory`]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]

pub enum FileOrDirectory {
    File(File),
    Directory(Directory),
}
