#[macro_export]
macro_rules! out {
    ($dst:expr, $quiet:expr, $($arg:tt)*) => {{
        if $quiet {
            $dst.write_fmt(format_args!($($arg)*)).unwrap()
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Write};

    #[test]
    fn test_out() {
        let mut out = Vec::new();
        out!(out, true, "{}", "Hello World");
        assert_eq!(out, b"Hello World");
    }
}
