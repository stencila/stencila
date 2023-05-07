use std::collections::HashSet;

use common::eyre::Result;
use common::{
    serde::Serialize,
    serde_json::{self, json, Map, Value},
};
/// Converts a value to a JSON object.
///
/// This function takes a value of any serializable type `T` and converts it into a
/// `serde_json::Map<String, serde_json::Value>`, which represents a JSON object.
///
/// # Arguments
///
/// * `value` - The value to convert to a JSON object.
///
/// # Returns
///
/// Returns the JSON object as a map of string keys to `serde_json::Value`.
///
/// # Panics
///
/// This function panics if the conversion from the value to JSON object fails.
fn object<T>(value: &T) -> Map<String, Value>
where
    T: Serialize,
{
    serde_json::from_value(json!(value)).unwrap_or_else(|_| unreachable!())
}
/// Extends an object with additional values.
///
/// This function takes a mutable reference to an object `origin` and extends it with
/// additional values from an iterator `values`. The `origin` object and the values
/// must be serializable. The function performs the following steps:
///
/// 1. Checks for duplicate keys between the `origin` object and the values. If any duplicate
///    keys are found, an error is returned.
/// 2. Merges the values into the `origin` object.
///
/// # Arguments
///
/// * `origin` - The mutable reference to the object to be extended.
/// * `values` - The iterator of values to be added to the object.
///
/// # Returns
///
/// Returns `Result<(), eyre::Error>` indicating the success or failure of the operation.
pub fn extend<T, I>(origin: &mut T, values: I) -> Result<()>
where
    T: Serialize + for<'de> common::serde::Deserialize<'de>,
    I: IntoIterator,
    I::Item: Serialize,
{
    let origin_map = object(origin);
    let value_maps: Vec<(String, Map<String, Value>)> = values
        .into_iter()
        .map(|value| (check_type(&value), object(&value)))
        .collect();

    // 1. Check keys
    let mut origin_keys: HashSet<_> = origin_map.keys().collect();

    let err_output: Vec<String> = value_maps
        .iter()
        .filter_map(|(type_name, value)| {
            let value_keys: HashSet<_> = value.keys().collect();
            let duplicate_keys = origin_keys
                .intersection(&value_keys)
                .collect::<HashSet<_>>();

            let result = if !duplicate_keys.is_empty() {
                // Duplicate keys found:

                Some(format!("{type_name} with {:?}", duplicate_keys))
            } else {
                None
            };
            origin_keys.extend(value_keys);
            result
        })
        .collect();

    if !err_output.is_empty() {
        common::eyre::bail!("Duplicate keys found: {:?}", err_output);
    }

    // 2. Merge values
    let mut origin_map = origin_map;
    origin_map.extend(value_maps.into_iter().flat_map(|(_, m)| m.into_iter()));

    *origin = serde_json::from_value(Value::Object(origin_map))?;
    Ok(())
}
/// Retrieves the type name of a value.
///
/// This function takes a reference to a value `value` and returns the type name as a string.
///
/// # Arguments
///
/// * `value` - The reference to the value.
///
/// # Returns
///
/// Returns the type name of the type
pub fn check_type<T>(_: &T) -> String {
    std::any::type_name::<T>()
        .split("::")
        .last()
        .expect("Invalid type name")
        .to_string()
}
/// ```ignore
/// impl_merge!(Dentist,LocalBusiness,MedicalBusiness,MedicalOrganization)
/// ```
/// generate
/// ```ignore
/// pub trait DentistParent: Serialize {}
/// impl DentistParent for LocalBusiness {}
/// impl DentistParent for MedicalBusiness {}
/// impl DentistParent for MedicalOrganization {}
/// // [LocalBusiness,MedicalBusiness,MedicalOrganization]
/// pub trait Expand: Serialize + for<'de> common::serde::Deserialize<'de> {
///     // type Parent: Serialize;
///     fn extend<I>(origin: &mut Self, values: I) -> common::eyre::Result<()>
///     where
///         I: IntoIterator,
///         I::Item: Serialize + DentistParent,  //<------
///     {
///         crate::extend(origin, values)
///     }
/// }
/// impl Expand for Dentist {}
/// ```
#[macro_export]
macro_rules! impl_merge {
    ($source:ty, $($target:ty),+) => {
        paste! {
            pub trait [<$source Parent>] : Serialize {}
            $(
                impl [<$source Parent>] for $target {}

            )*
            pub trait Expand: Serialize + for<'de> common::serde::Deserialize<'de> {
                fn extend<I>(origin: &mut Self, values: I) -> common::eyre::Result<()>
                where
                    I: IntoIterator,
                    I::Item: Serialize + [<$source Parent>],
                {
                    extend(origin, values)
                }
            }
            impl Expand for $source {}
        }
    };
}
