use std::env;
use std::path::PathBuf;

use crate::prelude::*;

/// Returns the full path to the directory of the current running executable.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let dir = env::current_exe().unwrap().dir().unwrap();
/// assert_eq!(exec::dir().unwrap(), dir);
/// ```
pub fn dir() -> Result<PathBuf> {
    Ok(env::current_exe()?.dir()?)
}

/// Check if the given executable exists in the `PATH` and is executable.
///
/// ### Examples
/// ```ignore
/// use fungus::prelude::*;
///
/// let dir = env::current_exe().unwrap().dir().unwrap();
/// assert_eq!(exec::dir().unwrap(), dir);
/// ```
pub fn exists() -> bool {
    false
}

/// Returns the full path of the given executable. Uses given path if resolvable and falls back on
/// the system `PATH` if simply a exec name.
/// ```
pub fn lookup<T: AsRef<Path>>(target: T) -> Result<PathBuf> {
    let path = target.as_ref();
    match path.has("/") {
        // Target is a path
        true => {
            let path = path.abs()?;
            if !path.exists() {
                return Err(PathError::does_not_exist(path).into());
            } else if path.is_dir() || !path.is_exec() {
                return Err(PathError::is_not_exec(path).into());
            }
            return Ok(path);
        }

        // Target is a name
        false => {
            let base = path.to_string()?;
            for dir in env::var("PATH")?.split(':') {
                // Unix shell semantics: path element "" means "."
                let path = match dir == "" {
                    true => sys::mash(".", &base).abs()?,
                    false => sys::mash(dir, &base).abs()?,
                };
                if !path.is_dir() && path.is_exec() {
                    return Ok(path);
                }
            }
            return Err(PathError::does_not_exist(target).into());
        }
    }
}

/// Returns the current running executable's name.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let base = env::current_exe().unwrap().base().unwrap();
/// assert_eq!(exec::name().unwrap(), base);
/// ```
pub fn name() -> Result<String> {
    Ok(env::current_exe()?.base()?)
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    // Reusable teset setup
    struct Setup {
        temp: PathBuf,
    }
    impl Setup {
        fn init() -> Self {
            let setup = Self { temp: PathBuf::from("tests/temp").abs().unwrap() };
            sys::mkdir(&setup.temp).unwrap();
            setup
        }
    }

    #[test]
    fn test_dir() {
        let cwd = env::current_dir().unwrap();
        let dir = cwd.mash("target/debug/deps");
        assert_eq!(exec::dir().unwrap(), dir);
    }

    // Can't modify PATH in parallel
    // #[test]
    // fn test_lookup() {
    //     let setup = Setup::init();
    //     let tmpdir = setup.temp.mash("exec_lookup");
    //     let file1 = tmpdir.mash("file1");
    //     assert!(sys::remove_all(&tmpdir).is_ok());
    //     assert!(sys::mkdir(&tmpdir).is_ok());

    //     // Test lookup by path
    //     assert!(sys::touch_p(&file1, 0o755).is_ok());
    //     assert_eq!(file1.is_exec(), true);
    //     assert_eq!(exec::lookup(&file1).unwrap(), file1.abs().unwrap());

    //     // Test lookup by PATH
    //     let saved_path = env::var("PATH").unwrap();
    //     let new_path = format!("{}:{}", tmpdir.abs().unwrap().to_string().unwrap(), &saved_path);
    //     println!("{}", &new_path);
    //     env::set_var("PATH", new_path);
    //     assert_eq!(exec::lookup(file1.base().unwrap()).unwrap(), file1.abs().unwrap());

    //     assert!(sys::remove_all(&tmpdir).is_ok());
    // }

    #[test]
    fn test_name() {
        let exec_path = env::current_exe().unwrap();
        let name = exec_path.base().unwrap();
        assert_eq!(name, exec::name().unwrap());
    }
}
