// Exports
pub mod agent;

// Export extensions
cfgblock! {
    #[cfg(any(feature = "_net_", feature = "_arch_"))]
    pub mod git;
    pub use git2::{FetchOptions, Progress, RemoteCallbacks};
    pub use git2::build::{CheckoutBuilder, RepoBuilder};
}
