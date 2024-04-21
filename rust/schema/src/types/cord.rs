use std::{fmt, ops::Range};

use common::serde::{
    de::{self, MapAccess, Visitor as SerdeVisitor},
    ser::SerializeStruct,
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::prelude::*;

#[derive(Debug, Default, Clone, Deref, DerefMut)]
pub struct Cord {
    /// The string value of the cord
    #[deref]
    #[deref_mut]
    pub(crate) string: String,

    /// The authorship of the current value of the cord
    ///
    /// Run length encoding of which authors created which UTF-8 bytes
    /// in the value. A value of 0 indicates an anonymous author. A value
    /// of >0 corresponds to the author in the closest ancestor node
    /// with an `authors` property.
    pub authorship: Vec<(u16, usize)>,
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

impl Serialize for Cord {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self.authorship.is_empty() {
            // Serialize just the string if authorship is empty
            serializer.serialize_str(&self.string)
        } else {
            // Otherwise, serialize as an object with both fields
            let mut state = serializer.serialize_struct("Cord", 2)?;
            state.serialize_field("string", &self.string)?;
            state.serialize_field("authorship", &self.authorship)?;
            state.end()
        }
    }
}

struct CordVisitor;

impl<'de> SerdeVisitor<'de> for CordVisitor {
    type Value = Cord;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string or map with a string and authorship")
    }

    // Deserialize Cord from a simple string
    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        Ok(Cord {
            string: value.to_owned(),
            authorship: Vec::new(),
        })
    }

    // Deserialize Cord from a map
    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
        let mut string = None;
        let mut authorship = None;

        while let Some(key) = map.next_key()? {
            match key {
                "string" => {
                    if string.is_some() {
                        return Err(de::Error::duplicate_field("string"));
                    }
                    string = Some(map.next_value()?);
                }
                "authorship" => {
                    if authorship.is_some() {
                        return Err(de::Error::duplicate_field("authorship"));
                    }
                    authorship = Some(map.next_value()?);
                }
                _ => return Err(de::Error::unknown_field(key, &["string", "authorship"])),
            }
        }

        let string = string.ok_or_else(|| de::Error::missing_field("string"))?;
        let authorship = authorship.unwrap_or_default();

        Ok(Cord { string, authorship })
    }
}

impl<'de> Deserialize<'de> for Cord {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(CordVisitor)
    }
}

// An operation on a `Cord`
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(crate = "common::serde")]
pub enum CordOp {
    Insert(usize, String),
    Delete(Range<usize>),
    Replace(Range<usize>, String),
}
