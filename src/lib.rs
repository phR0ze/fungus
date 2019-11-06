use std::error::Error;

// New general purpose ok or error result
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
