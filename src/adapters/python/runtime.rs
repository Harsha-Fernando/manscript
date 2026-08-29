//! Python runtime notes live in the uv/system providers. This module exists
//! so adapters/python/runtime.rs matches the planned layout.

pub fn language_id() -> &'static str {
    "python"
}
