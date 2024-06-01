use std::ops::Range;

use common::serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::prelude::*;

#[derive(Debug, Default, Clone, Deref, DerefMut, Serialize)]
#[serde(crate = "common::serde")]
pub struct Cord {
    /// The string value of the cord
    #[deref]
    #[deref_mut]
    pub string: String,

    /// The runs of authorship in the cord
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub authorship: Vec<CordAuthorship>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CordAuthorship {
    /// A count of all authors of the run
    pub count: u8,

    /// The last eight authors, each as a `u8`, encoded within a `u64` with the most
    /// recent author at the least significant digit.
    ///
    /// The `u8` for each author is the index of the author in the closest ancestor node
    /// that has an `authors` property. A value of u8::MAX indicates an unknown author.
    pub authors: u64,

    /// The provenance byte
    pub provenance: u8,

    /// The number of characters (Unicode code points) in the run
    pub length: u32,
}

impl CordAuthorship {
    pub fn new(count: u8, authors: u64, provenance: u8, length: u32) -> Self {
        Self {
            count,
            authors,
            provenance,
            length,
        }
    }

    pub fn from_tuple((count, authors, provenance, length): (u8, u64, u8, u32)) -> Self {
        Self::new(count, authors, provenance, length)
    }

    pub fn as_tuple(&self) -> (u8, u64, u8, u32) {
        (self.count, self.authors, self.provenance, self.length)
    }
}

impl PartialEq for Cord {
    fn eq(&self, other: &Self) -> bool {
        // Ignore authorship for equality
        self.string == other.string
    }
}

impl<S> From<S> for Cord
where
    S: AsRef<str>,
{
    fn from(value: S) -> Self {
        Self {
            string: value.as_ref().to_string(),
            authorship: Vec::new(),
        }
    }
}

impl From<Cord> for String {
    fn from(cord: Cord) -> Self {
        cord.string.clone()
    }
}

impl<'de> Deserialize<'de> for Cord {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(crate = "common::serde")]
        struct Map {
            string: String,
            #[serde(default)]
            authorship: Vec<CordAuthorship>,
        }

        #[derive(Deserialize)]
        #[serde(untagged, crate = "common::serde")]
        enum StringOrMap {
            String(String),
            Map(Map),
        }

        let cord = match StringOrMap::deserialize(deserializer)? {
            StringOrMap::String(string) => Cord {
                string,
                ..Default::default()
            },
            StringOrMap::Map(map) => Cord {
                string: map.string,
                authorship: map.authorship,
            },
        };

        Ok(cord)
    }
}

impl Serialize for CordAuthorship {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_tuple().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for CordAuthorship {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        type Tuple = (u8, u64, u8, u32);
        let tuple = Tuple::deserialize(deserializer)?;
        Ok(CordAuthorship::from_tuple(tuple))
    }
}

// An operation on a `Cord`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub enum CordOp {
    Insert(usize, String),
    Delete(Range<usize>),
    Replace(Range<usize>, String),
}
