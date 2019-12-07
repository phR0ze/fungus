use std::env;
use std::path::{Component, Path, PathBuf};

use core::*;

// Path utilities
// -------------------------------------------------------------------------------------------------
pub mod paths {
    use super::*;

    /// Return the given path in an absolute clean form
    pub fn abs<T: AsRef<Path>>(path: T) -> Result<PathBuf> {
        let _path = path.as_ref();

        // Check for empty string
        if _path.empty() {
            return Err(PathError::empty());
        }

        // Expand home directory
        let mut path_buf = _path.expand()?;

        // Trim protocol prefix if needed
        path_buf = path_buf.trim_protocol()?;

        // Clean the resulting path
        path_buf = path_buf.clean()?;

        // Expand relative directories if needed
        if !path_buf.is_absolute() {
            let curr = env::current_dir()?;

            // Unwrap works here as there will always be Some
            path_buf = match path_buf.first()? {
                Component::CurDir => curr.join(path_buf),
                Component::ParentDir => curr.dirname()?.join(path_buf.trim_first()?),
                _ => curr.join(path_buf),
            }
        }

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

    /// Returns a vector of all paths from the given target glob with path expansion and sorted by
    /// name.
    ///
    /// Doesn't include the target itself only its children nor is this recursive.
    pub fn glob<T: AsRef<Path>>(pattern: T) -> Result<Vec<PathBuf>> {
        let mut paths: Vec<PathBuf> = Vec::new();
        let _str = pattern.as_ref().to_string()?;
        for x in glob::glob(&_str)? {
            paths.push(x?.abs()?);
        }
        Ok(paths)
    }
}

// Path extensions
// -------------------------------------------------------------------------------------------------
pub trait PathExt {
    /// Return the path in an absolute clean form
    fn abs(&self) -> Result<PathBuf>;

    /// Return the shortest path equivalent to the path by purely lexical processing and thus does not handle
    /// links correctly in some cases, use canonicalize in those cases. It applies the following rules
    /// interatively until no further processing can be done.
    ///
    ///	1. Replace multiple slashes with a single
    ///	2. Eliminate each . path name element (the current directory)
    ///	3. Eliminate each inner .. path name element (the parent directory)
    ///	   along with the non-.. element that precedes it.
    ///	4. Eliminate .. elements that begin a rooted path:
    ///	   that is, replace "/.." by "/" at the beginning of a path.
    ///  5. Leave intact ".." elements that begin a non-rooted path.
    ///  6. Drop trailing '/' unless it is the root
    ///
    /// If the result of this process is an empty string, return the string `.`, representing the current directory.
    fn clean(&self) -> Result<PathBuf>;

    /// Returns the `Path` without its final component, if there is one.
    fn dirname(&self) -> Result<PathBuf>;

    /// Returns true if the `Path` is empty.
    fn empty(&self) -> bool;

    /// Expand the path to include the home prefix if necessary
    fn expand(&self) -> Result<PathBuf>;

    /// Returns the first path component.
    fn first(&self) -> Result<Component>;

    /// Returns true if the `Path` as a String contains the given string
    fn has<T: AsRef<str>>(&self, value: T) -> bool;

    /// Returns true if the `Path` as a String has the given string prefix
    fn has_prefix<T: AsRef<str>>(&self, value: T) -> bool;

    /// Returns true if the `Path` as a String has the given string suffix
    fn has_suffix<T: AsRef<str>>(&self, value: T) -> bool;

    /// Returns the last path component.
    fn last(&self) -> Result<Component>;

    /// Returns the final component of the `Path`, if there is one.
    fn name(&self) -> Result<String>;

    /// Returns the `Path` as a String
    fn to_string(&self) -> Result<String>;

    /// Returns the `Path` with the file extension removed
    fn trim_ext(&self) -> Result<PathBuf>;

    /// Returns the `Path` with the first component trimmed off
    fn trim_first(&self) -> Result<PathBuf>;

    /// Returns the `Path` with the last component trimmed off
    fn trim_last(&self) -> Result<PathBuf>;

    /// Returns the `Path` with well known protocol prefixes removed.
    fn trim_protocol(&self) -> Result<PathBuf>;

    /// Returns a string slice with the given suffix trimmed off else the original string.
    fn trim_suffix<T: AsRef<str>>(&self, value: T) -> Result<PathBuf>;
}

impl PathExt for Path {
    fn abs(&self) -> Result<PathBuf> {
        paths::abs(self)
    }

