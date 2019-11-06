#[cfg(test)]
mod tests {
    use sys::*;

    #[test]
    fn it_works() {
        for x in getpaths(&"*").unwrap() {
            println!("{:?}", x)
        }
    }
}
