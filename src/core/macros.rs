/// Provides the ability to define `#[cfg]` statements for multiple items
///
/// ### Examples
/// ```ignore
/// use fungus::prelude::*;
///
/// cfgblk! {
///     #[cfg(feature = "foo")])
///     use libc;
///     use std::ffi::CString;
/// }
///
/// // Expands to
/// #[cfg(feature = "foo")])
/// use libc;
/// #[cfg(feature = "foo")])
/// use std::ffi::CString;
/// ```
#[macro_export]
macro_rules! cfgblock {

    // Handle a single item
    (#[$attr:meta] $item:item) => {
        #[$attr] $item
    };

    // Handle more than one item recursively
    (#[$attr:meta] $($tail:item)*) => {
        $(cfgblock!{#[$attr] $tail})*
    };
}

/// Provides a fatal log function for logging at error level then terminating the process.
#[macro_export]
macro_rules! fatal {
    (target: $target:expr, $($arg:tt)+) => (
        $crate::core::Logger::fatal(
            log::__log_format_args!($($arg)+),
            log::Level::Error,
            &($target, log::__log_module_path!(), log::__log_file!(), log::__log_line!()),
        );
    );
    ($($arg:tt)+) => (
        fatal!(target: log::__log_module_path!(), $($arg)+);
    )
}

// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    // use crate::prelude::*;

    #[test]
    fn test_macros() {
        fatal!("foobar!");
    }
}
