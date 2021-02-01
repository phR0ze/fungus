use crate::core::StringExt;

pub const KIBIBYTE: u64 = 1024;
pub const MEBIBYTE: u64 = KIBIBYTE * 1024;
pub const GIBIBYTE: u64 = MEBIBYTE * 1024;
pub const TEBIBYTE: u64 = GIBIBYTE * 1024;

/// Converts the given value in bytes to a human readable format
/// e.g. 3195728 = 3.05 MiB
pub fn to_human(val: u64) -> String {
    let (value, unit) = if val >= TEBIBYTE {
        (val as f64 / TEBIBYTE as f64, "TiB")
    } else if val >= GIBIBYTE {
        (val as f64 / GIBIBYTE as f64, "GiB")
    } else if val >= MEBIBYTE {
        (val as f64 / MEBIBYTE as f64, "MiB")
    } else if val >= KIBIBYTE {
        (val as f64 / KIBIBYTE as f64, "KiB")
    } else {
        (val as f64 as f64, "bytes")
    };

    let result = format!("{:.2}", value);
    format!("{} {}", result.trim_suffix(".00"), unit)
}

/// Convert the given value in bytes to increments of KiB
pub fn to_kib(value: u64) -> f64 {
    value as f64 / KIBIBYTE as f64
}

/// Convert the given value in bytes to increments of MiB
pub fn to_mib(value: u64) -> f64 {
    value as f64 / MEBIBYTE as f64
}

/// Convert the given value in bytes to increments of GiB
pub fn to_gib(value: u64) -> f64 {
    value as f64 / GIBIBYTE as f64
}

/// Convert the given value in bytes to increments of TiB
pub fn to_tib(value: u64) -> f64 {
    value as f64 / TEBIBYTE as f64
}

// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn test_to_human() {
        assert_eq!(unit::bytes::to_human(10), "10 bytes");
        assert_eq!(unit::bytes::to_human(1024), "1 KiB");
        assert_eq!(unit::bytes::to_human(5024), "4.91 KiB");
        assert_eq!(unit::bytes::to_human(3 * unit::TEBIBYTE), "3 TiB");
        assert_eq!(unit::bytes::to_human(3 * unit::GIBIBYTE + 500), "3 GiB");
        assert_eq!(unit::bytes::to_human(3 * unit::GIBIBYTE + 500 * unit::MEBIBYTE), "3.49 GiB");
        assert_eq!(unit::bytes::to_human(3 * unit::MEBIBYTE + 50000), "3.05 MiB");
        assert_eq!(unit::bytes::to_human(3195728), "3.05 MiB");
    }

    #[test]
    fn test_to_kib() {
        assert_eq!(unit::bytes::to_kib(1000 * unit::KIBIBYTE), 1000.0);
    }

    #[test]
    fn test_to_mib() {
        assert_eq!(unit::bytes::to_mib(3195728), 3.0476837158203125);
    }

    #[test]
    fn test_to_gib() {
        assert_eq!(unit::bytes::to_gib(1000 * unit::GIBIBYTE), 1000.0);
    }

    #[test]
    fn test_to_tib() {
        assert_eq!(unit::bytes::to_tib(1000 * unit::TEBIBYTE), 1000.0);
    }
}
