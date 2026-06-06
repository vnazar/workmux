//! Pi agent status tracking setup.
//!
//! Detects pi via its config directory at `~/.pi/agent/`.
//! Override with `PI_CODING_AGENT_DIR` env var.
//!
//! Installs extension by writing `workmux-status.ts` to the extensions directory.

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

use super::StatusCheck;

/// The pi extension source, embedded at compile time.
const EXTENSION_SOURCE: &str = include_str!("../../.pi/extensions/workmux-status.ts");

fn pi_agent_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("PI_CODING_AGENT_DIR") {
        return Some(PathBuf::from(dir));
    }
    home::home_dir().map(|h| h.join(".pi/agent"))
}

fn extension_path() -> Option<PathBuf> {
    pi_agent_dir().map(|d| d.join("extensions/workmux-status.ts"))
}

/// Detect if pi is present via filesystem.
/// Returns the reason string if detected, None otherwise.
pub fn detect() -> Option<&'static str> {
    if std::env::var("PI_CODING_AGENT_DIR").is_ok_and(|d| PathBuf::from(d).is_dir()) {
        return Some("found $PI_CODING_AGENT_DIR");
    }
    if pi_agent_dir().is_some_and(|d| d.is_dir()) {
        return Some("found ~/.pi/agent/");
    }
    None
}

/// Check if workmux extension is installed for pi.
pub fn check() -> Result<StatusCheck> {
    let Some(path) = extension_path() else {
        return Ok(StatusCheck::NotInstalled);
    };

    if path.exists() {
        Ok(StatusCheck::Installed)
    } else {
        Ok(StatusCheck::NotInstalled)
    }
}

/// Install workmux extension for pi.
/// Returns a description of what was done.
pub fn install() -> Result<String> {
    let path =
        extension_path().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("Failed to create pi extensions directory")?;
    }

    fs::write(&path, EXTENSION_SOURCE).context("Failed to write pi extension")?;

    Ok(format!(
        "Installed extension to {}. Restart pi for it to take effect.",
        path.display()
    ))
}

/// Remove workmux extension for pi agent.
///
/// Deletes the extension file and cleans up empty parent directories.
pub fn uninstall() -> Result<String> {
    let Some(path) = extension_path() else {
        return Ok("pi config dir not found, nothing to uninstall".to_string());
    };
    uninstall_at(path)
}

fn uninstall_at(path: PathBuf) -> Result<String> {
    if !path.exists() {
        return Ok("No pi extension found".to_string());
    }
    fs::remove_file(&path)?;
    // Clean up empty extensions directory
    if let Some(parent) = path.parent()
        && parent.read_dir().is_ok_and(|mut it| it.next().is_none())
    {
        let _ = fs::remove_dir(parent);
    }
    Ok(format!("Removed pi extension at {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uninstall_no_extension_file() {
        let tmp = tempfile::tempdir().unwrap();
        let ext_path = tmp.path().join("extensions/workmux-status.ts");
        let result = uninstall_at(ext_path).unwrap();
        assert!(result.contains("No pi extension found"));
    }

    #[test]
    fn test_uninstall_removes_extension_file() {
        let tmp = tempfile::tempdir().unwrap();
        let ext_dir = tmp.path().join("extensions");
        std::fs::create_dir_all(&ext_dir).unwrap();
        let ext_path = ext_dir.join("workmux-status.ts");
        std::fs::write(&ext_path, "// extension").unwrap();

        let result = uninstall_at(ext_path.clone()).unwrap();
        assert!(result.contains("Removed pi extension"));
        assert!(!ext_path.exists());
    }

    #[test]
    fn test_uninstall_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        let ext_path = tmp.path().join("extensions/workmux-status.ts");
        let result1 = uninstall_at(ext_path.clone()).unwrap();
        assert!(result1.contains("No pi extension found"));
        let result2 = uninstall_at(ext_path).unwrap();
        assert!(result2.contains("No pi extension found"));
    }
}
