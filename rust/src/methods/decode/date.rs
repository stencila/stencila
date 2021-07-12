use chrono::{DateTime, Local, TimeZone, Utc};
use eyre::Result;
use stencila_schema::{Date, Node};

/// Decode a string into a `Node::Date` variant.
///
/// Always returns an `Ok` result with a `Node::Date` value
/// but its `value` may be `None`.
pub fn decode(input: &str) -> Result<Node> {
    Ok(Node::Date(decode_date(input)))
}

/// Decode a string to a `Date` struct.
///
/// If the string input could not be parsed as a date, will
/// return a date with an empty `value` string.
pub fn decode_date(input: &str) -> Date {
    if let Some(date) = decode_date_maybe(input) {
        date
    } else {
        Date {
            ..Default::default()
        }
    }
}

/// Attempt to decode a string to a `Date` struct.
///
/// Returns `Some(Date)` if parsing was successful, `None` otherwise.
pub fn decode_date_maybe(input: &str) -> Option<Date> {
    if let Ok((naive, offset)) = dtparse::parse(input) {
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
        Some(Date {
            value: utc.to_rfc3339(),
            ..Default::default()
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decoding() {
        // Adding timezone avoids failures due to different local time on 
        // test host machines.
        assert_eq!(&decode_date("11 Jul 2021; 12 am +00:00").value, "2021-07-11T00:00:00+00:00");
        assert_eq!(&decode_date("July 11 2021; 6 am +06:00").value, "2021-07-11T00:00:00+00:00");
        assert_eq!(&decode_date("2021-07-11; 13:00 +13:00").value, "2021-07-11T00:00:00+00:00");
    }
}
