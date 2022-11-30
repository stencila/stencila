/// Assert that two nodes are equal based on their JSON representation
///
/// This has the advantage over `pretty_assertions::assert_eq` of not requiring the
/// `==` operator to be defined for the types and hiding less usually irrelevant
/// details such as `Box` wrappers.
#[macro_export]
macro_rules! assert_json_eq {
    ($a:expr, $b:expr) => {
        test_utils::pretty_assertions::assert_eq!(
            test_utils::common::serde_json::to_value(&$a).unwrap(),
            test_utils::common::serde_json::to_value(&$b).unwrap()
        );
    };
}

/// Assert that two nodes are equal based on their JSON representation
///
/// This is a convenience macro to avoid having to import and use `json!`
/// on the second argument of `assert_json_eq`.
#[macro_export]
macro_rules! assert_json_is {
    ($a:expr, $b:tt) => {
        test_utils::pretty_assertions::assert_eq!(
            test_utils::common::serde_json::json!($a),
            test_utils::common::serde_json::json!($b)
        );
    };
}
