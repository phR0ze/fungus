// Exports
pub mod agent;
pub mod git;

// Export extensions
cfgblock! {
    #[cfg(any(feature = "_net_", feature = "_arch_"))]
    pub use git2::{FetchOptions, Progress, RemoteCallbacks};
    pub use git2::build::{CheckoutBuilder, RepoBuilder};
}
