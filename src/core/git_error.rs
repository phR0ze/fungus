cfgblock! {
    #[cfg(any(feature = "_net_", feature = "_arch_"))]
    use failure::Fail;
    use std::fmt;
}

// An error indicating that something went wrong with an arch linux operation
#[cfg(any(feature = "_net_", feature = "_arch_"))]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Fail)]
pub enum GitError {
    /// An error indicating a failure to convert the file value to a string.
    FailedToString,

    /// An error indicating that the given branch was not found.
    BranchNotFound(String),

    /// An error indicating that the given repo was not found.
    RepoNotFound(String),
}

#[cfg(any(feature = "_net_", feature = "_arch_"))]
impl GitError {
    /// Return an error indicating that the given branch was not found.
    pub fn branch_not_found<T: AsRef<str>>(pkg: T) -> GitError {
        GitError::BranchNotFound(pkg.as_ref().to_string())
    }

    /// Return an error indicating that the given repo was not found.
    pub fn repo_not_found<T: AsRef<str>>(repo: T) -> GitError {
        GitError::RepoNotFound(repo.as_ref().to_string())
    }
}

#[cfg(any(feature = "_net_", feature = "_arch_"))]
impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GitError::FailedToString => write!(f, "failed to convert file value to string"),
            GitError::BranchNotFound(ref pkg) => write!(f, "failed to find branch: {}", pkg),
            GitError::RepoNotFound(ref repo) => write!(f, "failed to find repo: {}", repo),
        }
    }
}

#[cfg(any(feature = "_net_", feature = "_arch_"))]
#[cfg(test)]
mod tests {
    use crate::core::*;

    #[test]
    fn test_errors() {
        assert_eq!(format!("{}", GitError::FailedToString), "failed to convert file value to string");
        assert_eq!(GitError::branch_not_found("foo"), GitError::BranchNotFound("foo".to_string()));
        assert_eq!(format!("{}", GitError::BranchNotFound("foo".to_string())), "failed to find branch: foo");
        assert_eq!(GitError::repo_not_found("foo"), GitError::RepoNotFound("foo".to_string()));
        assert_eq!(format!("{}", GitError::RepoNotFound("foo".to_string())), "failed to find repo: foo");
    }
}
