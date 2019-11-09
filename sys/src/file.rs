// use std::fs::File;
// use std::io::prelude::*;
// use std::path::Path;

// use errors::Result;

// // File utilities
// // -------------------------------------------------------------------------------------------------

// // Returns the contents of the target path as a String.
// pub fn read_string<T: AsRef<Path>>(path: &T) -> Result<String> {
//     let mut file = File::open(path.as_ref())?;
//     let mut contents = String::new();
//     file.read_to_string(&mut contents)?;
//     Ok(contents)
// }

// // Writes the String to the given file path.
// pub fn write_string<T: AsRef<Path>>(path: &T) -> Result<()> {
//     let mut file = File::open(path.as_ref())?;
//     let mut contents = String::new();
//     file.read_to_string(&mut contents)?;
//     Ok(contents)
// }

// // Unit tests
// // -------------------------------------------------------------------------------------------------
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::env;
//     use std::path::PathBuf;

//     // Get the temp dir `rs/test/temp` path
//     fn temp_dir() -> PathBuf {
//         let cwd = env::current_dir().unwrap();
//         cwd.join("../test/temp")
//     }

//     #[test]
//     fn test_read_file() {
//         //let data = read_file("");
//         temp_dir();
//     }
// }
