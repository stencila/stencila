use serde::{Serialize, de::DeserializeOwned};

use codec::eyre::Result;

impl<T> CborCodec for T where T: DeserializeOwned + Serialize {}

pub trait CborCodec: DeserializeOwned + Serialize {
    /// Decode a Stencila Schema node from CBOR
    fn from_cbor(cbor: &[u8]) -> Result<Self> {
        Ok(ciborium::from_reader(cbor)?)
    }

    /// Encode a Stencila Schema node to CBOR
    fn to_cbor(&self) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        ciborium::into_writer(self, &mut bytes)?;
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A test that the CBOR crate we are using behaves as expected
    #[test]
    fn roundtrip() -> Result<()> {
        use serde::Deserialize;

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct Struct1 {
            a: bool,
            b: u8,
            c: f32,
        }

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct Struct2 {
            a: String,
            b: Vec<usize>,
            c: Vec<Struct1>,
        }

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(tag = "type")]
        enum Enum {
            Struct1(Struct1),
            Struct2(Struct2),
        }

        let a = Struct1 {
            a: true,
            b: 1,
            c: 1.23,
        };
        let b = Struct1::from_cbor(&a.to_cbor()?)?;
        assert_eq!(a, b);

        let a = Enum::Struct1(a);
        let b = Enum::from_cbor(&a.to_cbor()?)?;
        assert_eq!(a, b);

        let a = Struct2 {
            a: "Hello world".to_string(),
            b: vec![1, 2, 3],
            c: vec![Struct1 {
                a: true,
                b: 2,
                c: 3.45,
            }],
        };
        let b = Struct2::from_cbor(&a.to_cbor()?)?;
        assert_eq!(a, b);

        let a = Enum::Struct2(a);
        let b = Enum::from_cbor(&a.to_cbor()?)?;
        assert_eq!(a, b);

        Ok(())
    }
}
