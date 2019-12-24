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

#[cfg(test)]
mod tests {
    use crate::presys::*;
    use std::io::{self, Write};

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
