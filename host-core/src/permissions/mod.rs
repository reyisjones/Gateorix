//! Permission enforcement module.
//!
//! Checks whether a requested operation is allowed by the application manifest.
//! All capability checks go through this module before execution.

use crate::app::PermissionConfig;
use std::path::{Path, PathBuf};

/// Capability categories that can be permission-gated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Capability {
    /// Filesystem access to a specific path.
    Filesystem(PathBuf),
    /// Process/shell execution.
    Process,
    /// Desktop notifications.
    Notifications,
    /// Clipboard read/write.
    Clipboard,
}

/// Permission checker backed by the app manifest configuration.
pub struct PermissionGuard {
    config: PermissionConfig,
    /// Resolved base directory for relative filesystem paths.
    base_dir: PathBuf,
}

impl PermissionGuard {
    /// Create a new guard from manifest permissions and the project root.
    pub fn new(config: PermissionConfig, base_dir: PathBuf) -> Self {
        Self { config, base_dir }
    }

    /// Check whether the given capability is allowed.
    pub fn check(&self, capability: &Capability) -> PermissionResult {
        match capability {
            Capability::Filesystem(path) => self.check_filesystem(path),
            Capability::Process => {
                if self.config.process {
                    PermissionResult::Allowed
                } else {
                    PermissionResult::Denied("process execution is not permitted".into())
                }
            }
            Capability::Notifications => {
                if self.config.notifications {
                    PermissionResult::Allowed
                } else {
                    PermissionResult::Denied("notifications are not permitted".into())
                }
            }
            Capability::Clipboard => {
                if self.config.clipboard {
                    PermissionResult::Allowed
                } else {
                    PermissionResult::Denied("clipboard access is not permitted".into())
                }
            }
        }
    }

    /// Validate that a filesystem path falls within an allowed scope.
    fn check_filesystem(&self, requested_path: &Path) -> PermissionResult {
        if self.config.filesystem.is_empty() {
            return PermissionResult::Denied("no filesystem paths are permitted".into());
        }

        let canonical_requested = self.base_dir.join(requested_path);

        for allowed in &self.config.filesystem {
            let canonical_allowed = self.base_dir.join(allowed);
            if canonical_requested.starts_with(&canonical_allowed) {
                return PermissionResult::Allowed;
            }
        }

        PermissionResult::Denied(format!(
            "path {:?} is outside permitted filesystem scopes",
            requested_path
        ))
    }
}

/// Result of a permission check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionResult {
    Allowed,
    Denied(String),
}

impl PermissionResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, PermissionResult::Allowed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::PermissionConfig;

    fn test_config() -> PermissionConfig {
        PermissionConfig {
            filesystem: vec!["./data".into()],
            process: false,
            notifications: true,
            clipboard: true,
        }
    }

    #[test]
    fn filesystem_allowed_within_scope() {
        let guard = PermissionGuard::new(test_config(), PathBuf::from("/app"));
        let result = guard.check(&Capability::Filesystem(PathBuf::from("./data/notes.txt")));
        assert!(result.is_allowed());
    }

    #[test]
    fn filesystem_denied_outside_scope() {
        let guard = PermissionGuard::new(test_config(), PathBuf::from("/app"));
        let result = guard.check(&Capability::Filesystem(PathBuf::from("./secrets/key.pem")));
        assert!(!result.is_allowed());
    }

    #[test]
    fn process_denied_by_default() {
        let guard = PermissionGuard::new(test_config(), PathBuf::from("/app"));
        assert!(!guard.check(&Capability::Process).is_allowed());
    }

    #[test]
    fn notifications_allowed() {
        let guard = PermissionGuard::new(test_config(), PathBuf::from("/app"));
        assert!(guard.check(&Capability::Notifications).is_allowed());
    }
}
