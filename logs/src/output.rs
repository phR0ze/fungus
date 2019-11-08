// Test defining how to write conditional output via the write*! macros
// that allows for swapping out stdout for a buffer.
#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::fmt;
    use std::io;
    use std::rc::Rc;

    struct MyObj {
        quiet: bool,
        out: Rc<RefCell<dyn io::Write>>,
    }
    impl MyObj {
        fn new(quiet: bool, out: Rc<RefCell<dyn io::Write>>) -> Self {
            MyObj { quiet, out }
        }

        // Because write*! macro variants only look for the existance of this function we
        // actually don't need to implement the entire fmt::Write trait only this func.
        fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) {
            if !self.quiet {
                self.out.borrow_mut().write_fmt(fmt).unwrap();
            }
        }
    }

    #[test]
    fn test_out() {
        // stdout
        let mut obj = MyObj::new(false, Rc::new(RefCell::new(io::stdout())));
        writeln!(obj, "{}", "Hello World");

        // Buffer
        let out: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(Vec::new()));
        let mut obj = MyObj::new(false, out.clone());
        writeln!(obj, "{}", "Hello World");
        assert_eq!(*out.borrow(), b"Hello World\n");
    }
}
