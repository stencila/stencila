use json5format::{FormatOptions, Json5Format, ParsedDocument};

use codec::common::{
    eyre::Result,
    serde::{de::DeserializeOwned, Serialize},
};

impl<T> Json5Codec for T where T: DeserializeOwned + Serialize {}

pub trait Json5Codec: DeserializeOwned + Serialize {
    /// Decode a Stencila Schema node from JSON5
    fn from_json5(json5: &str) -> Result<Self> {
        Ok(json5::from_str(json5)?)
    }

    /// Encode a Stencila Schema node to compact JSON5
    ///
    /// The `json5format` crate does not have a "compact mode", so this
    /// sets options to get as close to that as possible, and then removes
    /// newlines.
    ///
    /// Spaces after property colons remain. At this time, a PR to `json5format`
    /// would be necessary to add an option to exclude those from output.
    fn to_json5(&self) -> Result<String>
    where
        Self: Sized,
    {
        let json5 = to_json5(
            self,
            FormatOptions {
                indent_by: 0,
                collapse_containers_of_one: true,
                trailing_commas: false,
                ..Default::default()
            },
        )?;

        let compacted = json5.replace('\n', "");

        Ok(compacted)
    }

    /// Encode a Stencila Schema node to indented JSON5
    ///
    /// This overrides `json5format` defaults to make the output
    /// a little less busy (e.g. no trailing commas).
    fn to_json5_pretty(&self) -> Result<String> {
        to_json5(
            self,
            FormatOptions {
                indent_by: 2,
                collapse_containers_of_one: true,
                trailing_commas: false,
                ..Default::default()
            },
        )
    }
}

/// Serialize to JSON5 with options
fn to_json5<T>(value: T, options: FormatOptions) -> Result<String>
where
    T: Serialize,
{
    let json5 = json5::to_string(&value)?;
    let parsed = ParsedDocument::from_str(&json5, None)?;

    let format = Json5Format::with_options(options)?;
    let formatted = format.to_string(&parsed)?;

    Ok(formatted)
}
