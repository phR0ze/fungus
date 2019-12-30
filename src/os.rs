use std::io;

// Substitute stdout and stderr for testing
pub struct Stdio<T: io::Write, U: io::Write> {
    pub out: T,
    pub err: U,
}
impl<T: io::Write, U: io::Write> Stdio<T, U> {
    pub fn new(out: T, err: U) -> Self {
        Stdio { out, err }
    }
}

/// Type of operating system rust is running on
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Platform {
    Linux,
    MacOS,
    Windows,
}

/// Detect at runtime the type of operating system we are running.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert_eq!(sys::platform(), sys::Platform::Linux);
/// ```
#[cfg(target_os = "linux")]
pub fn platform() -> Platform {
    Platform::Linux
}

/// Detect at runtime the type of operating system we are running.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert_eq!(sys::platform(), sys::Platform::MacOS);
/// ```
#[cfg(target_os = "macos")]
pub fn platform() -> Platform {
    Platform::MacOS
}

/// Detect at runtime the type of operating system we are running.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert_eq!(sys::platform(), sys::Platform::Windows);
/// ```
#[cfg(target_os = "windows")]
pub fn platform() -> Platform {
    Platform::Windows
}

/// True if rust is running on linux
pub fn linux() -> bool {
    platform() == Platform::Linux
}

/// True if rust is running on macos
pub fn macos() -> bool {
    platform() == Platform::MacOS
}

/// True if rust is running on windows
pub fn windows() -> bool {
    platform() == Platform::Windows
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use std::io::{self, Write};

    #[test]
    #[cfg(target_os = "linux")]
    fn test_linux() {
        assert_eq!(sys::linux(), true);
        assert_eq!(sys::macos(), false);
        assert_eq!(sys::windows(), false);
    }

    #[test]
    fn test_stdio() {
        // real
        let mut stdio = sys::Stdio::new(io::stdout(), io::stderr());
        writeln!(stdio.out, "Hello out").unwrap();
        writeln!(stdio.err, "Hello err").unwrap();

        // buffer
        let mut stdio = sys::Stdio::new(Vec::new(), Vec::new());
        writeln!(stdio.out, "Hello out").unwrap();
        writeln!(stdio.err, "Hello err").unwrap();
        assert_eq!(stdio.out, b"Hello out\n");
        assert_eq!(stdio.err, b"Hello err\n");
    }
}
