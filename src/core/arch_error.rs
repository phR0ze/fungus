cfgblock! {
    #[cfg(feature = "_arch_")]
    use failure::Fail;
    use std::fmt;
}

// An error indicating that something went wrong with an arch linux operation
#[cfg(feature = "_arch_")]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Fail)]
pub enum ArchError {
    /// An error indicating that the given package was not found.
    PackageNotFound(String),

    /// An error indicating that the given repo was not found.
    RepoNotFound(String),
}
#[cfg(feature = "_arch_")]
impl ArchError {
    /// Return an error indicating that the given package was not found.
    pub fn package_not_found<T: AsRef<str>>(pkg: T) -> ArchError {
        ArchError::PackageNotFound(pkg.as_ref().to_string())
    }

    /// Return an error indicating that the given repo was not found.
    pub fn repo_not_found<T: AsRef<str>>(repo: T) -> ArchError {
        ArchError::RepoNotFound(repo.as_ref().to_string())
    }
}

#[cfg(feature = "_arch_")]
impl fmt::Display for ArchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ArchError::PackageNotFound(ref pkg) => write!(f, "failed to find package: {}", pkg),
            ArchError::RepoNotFound(ref repo) => write!(f, "failed to find repo: {}", repo),
        }
    }
}

#[cfg(feature = "_arch_")]
#[cfg(test)]
mod tests {
    use crate::core::*;

    #[test]
    fn test_errors() {
        assert_eq!(ArchError::package_not_found("foo"), ArchError::PackageNotFound("foo".to_string()));
        assert_eq!(format!("{}", ArchError::PackageNotFound("foo".to_string())), "failed to find package: foo");
        assert_eq!(ArchError::repo_not_found("foo"), ArchError::RepoNotFound("foo".to_string()));
        assert_eq!(format!("{}", ArchError::RepoNotFound("foo".to_string())), "failed to find repo: foo");
    }
}
