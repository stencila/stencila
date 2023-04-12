use crate::prelude::*;

use super::directory::Directory;
use super::file::File;

/// [`File`] or [`Directory`]
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Read, Write)]
#[serde(untagged, crate = "common::serde")]

pub enum FileOrDirectory {
    File(File),
    Directory(Directory),
}
