use glob::glob;
use std::env;
use std::io;
use std::path::{Component, Path, PathBuf};

use core::*;

// Path utilities
// -------------------------------------------------------------------------------------------------
pub fn abs<T: AsRef<Path>>(path: T) -> Result<PathBuf> {
    let _path = path.as_ref();

    // Check for empty string
    if _path.to_string()? == "" {
        return Err(PathError::Empty.into());
    }

    // Expand home directory and trim trailing slash if needed
    let mut path_buf = _path.expand()?;
    let mut path_str = path_buf.to_string()?;
    if path_str.len() > 1 {
        path_buf = path_buf.trim_end_matches("/")?;
        path_str = path_buf.to_string()?;
    }

    // Expand current directory if needed
    if !path_buf.is_absolute() {
        // Unwrap is acceptable here as Some will always exist
        match path_str.split("/").next().unwrap() {
            "." => path_buf = env::current_dir()?.join(&path_str[1..]),
            ".." => path_buf = env::current_dir()?.dirname()?.join(&path_str[2..]),
            _ => path_buf = env::current_dir()?.join(path_buf),
        }
    }

    // Clean the path

    Ok(path_buf)
}

// Returns the full path to the directory of the current running executable.
pub fn exec_dir() -> Result<PathBuf> {
    Ok(env::current_exe()?.dirname()?)
}

// Returns the current running executable's name.
pub fn exec_name() -> Result<String> {
    Ok(env::current_exe()?.name()?)
}

// Returns a vector of all paths from the given target glob, sorted by name.
// Doesn't include the target itself only its children nor is this recursive.
pub fn getpaths<T: AsRef<Path>>(pattern: T) -> Result<Vec<PathBuf>> {
    let mut paths: Vec<PathBuf> = Vec::new();
    let _str = pattern.as_ref().to_string()?;
    for x in glob(&_str)? {
        let path = x?;
        paths.push(path);
    }
    Ok(paths)
}

// Path extensions
// -------------------------------------------------------------------------------------------------
pub trait PathExt {
    fn contains_str<T: AsRef<str>>(&self, value: T) -> bool;
    fn clean(&self) -> Result<PathBuf>;
    fn dirname(&self) -> Result<PathBuf>;
    fn empty(&self) -> bool;
    fn expand(&self) -> Result<PathBuf>;
    fn first(&self) -> Result<Component>;
    fn last(&self) -> Result<Component>;
    fn name(&self) -> Result<String>;
    fn starts_with_str<T: AsRef<str>>(&self, value: T) -> bool;
    fn to_string(&self) -> Result<String>;
    fn trim_protocol(&self) -> Result<PathBuf>;
    fn trim_end_matches<T: AsRef<str>>(&self, value: T) -> Result<PathBuf>;
}
impl PathExt for Path {
    // Returns true if the `Path` as a String contains the given string
    fn contains_str<T: AsRef<str>>(&self, value: T) -> bool {
        let res = self.to_string();
        let _str = match res {
            Ok(s) => s,
            Err(_) => return false,
        };
        if _str.contains(value.as_ref()) {
            return true;
        }
        false
    }

    // Return the shortest path equivalent to the path by purely lexical processing and thus does not handle
    // links correctly in some cases, use canonicalize in those cases. It applies the following rules
    // interatively until no further processing can be done.
    //
    //	1. Replace multiple slashes with a single
    //	2. Eliminate each . path name element (the current directory)
    //	3. Eliminate each inner .. path name element (the parent directory)
    //	   along with the non-.. element that precedes it.
    //	4. Eliminate .. elements that begin a rooted path:
    //	   that is, replace "/.." by "/" at the beginning of a path.
    //  5. Leave intact ".." elements that begin a non-rooted path.
    //  6. Drop trailing '/' unless it is the root
    //
    // If the result of this process is an empty string, return the string `.`, representing the current directory.
    fn clean(&self) -> Result<PathBuf> {
        let path_str = self.to_string()?;

        // Components already handles the following cases:
        // 1. Repeated separators are ignored, so a/b and a//b both have a and b as components.
        // 2. Occurrences of . are normalized away, except if they are at the beginning of the path.
        //    e.g. a/./b, a/b/, a/b/. and a/b all have a and b as components, but ./a/b starts with an additional CurDir component.
        // 6. A trailing slash is normalized away, /a/b and /a/b/ are equivalent.
        let mut cnt = 0;
        let mut path_buf = PathBuf::new();
        for component in self.components() {
            match component {
                // 2. Eliminate . path name at begining of path
                x if x == Component::CurDir && cnt == 0 => continue,

                // 4/5. Eliminate .. elements that begin a root path and leave .. begining non rooted path
                x if x == Component::ParentDir && cnt == 1 && path_buf.starts_with("/") => continue,

                // 3. Eliminate inner .. path name elements
                x if x == Component::ParentDir && cnt > 0 => {
                    // match path_buf.first()? {
                    //     Component::CurDir => (),
                    //     Component::ParentDir => (),
                    //     Component::RootDir => (),
                    //     Component::Normal => (),
                    //     _ => (),
                    // }
                    // // path_buf.to_string()? != "/");
                    // // path_buf.pop();
                    // // cnt -= 1;
                    // // continue;
                    continue;
                }

                _ => false,
            };

            cnt += 1;
            path_buf.push(component);
        }

        // Ensure if empty the current dir is returned
        if path_buf.empty() {
            path_buf.push(".");
        }
        Ok(path_buf)
    }

