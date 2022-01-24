use chrono::{DateTime, Local, TimeZone, Utc};
use codec::{
    eyre::{bail, Result},
    stencila_schema::{Date, Node},
    utils::vec_string,
    Codec, CodecTrait, DecodeOptions, EncodeOptions,
};
use dtparse::parse;

// A codec for date/time strings
pub struct DateCodec {}

impl CodecTrait for DateCodec {
    fn spec() -> Codec {
        Codec {
            status: "beta".to_string(),
            formats: vec_string!["date"],
            root_types: vec_string!["Date"],
            ..Default::default()
        }
    }

    fn from_str(str: &str, _options: Option<DecodeOptions>) -> Result<Node> {
        if let Ok((naive, offset)) = parse(str) {
            let utc = match offset {
                Some(offset) => {
                    let tz = offset.from_local_datetime(&naive).unwrap();
                    DateTime::<Utc>::from(tz)
                }
                None => {
                    let local = Local.from_local_datetime(&naive).unwrap();
                    DateTime::<Utc>::from(local)
                }
            };
            Ok(Node::Date(Date {
                value: utc.to_rfc3339(),
                ..Default::default()
            }))
        } else {
            bail!("Unable to decode string to a `Date`: {}", str)
        }
    }

    fn to_string(node: &Node, _options: Option<EncodeOptions>) -> Result<String> {
        match node {
            Node::Date(date) => Ok(date.value.clone()),
            _ => bail!("Unable to encode node that is not a `Date`"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::assert_json_eq;

    #[test]
    fn from_str() -> Result<()> {
        // Adding timezone avoids failures due to different local time on
        // test host machines.
        assert_json_eq!(
            DateCodec::from_str("11 Jul 2021; 12 am +00:00", None)?,
            Node::Date(Date {
                value: "2021-07-11T00:00:00+00:00".to_string(),
                ..Default::default()
            })
        );
        assert_json_eq!(
            DateCodec::from_str("July 11 2021; 6 am +06:00", None)?,
            Node::Date(Date {
                value: "2021-07-11T00:00:00+00:00".to_string(),
                ..Default::default()
            })
        );
        assert_json_eq!(
            DateCodec::from_str("2021-07-11; 13:00 +13:00", None)?,
            Node::Date(Date {
                value: "2021-07-11T00:00:00+00:00".to_string(),
                ..Default::default()
            })
        );
        Ok(())
    }

    #[test]
    fn to_string() -> Result<()> {
        assert_eq!(
            DateCodec::to_string(
                &Node::Date(Date {
                    value: "2021-07-11T00:00:00+00:00".to_string(),
                    ..Default::default()
                }),
                None
            )?,
            "2021-07-11T00:00:00+00:00"
        );
        assert_eq!(
            DateCodec::to_string(
                &Node::Date(Date {
                    value: "2021-07-11".to_string(),
                    ..Default::default()
                }),
                None
            )?,
            "2021-07-11"
        );
        Ok(())
    }
}