    fn clean(&self) -> Result<PathBuf> {
        // Components already handles the following cases:
        // 1. Repeated separators are ignored, so a/b and a//b both have a and b as components.
        // 2. Occurrences of . are normalized away, except if they are at the beginning of the path.
        //    e.g. a/./b, a/b/, a/b/. and a/b all have a and b as components, but ./a/b starts with an additional CurDir component.
        // 6. A trailing slash is normalized away, /a/b and /a/b/ are equivalent.
        let mut cnt = 0;
        let mut prev = None;
        let mut path_buf = PathBuf::new();
        for component in self.components() {
            match component {
                // 2. Eliminate . path name at begining of path for simplicity
                x if x == Component::CurDir && cnt == 0 => continue,

                // 5. Leave .. begining non rooted path
                x if x == Component::ParentDir && cnt > 0 && !prev.has(&Component::ParentDir) => {
                    match prev.unwrap() {
                        // 4. Eliminate .. elements that begin a root path
                        Component::RootDir => (),

                        // 3. Eliminate inner .. path name elements
                        Component::Normal(_) => {
                            cnt -= 1;
                            path_buf.pop();
                            prev = path_buf.components().last();
                        }
                        _ => (),
                    }
                    continue;
                }

                // Normal
                _ => {
                    cnt += 1;
                    path_buf.push(component);
                    prev = Some(component);
                }
            };
        }

        // Ensure if empty the current dir is returned
        if path_buf.empty() {
            path_buf.push(".");
        }
        Ok(path_buf)
    }

    fn dirname(&self) -> Result<PathBuf> {
        let dir = self.parent().ok_or_else(|| PathError::parent_not_found(self))?;
        Ok(dir.to_path_buf())
    }

    fn empty(&self) -> bool {
        match self.to_string() {
            Ok(s) => s == "",
            Err(_) => false,
        }
    }

    fn expand(&self) -> Result<PathBuf> {
        let path_str = self.to_string()?;
        let mut expanded = self.to_path_buf();

        // Check for invalid home expansion
        match path_str.matches("~").count() {
            // Only home expansion at the begining of the path is allowed
            cnt if cnt > 1 => return Err(PathError::multiple_home_symbols(self)),

            // Invalid home expansion requested
            cnt if cnt == 1 && !self.has_prefix("~/") => {
                return Err(PathError::invalid_expansion(self));
            }

            // Replace prefix with home directory
            1 => expanded = crate::users::user::home()?.join(&path_str[2..]),
            _ => (),
        }

        Ok(expanded)
    }

    fn first(&self) -> Result<Component> {
        self.components().first_result()
    }

    fn has<T: AsRef<str>>(&self, value: T) -> bool {
        match self.to_string() {
            Ok(s) => s.contains(value.as_ref()),
            Err(_) => false,
        }
    }

    fn has_prefix<T: AsRef<str>>(&self, value: T) -> bool {
        match self.to_string() {
            Ok(s) => s.starts_with(value.as_ref()),
            Err(_) => false,
        }
    }

    fn has_suffix<T: AsRef<str>>(&self, value: T) -> bool {
        match self.to_string() {
            Ok(s) => s.ends_with(value.as_ref()),
            Err(_) => false,
        }
    }

    fn last(&self) -> Result<Component> {
        self.components().last_result()
    }

    fn name(&self) -> Result<String> {
        let os_str = self.file_name().ok_or_else(|| PathError::filename_not_found(self))?;
        let filename = os_str.to_str().ok_or_else(|| PathError::failed_to_string(self))?;
        Ok(String::from(filename))
    }

    fn to_string(&self) -> Result<String> {
        let _str = self.to_str().ok_or_else(|| PathError::failed_to_string(self))?;
        Ok(String::from(_str))
    }

    fn trim_ext(&self) -> Result<PathBuf> {
        match self.file_stem() {
            Some(val) => Ok(self.trim_last()?.join(val)),
            None => Ok(self.to_path_buf()),
        }
    }

    fn trim_first(&self) -> Result<PathBuf> {
        Ok(self.components().drop(1).as_path().to_path_buf())
    }

    fn trim_last(&self) -> Result<PathBuf> {
        Ok(self.components().drop(-1).as_path().to_path_buf())
    }

    fn trim_protocol(&self) -> Result<PathBuf> {
        let mut s = self.to_string()?;
        if let Some(i) = s.find("//") {
            let (prefix, suffix) = s.split_at(i + 2);
            let lower = prefix.to_lowercase();
            let lower = lower.trim_start_matches("file://");
            let lower = lower.trim_start_matches("ftp://");
            let lower = lower.trim_start_matches("http://");
            let lower = lower.trim_start_matches("https://");
            if lower != "" {
                s = format!("{}{}", prefix, suffix);
            } else {
                return Ok(PathBuf::from(suffix));
            }
        }
        Ok(PathBuf::from(s))
    }

