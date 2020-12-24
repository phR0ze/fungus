mod file;
mod os;
mod path;

// Export contents of modules into sys
pub use file::*;
pub use os::*;
pub use path::*;

// Export modules directly
pub mod env;
pub mod exec;
pub mod user;
pub mod term;

// Export extensions
pub mod ext {
    pub use super::path::PathColorExt;
    pub use super::path::PathExt;
}
