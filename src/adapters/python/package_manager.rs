//! Python package installation is implemented in the Python language adapter
//! (pip / uv). This module keeps the planned layout without leaking pip into core.

pub fn default_tool() -> &'static str {
    "pip"
}