    fn trim_suffix<T: AsRef<str>>(&self, value: T) -> Result<PathBuf> {
        let old = self.to_string()?;
        let _value = value.as_ref();
        if old.ends_with(_value) {
            let new = &old[..old.len() - _value.len()];
            return Ok(PathBuf::from(new));
        }
        Ok(PathBuf::from(old))
    }
}

// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use std::env;
    use std::ffi::OsStr;
    use std::path::{Component, PathBuf};

    use crate::preamble::*;

    #[test]
    fn test_abs() {
        let home = env::var("HOME").unwrap();
        let cwd = env::current_dir().unwrap();
        let prev = cwd.dirname().unwrap();

        // expand previous directory and drop trailing slashes
        assert_eq!(PathBuf::from(&prev), crate::abs("..//").unwrap());
        assert_eq!(PathBuf::from(&prev), crate::abs("../").unwrap());
        assert_eq!(PathBuf::from(&prev), crate::abs("..").unwrap());

        // expand current directory and drop trailing slashes
        assert_eq!(PathBuf::from(&cwd), crate::abs(".//").unwrap());
        assert_eq!(PathBuf::from(&cwd), crate::abs("./").unwrap());
        assert_eq!(PathBuf::from(&cwd), crate::abs(".").unwrap());

        // expand relative directory
        assert_eq!(PathBuf::from(&cwd).join("foo"), crate::abs("foo").unwrap());

        // expand home path
        match crate::abs("~/foo") {
            Ok(val) => assert_eq!(PathBuf::from(&home).join("foo"), val),
            Err(e) => panic!("{:?}", e),
        }

        // More complicated
        match crate::abs("~/foo/bar/../.") {
            Ok(val) => assert_eq!(PathBuf::from(&home).join("foo"), val),
            Err(e) => panic!("{:?}", e),
        }
        match crate::abs("~/foo/bar/../") {
            Ok(val) => assert_eq!(PathBuf::from(&home).join("foo"), val),
            Err(e) => panic!("{:?}", e),
        }
        match crate::abs("~/foo/bar/../blah") {
            Ok(val) => assert_eq!(PathBuf::from(&home).join("foo/blah"), val),
            Err(e) => panic!("{:?}", e),
        }
    }

    #[test]
    fn test_exec_dir() {
        let cwd = env::current_dir().unwrap();
        let dir = cwd.parent().unwrap().join("target/debug/deps");
        assert_eq!(dir, crate::exec_dir().unwrap());
    }

    #[test]
    fn test_exec_name() {
        let exec_path = env::current_exe().unwrap();
        let name = exec_path.name().unwrap();
        assert_eq!(name, crate::exec_name().unwrap());
    }

    #[test]
    fn test_glob() {
        let cwd = env::current_dir().unwrap();

        let paths = crate::glob(&"*").unwrap();
        assert_eq!(&cwd.join(".vscode"), paths.first().unwrap());
        assert_eq!(&cwd.join("test"), paths.last().unwrap());
    }

