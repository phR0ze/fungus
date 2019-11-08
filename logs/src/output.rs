#[macro_export]
macro_rules! out {
    ($dst:expr, $quiet:expr, $($arg:tt)*) => {{
        if !$quiet {
            $dst.write_fmt(format_args!($($arg)*)).unwrap()
        }
    }};
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    #[test]
    fn test_out() {
        let mut out = Vec::new();
        out!(out, false, "{}", "Hello World");
        assert_eq!(out, b"Hello World");
    }
}
