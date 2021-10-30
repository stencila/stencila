//! A codec for date(time)s

use chrono::{DateTime, Local, TimeZone, Utc};
use codec_trait::{
    eyre::{bail, Result},
    stencila_schema::{Date, Node},
    Codec,
};
use dtparse::parse;

pub struct DateCodec {}

impl Codec for DateCodec {
    fn from_str(str: &str) -> Result<Node> {
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
            bail!("Unable to decode as a `Date`: {}", str)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::assert_debug_eq;

    #[test]
    fn from_str() -> Result<()> {
        // Adding timezone avoids failures due to different local time on
        // test host machines.
        assert_debug_eq(
            DateCodec::from_str("11 Jul 2021; 12 am +00:00")?,
            Node::Date(Date {
                value: "2021-07-11T00:00:00+00:00".to_string(),
                ..Default::default()
            }),
        );
        assert_debug_eq(
            DateCodec::from_str("July 11 2021; 6 am +06:00")?,
            Node::Date(Date {
                value: "2021-07-11T00:00:00+00:00".to_string(),
                ..Default::default()
            }),
        );
        assert_debug_eq(
            DateCodec::from_str("2021-07-11; 13:00 +13:00")?,
            Node::Date(Date {
                value: "2021-07-11T00:00:00+00:00".to_string(),
                ..Default::default()
            }),
        );
        Ok(())
    }
}
