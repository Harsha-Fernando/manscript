use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsKind {
    Macos,
    Linux,
    Windows,
    Other,
}

pub fn os_kind() -> OsKind {
    match env::consts::OS {
        "macos" => OsKind::Macos,
        "linux" => OsKind::Linux,
        "windows" => OsKind::Windows,
        _ => OsKind::Other,
    }
}

pub fn platform_label() -> String {
    format!("{} {}", display_os(), env::consts::ARCH)
}

pub fn display_os() -> &'static str {
    match os_kind() {
        OsKind::Macos => "macOS",
        OsKind::Linux => "Linux",
        OsKind::Windows => "Windows",
        OsKind::Other => env::consts::OS,
    }
}

pub fn is_windows() -> bool {
    os_kind() == OsKind::Windows
}

pub fn env_bin_dir(environment_root: &std::path::Path) -> PathBuf {
    if is_windows() {
        environment_root.join("Scripts")
    } else {
        environment_root.join("bin")
    }
}

pub fn python_bin_name() -> &'static str {
    if is_windows() {
        "python.exe"
    } else {
        "python"
    }
}

pub fn exe_name(name: &str) -> String {
    if is_windows() && !name.ends_with(".exe") {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}

pub fn uv_download_target() -> Option<(&'static str, &'static str)> {
    // (asset archive name without extension handling, binary name)
    match (env::consts::OS, env::consts::ARCH) {
        ("macos", "aarch64") => Some(("uv-aarch64-apple-darwin", "uv")),
        ("macos", "x86_64") => Some(("uv-x86_64-apple-darwin", "uv")),
        ("linux", "aarch64") => Some(("uv-aarch64-unknown-linux-gnu", "uv")),
        ("linux", "x86_64") => Some(("uv-x86_64-unknown-linux-gnu", "uv")),
        ("windows", "x86_64") => Some(("uv-x86_64-pc-windows-msvc", "uv.exe")),
        _ => None,
    }
}

pub fn mise_platform_suffix() -> Option<(&'static str, &'static str)> {
    match (env::consts::OS, env::consts::ARCH) {
        ("macos", "aarch64") => Some(("macos-arm64", "mise")),
        ("macos", "x86_64") => Some(("macos-x64", "mise")),
        ("linux", "aarch64") => Some(("linux-arm64", "mise")),
        ("linux", "x86_64") => Some(("linux-x64", "mise")),
        ("windows", "x86_64") => Some(("windows-x64", "mise.exe")),
        _ => None,
    }
}
