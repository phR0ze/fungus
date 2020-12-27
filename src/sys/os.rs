use crate::errors::*;
use crate::sys;
use std::io;

/// Type of operating system rust is running on
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Arch {
    X86,    // 32bit
    X86_64, // 64bit
}

/// Detect at runtime the system architecture rust is running on.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert_eq!(sys::arch(), sys::Arch::X86);
/// ```

#[cfg(target_arch = "x86")]
pub fn arch() -> Arch {
    Arch::X86
}

/// Detect at runtime the system architecture rust is running on.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert_eq!(sys::arch(), sys::Arch::X86_64);
/// ```
#[cfg(target_arch = "x86_64")]
pub fn arch() -> Arch {
    Arch::X86_64
}

/// Returns true if the system is a x86 system.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert_eq!(sys::x86(), false);
/// ```
pub fn x86() -> bool {
    arch() == Arch::X86
}

/// Returns true if the system is a x86_64 system.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert_eq!(sys::x86_64(), true);
/// ```
pub fn x86_64() -> bool {
    arch() == Arch::X86_64
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

/// Type of operating system rust is running on
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Info {
    pub arch: Arch,      // System architecture
    pub kernel: String,  // Kernel version e.g. 5.3.13
    pub release: String, // Kernel release e.g. 5.3.13-arch1-1
}

/// Get system information
pub fn info() -> FuResult<Info> {
    // Extract kernel release and version
    let data = sys::readstring("/proc/version")?;
    let release = data.split(' ').nth(2).ok_or(OsError::KernelReleaseNotFound)?;
    let ver_len = release.find('-').ok_or(OsError::KernelVersionNotFound)?;
    let (version, _) = release.split_at(ver_len);

    Ok(Info { arch: arch(), kernel: version.to_string(), release: release.to_string() })
}

// Substitute stdout and stderr
pub struct Stdio<T: io::Write, U: io::Write> {
    pub out: T,
    pub err: U,
}
impl<T: io::Write, U: io::Write> Stdio<T, U> {
    pub fn new(out: T, err: U) -> Self {
        Stdio { out, err }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use std::io::{self, Write};

    #[test]
    fn test_info() {
        assert!(sys::info().is_ok());
    }

    #[test]
    fn test_arch() {
        sys::x86();
        sys::x86_64();
    }

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
