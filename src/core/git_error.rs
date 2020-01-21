cfgblock! {
    #[cfg(any(feature = "_net_", feature = "_arch_"))]
    use failure::Fail;
    use std::fmt;
}

// An error indicating that something went wrong with an arch linux operation
#[cfg(any(feature = "_net_", feature = "_arch_"))]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Fail)]
pub enum GitError {
    /// An error indicating that the given branch was not found.
    BranchNotFound(String),

    /// An error indicating that only fast forwards are allowed.
    FastForwardOnly,

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
            GitError::BranchNotFound(ref pkg) => write!(f, "failed to find branch: {}", pkg),
            GitError::FastForwardOnly => write!(f, "only fast-forward supported"),
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
        assert_eq!(GitError::branch_not_found("foo"), GitError::BranchNotFound("foo".to_string()));
        assert_eq!(format!("{}", GitError::BranchNotFound("foo".to_string())), "failed to find branch: foo");
        assert_eq!(format!("{}", GitError::FastForwardOnly), "only fast-forward supported");
        assert_eq!(GitError::repo_not_found("foo"), GitError::RepoNotFound("foo".to_string()));
        assert_eq!(format!("{}", GitError::RepoNotFound("foo".to_string())), "failed to find repo: foo");
    }
}
