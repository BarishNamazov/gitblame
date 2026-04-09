/// Shared error type alias used throughout the gitblame crate.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
