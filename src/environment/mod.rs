//! Generic environment location helpers. Language-specific isolation lives in adapters.

pub use crate::utils::filesystem::project_environment_dir;
pub use crate::utils::platform::env_bin_dir;
