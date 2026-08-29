use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::core::errors::{ManscriptError, Result};
use crate::utils::filesystem::ensure_dir;

pub fn download_to_file(url: &str, dest: &Path) -> Result<()> {
    // Progress label is fixed copy (no URL/tokens). Failure messages may name the URL after the spinner stops.
    let spin = crate::utils::output::download_spinner();
    let result = download_to_file_inner(url, dest);
    match &result {
        Ok(()) => spin.finish_ok("Download complete"),
        Err(_) => drop(spin),
    }
    result
}

fn download_to_file_inner(url: &str, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        ensure_dir(parent)?;
    }
    let response = ureq::get(url)
        .set("User-Agent", "manscript/0.1")
        .call()
        .map_err(|e| {
            ManscriptError::Message(format!(
                "ManScript could not download a required tool.\n\nURL:\n  {url}\n\nNetwork detail:\n  {e}\n\nCheck your connection and try `manscript setup` again."
            ))
        })?;
    let mut reader = response.into_reader();
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .map_err(|e| {
            ManscriptError::Message(format!(
                "ManScript downloaded a response but could not read it.\n\nSystem detail:\n  {e}\n\nTry `manscript setup` again."
            ))
        })?;
    fs::write(dest, bytes)?;
    Ok(())
}

pub fn extract_tar_gz(archive: &Path, dest: &Path) -> Result<()> {
    ensure_dir(dest)?;
    let file = fs::File::open(archive)?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    archive
        .unpack(dest)
        .map_err(|e| {
            ManscriptError::Message(format!(
                "ManScript could not extract the downloaded tool archive.\n\nSystem detail:\n  {e}\n\nRemove the incomplete download from the ManScript cache and run `manscript setup` again."
            ))
        })?;
    Ok(())
}

pub fn find_named_file(root: &Path, name: &str) -> Option<PathBuf> {
    fn walk(dir: &Path, name: &str, depth: usize) -> Option<PathBuf> {
        if depth > 6 {
            return None;
        }
        let entries = fs::read_dir(dir).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(found) = walk(&path, name, depth + 1) {
                    return Some(found);
                }
            } else if path.file_name().and_then(|s| s.to_str()) == Some(name) {
                return Some(path);
            }
        }
        None
    }
    walk(root, name, 0)
}
