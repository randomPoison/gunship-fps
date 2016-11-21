/// Adds a more useable interface over the logging api.
///
/// The main goal of the log!() macro is to provide a println!() -like api that can be used both to
/// quickly.
macro_rules! log {
    ($text:expr) => { println!($text); };
    ($text:expr, $($arg:expr),*) => { println!($text $(, $arg)*); };
}
