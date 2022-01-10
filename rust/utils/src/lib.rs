/// Concisely create a `Some::<String>`
#[macro_export]
macro_rules! some_string {
    ($x:expr) => {
        Some($x.to_string())
    };
}

/// Concisely create a `Some::<Box<String>>`
#[macro_export]
macro_rules! some_box_string {
    ($x:expr) => {
        Some(Box::new($x.to_string()))
    };
}

/// Concisely create a `Vec<String>`
#[macro_export]
macro_rules! vec_string {
    ($($x:expr),*) => (vec![$($x.to_string()),*]);
}