    // Path tests
    // ---------------------------------------------------------------------------------------------
    #[test]
    fn test_pathext_clean() {
        let tests = vec![
            // Root
            ("/", "/"),
            // Remove trailing slashes
            ("/", "//"),
            ("/", "///"),
            (".", ".//"),
            // Remove duplicates and handle rooted parent ref
            ("/", "//.."),
            ("..", "..//"),
            ("/", "/..//"),
            ("foo/bar/blah", "foo//bar///blah"),
            ("/foo/bar/blah", "/foo//bar///blah"),
            // Unneeded current dirs and duplicates
            ("/", "/.//./"),
            (".", "././/./"),
            (".", "./"),
            ("/", "/./"),
            ("foo", "./foo"),
            ("foo/bar", "./foo/./bar"),
            ("/foo/bar", "/foo/./bar"),
            ("foo/bar", "foo/bar/."),
            // Handle parent references
            ("/", "/.."),
            ("/foo", "/../foo"),
            (".", "foo/.."),
            ("../foo", "../foo"),
            ("/bar", "/foo/../bar"),
            ("foo", "foo/bar/.."),
            ("bar", "foo/../bar"),
            ("/bar", "/foo/../bar"),
            (".", "foo/bar/../../"),
            ("..", "foo/bar/../../.."),
            ("/", "/foo/bar/../../.."),
            ("/", "/foo/bar/../../../.."),
            ("../..", "foo/bar/../../../.."),
            ("blah/bar", "foo/bar/../../blah/bar"),
            ("blah", "foo/bar/../../blah/bar/.."),
            ("../foo", "../foo"),
            ("../foo", "../foo/"),
            ("../foo/bar", "../foo/bar"),
            ("..", "../foo/.."),
            ("~/foo", "~/foo"),
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
        assert_eq!(PathBuf::from("").empty(), true);

        // false
        assert_eq!(PathBuf::from("/foo").empty(), false);
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
        assert_eq!(Component::Normal(OsStr::new("foo")), PathBuf::from("foo/bar").first().unwrap());
    }

    #[test]
    fn test_pathext_has() {
        let path = PathBuf::from("/foo/bar");
        assert!(path.has("foo"));
        assert!(path.has("/foo"));
        assert!(path.has("/"));
        assert!(path.has("/ba"));
        assert!(!path.has("bob"));
    }

    #[test]
    fn test_pathext_has_prefix() {
        let path = PathBuf::from("/foo/bar");
        assert_eq!(path.has_prefix("/foo"), true);
        assert_eq!(path.has_prefix("foo"), false);
    }

    #[test]
    fn test_pathext_has_suffix() {
        let path = PathBuf::from("/foo/bar");
        assert_eq!(path.has_suffix("/foo"), false);
        assert_eq!(path.has_suffix("/bar"), true);
    }

    #[test]
    fn test_pathext_name() {
        assert_eq!("bar", PathBuf::from("/foo/bar").name().unwrap());
    }

    #[test]
    fn test_pathext_last() {
        assert_eq!(Component::RootDir, PathBuf::from("/").last().unwrap());
        assert_eq!(Component::CurDir, PathBuf::from(".").last().unwrap());
        assert_eq!(Component::ParentDir, PathBuf::from("..").last().unwrap());
        assert_eq!(Component::Normal(OsStr::new("foo")), PathBuf::from("foo").last().unwrap());
        assert_eq!(Component::Normal(OsStr::new("bar")), PathBuf::from("/foo/bar").last().unwrap());
    }

    #[test]
    fn test_pathext_to_string() {
        assert_eq!("/foo".to_string(), PathBuf::from("/foo").to_string().unwrap());
    }

    #[test]
    fn test_pathext_trim_last() {
        assert_eq!(PathBuf::new(), PathBuf::from("/").trim_last().unwrap());
        assert_eq!(PathBuf::from("/"), PathBuf::from("/foo").trim_last().unwrap());
    }

    #[test]
    fn test_pathext_trim_ext() {
        assert_eq!(PathBuf::new(), PathBuf::from("").trim_ext().unwrap());
        assert_eq!(PathBuf::from("foo"), PathBuf::from("foo.exe").trim_ext().unwrap());
        assert_eq!(PathBuf::from("/foo/bar"), PathBuf::from("/foo/bar.exe").trim_ext().unwrap());
    }

    #[test]
    fn test_pathext_trim_first() {
        assert_eq!(PathBuf::new(), PathBuf::from("/").trim_first().unwrap());
        assert_eq!(PathBuf::from("foo"), PathBuf::from("/foo").trim_first().unwrap());
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

        // Check case is being considered
        assert_eq!(PathBuf::from("Foo"), PathBuf::from("HTTPS://Foo").trim_protocol().unwrap());
        assert_eq!(PathBuf::from("Foo"), PathBuf::from("Https://Foo").trim_protocol().unwrap());
        assert_eq!(PathBuf::from("FoO"), PathBuf::from("HttpS://FoO").trim_protocol().unwrap());

        // Check non protocol matches are ignored
        assert_eq!(PathBuf::from("foo"), PathBuf::from("foo").trim_protocol().unwrap());
        assert_eq!(PathBuf::from("foo/bar"), PathBuf::from("foo/bar").trim_protocol().unwrap());
        assert_eq!(PathBuf::from("foo//bar"), PathBuf::from("foo//bar").trim_protocol().unwrap());
        assert_eq!(PathBuf::from("ntp:://foo"), PathBuf::from("ntp:://foo").trim_protocol().unwrap());
    }

    #[test]
    fn test_pathext_trim_suffix() {
        // drop root
        assert_eq!(PathBuf::new(), PathBuf::from("/").trim_suffix("/").unwrap());

        // drop end
        assert_eq!(PathBuf::from("/foo"), PathBuf::from("/foo/").trim_suffix("/").unwrap());

        // no change
        assert_eq!(PathBuf::from("/foo"), PathBuf::from("/foo").trim_suffix("/").unwrap());
    }
}