    // Returns the `Path` without its final component, if there is one.
    fn dirname(&self) -> Result<PathBuf> {
        let dir = self.parent().ok_or_else(|| PathError::ParentNotFound)?;
        Ok(dir.to_path_buf())
    }

    // Returns true if the `Path` is empty.
    fn empty(&self) -> bool {
        let res = self.to_string();
        let _str = match res {
            Ok(s) => s,
            Err(_) => return false,
        };
        _str == ""
    }

    // Expand the path to include the home prefix if necessary
    fn expand(&self) -> Result<PathBuf> {
        let path_str = self.to_string()?;
        let mut expanded = self.to_path_buf();

        // Check for invalid home expansion
        match path_str.matches("~").count() {
            // Only home expansion at the begining of the path is allowed
            cnt if cnt > 1 => return Err(PathError::MultipleHomeSymbols.into()),

            // Invalid home expansion requested
            cnt if cnt == 1 && !self.starts_with_str("~/") => {
                return Err(PathError::InvalidExpansion.into());
            }

            // Replace prefix with home directory
            1 => expanded = crate::user_home()?.join(&path_str[2..]),
            _ => (),
        }

        Ok(expanded)
    }

    // Returns the first path component.
    fn first(&self) -> Result<Component> {
        let component = self.components().next().ok_or_else(|| PathError::ComponentNotFound)?;
        Ok(component)
    }

    // Returns the last path component.
    fn last(&self) -> Result<Component> {
        let component = self.components().next().ok_or_else(|| PathError::ComponentNotFound)?;
        Ok(component)
    }

    // Returns the final component of the `Path`, if there is one.
    fn name(&self) -> Result<String> {
        let os_str = self.file_name().ok_or_else(|| PathError::FileNameNotFound)?;
        let filename = os_str.to_str().ok_or_else(|| PathError::FailedToString)?;
        Ok(String::from(filename))
    }

    // Returns true if the `Path` as a String starts with the given string
    fn starts_with_str<T: AsRef<str>>(&self, value: T) -> bool {
        let res = self.to_string();
        let _str = match res {
            Ok(s) => s,
            Err(_) => return false,
        };
        if _str.contains(value.as_ref()) {
            return true;
        }
        false
    }

    // Returns the `Path` as a String
    fn to_string(&self) -> Result<String> {
        let _str = self.to_str().ok_or_else(|| PathError::FailedToString)?;
        Ok(String::from(_str))
    }

    // Returns the `Path` with well known protocol prefixes removed.
    fn trim_protocol(&self) -> Result<PathBuf> {
        let _str = self.to_string()?;
        let _str = _str.to_lowercase();
        let _str = _str.trim_start_matches("file://");
        let _str = _str.trim_start_matches("ftp://");
        let _str = _str.trim_start_matches("http://");
        let _str = _str.trim_start_matches("https://");
        Ok(PathBuf::from(_str))
    }

    // Returns a string slice with all suffixes that match a pattern repeatedly removed.
    fn trim_end_matches<T: AsRef<str>>(&self, value: T) -> Result<PathBuf> {
        let _str = self.to_string()?;
        let _value = value.as_ref();
        Ok(PathBuf::from(_str.trim_end_matches(_value)))
    }
}

// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    #[test]
    fn test_abs() {
        let home = env::var("HOME").unwrap();
        let cwd = env::current_dir().unwrap();
        let prev = cwd.dirname().unwrap();

        // expand previous directory and drop slash
        assert_eq!(PathBuf::from(&prev), abs("../").unwrap());

        // expand previous directory
        assert_eq!(PathBuf::from(&prev), abs("..").unwrap());

        // expand current directory
        assert_eq!(PathBuf::from(&cwd), abs(".").unwrap());

        // expand relative directory
        assert_eq!(PathBuf::from(&cwd).join("foo"), abs("foo").unwrap());

        // expand home path
        assert_eq!(PathBuf::from(&home).join("foo"), abs("~/foo").unwrap());
    }

    #[test]
    fn test_exec_dir() {
        let cwd = env::current_dir().unwrap();
        let dir = cwd.parent().unwrap().join("target/debug/deps");
        assert_eq!(dir, exec_dir().unwrap());
    }

    #[test]
    fn test_exec_name() {
        let exec_path = env::current_exe().unwrap();
        let name = exec_path.name().unwrap();
        assert_eq!(name, exec_name().unwrap());
    }

    #[test]
    fn test_getpaths() {
        let paths = getpaths(&"*").unwrap();
        assert_eq!(&PathBuf::from(".vscode"), paths.first().unwrap());
        assert_eq!(&PathBuf::from("src"), paths.last().unwrap());
    }

    // Path tests
    // ---------------------------------------------------------------------------------------------
    #[test]
    fn test_pathext_contains() {
        let path = PathBuf::from("/foo/bar");
        assert!(path.contains_str("foo"));
        assert!(path.contains_str("/foo"));
        assert!(path.contains_str("/"));
        assert!(path.contains_str("/ba"));
        assert!(!path.contains_str("bob"));
    }

    #[test]
    fn test_pathext_clean() {
        let tests = vec![
            // Root case
            ("/", "/"),
            // // Remove trailing slashes
            // ("/", "//"),
            // ("/", "///"),
            // (".", ".//"),
            // // Remove duplicates and handle rooted parent ref
            // ("/", "//.."),
            //////("..", "..//"),
            // ("/", "/..//"),
            // // Unneeded current dirs and duplicates
            // ("/", "/.//./"),
            // (".", "././/./"),
            // (".", "./"),
            // ("/", "/./"),
            // ("foo", "./foo"),
            // ("foo/bar", "./foo/./bar"),
            // ("/foo/bar", "/foo/./bar"),
            // ("foo/bar", "foo/bar/."),
            // // Handle parent references
            // ("../foo", "../foo"),
            // ("/bar", "/foo/../bar"),
            // ("/", "/.."),
            // ("/foo", "/../foo"),
            // (".", "foo/.."),

            // ("foo", "foo/bar/.."),
            // ("foo", "foo/../bar"),
            // ("/foo", "/foo/../bar"),
            // (".", "foo/bar/../../"),
            // ("..", "foo/bar/../../.."),
            // ("/", "/foo/bar/../../.."),
            // ("/", "/foo/bar/../../../.."),
            // ("../..", "foo/bar/../../../.."),
            // ("test/path/../../another/path", "another/path"),
            // ("test/path/../../another/path/..", "another"),
            // ("../test", "../test"),
            // ("../test/", "../test"),
            // ("../test/path", "../test/path"),
            // ("../test/..", ".."),
        ];
        for test in tests {
            assert_eq!(PathBuf::from(test.0), PathBuf::from(test.1).clean().unwrap());
        }
    }

    #[test]
    fn test_pathext_dirname() {
        assert_eq!(PathBuf::from("/foo").as_path(), PathBuf::from("/foo/bar").dirname().unwrap());
    }

    #[test]
    fn test_pathext_empty() {
        // empty string
        assert!(PathBuf::from("").empty());

        // false
        assert!(!PathBuf::from("/foo").empty());
    }

    #[test]
    fn test_pathext_expand() {
        let home = env::var("HOME").unwrap();

        // happy path
        assert_eq!(PathBuf::from(&home).join("foo"), PathBuf::from("~/foo").expand().unwrap());

        // More than one ~
        assert!(PathBuf::from("~/foo~").expand().is_err());

        // invalid path
        assert!(PathBuf::from("~foo").expand().is_err());

        // empty path - nothing to do but no error
        assert_eq!(PathBuf::from(""), PathBuf::from("").expand().unwrap());

        // home not set
        {
            env::remove_var("HOME");
            assert!(PathBuf::from("~/foo").expand().is_err());
            env::set_var("HOME", &home);
        }
    }

    #[test]
    fn test_pathext_first() {
        assert_eq!(Component::RootDir, PathBuf::from("/").first().unwrap());
        assert_eq!(Component::CurDir, PathBuf::from(".").first().unwrap());
        assert_eq!(Component::ParentDir, PathBuf::from("..").first().unwrap());
        assert_eq!(Component::Normal(OsStr::new("foo")), PathBuf::from("foo").first().unwrap());
    }

    #[test]
    fn test_pathext_filename() {
        assert_eq!("bar", PathBuf::from("/foo/bar").name().unwrap());
    }

    #[test]
    fn test_pathext_to_string() {
        assert_eq!("/foo".to_string(), PathBuf::from("/foo").to_string().unwrap());
    }

    #[test]
    fn test_pathext_trim_protocol() {
        // no change
        assert_eq!(PathBuf::from("/foo"), PathBuf::from("/foo").trim_protocol().unwrap());

        // file://
        assert_eq!(PathBuf::from("/foo"), PathBuf::from("file:///foo").trim_protocol().unwrap());

        // ftp://
        assert_eq!(PathBuf::from("foo"), PathBuf::from("ftp://foo").trim_protocol().unwrap());

        // http://
        assert_eq!(PathBuf::from("foo"), PathBuf::from("http://foo").trim_protocol().unwrap());

        // https://
        assert_eq!(PathBuf::from("foo"), PathBuf::from("https://foo").trim_protocol().unwrap());

        // HTTPS://
        assert_eq!(PathBuf::from("foo"), PathBuf::from("HTTPS://foo").trim_protocol().unwrap());
    }

    #[test]
    fn test_pathext_trim_end_matches() {
        // drop root
        assert_eq!(PathBuf::new(), PathBuf::from("/").trim_end_matches("/").unwrap());

        // drop end
        assert_eq!(PathBuf::from("/foo"), PathBuf::from("/foo/").trim_end_matches("/").unwrap());

        // no change
        assert_eq!(PathBuf::from("/foo"), PathBuf::from("/foo").trim_end_matches("/").unwrap());
    }
}
