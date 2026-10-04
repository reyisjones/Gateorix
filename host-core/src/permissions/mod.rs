//! Permission enforcement module.
//!
//! Checks whether a requested operation is allowed by the application manifest.
//! All capability checks go through this module before execution.

use crate::app::PermissionConfig;
use cap_std::{ambient_authority, fs::Dir};
use std::io;
use std::path::{Component, Path, PathBuf};

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
    scopes: Vec<(PathBuf, Dir)>,
}

impl PermissionGuard {
    /// Create a new guard from manifest permissions and the project root.
    pub fn new(config: PermissionConfig, base_dir: PathBuf) -> Self {
        let scopes = config
            .filesystem
            .iter()
            .filter_map(|scope| {
                let relative = relative_path(Path::new(scope)).ok()?;
                let directory =
                    Dir::open_ambient_dir(base_dir.join(&relative), ambient_authority()).ok()?;
                Some((relative, directory))
            })
            .collect();
        Self { config, scopes }
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
        match self.filesystem(requested_path) {
            Ok(scope) => match scope.directory.metadata(&scope.relative) {
                Ok(_) => PermissionResult::Allowed,
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    match scope.directory.symlink_metadata(&scope.relative) {
                        Ok(_) => PermissionResult::Denied("unresolved filesystem target".into()),
                        Err(error) if error.kind() == io::ErrorKind::NotFound => {
                            let parent = scope
                                .relative
                                .parent()
                                .filter(|parent| !parent.as_os_str().is_empty())
                                .unwrap_or(Path::new("."));
                            match scope.directory.open_dir(parent) {
                                Ok(_) => PermissionResult::Allowed,
                                Err(error) => PermissionResult::Denied(error.to_string()),
                            }
                        }
                        Err(error) => PermissionResult::Denied(error.to_string()),
                    }
                }
                Err(error) => PermissionResult::Denied(error.to_string()),
            },
            Err(error) => PermissionResult::Denied(error.to_string()),
        }
    }

    pub fn filesystem(&self, requested_path: &Path) -> io::Result<ScopedFilesystem<'_>> {
        let requested = relative_path(requested_path)?;
        let (scope, directory) = self
            .scopes
            .iter()
            .filter(|(scope, _)| requested.starts_with(scope))
            .max_by_key(|(scope, _)| scope.components().count())
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "path outside permitted filesystem scopes",
                )
            })?;
        let relative = requested.strip_prefix(scope).map_err(io::Error::other)?;
        Ok(ScopedFilesystem {
            directory,
            relative: if relative.as_os_str().is_empty() {
                PathBuf::from(".")
            } else {
                relative.to_path_buf()
            },
        })
    }
}

fn relative_path(path: &Path) -> io::Result<PathBuf> {
    if path
        .components()
        .any(|component| !matches!(component, Component::Normal(_) | Component::CurDir))
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "absolute paths and parent traversal are not permitted",
        ));
    }
    Ok(path
        .components()
        .filter(|component| !matches!(component, Component::CurDir))
        .collect())
}

pub struct ScopedFilesystem<'guard> {
    directory: &'guard Dir,
    relative: PathBuf,
}

impl ScopedFilesystem<'_> {
    pub fn read_text(&self) -> io::Result<String> {
        self.directory.read_to_string(&self.relative)
    }

    pub fn write_text(&self, content: &str) -> io::Result<()> {
        self.directory.write(&self.relative, content)
    }

    pub fn exists(&self) -> io::Result<bool> {
        match self.directory.metadata(&self.relative) {
            Ok(_) => Ok(true),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }

    pub fn list_dir(&self) -> io::Result<Vec<String>> {
        self.directory
            .read_dir(&self.relative)?
            .map(|entry| entry.map(|entry| entry.file_name().to_string_lossy().into_owned()))
            .collect()
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
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("data")).unwrap();
        let guard = PermissionGuard::new(test_config(), root.path().into());
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

    #[test]
    fn scoped_operations_allow_new_files_and_deny_escapes() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("data")).unwrap();
        let guard = PermissionGuard::new(test_config(), root.path().into());
        let file = guard.filesystem(Path::new("./data/new.txt")).unwrap();
        file.write_text("hello").unwrap();
        assert_eq!(file.read_text().unwrap(), "hello");
        assert!(file.exists().unwrap());
        assert_eq!(
            guard
                .filesystem(Path::new("data"))
                .unwrap()
                .list_dir()
                .unwrap(),
            vec!["new.txt"]
        );
        for path in [
            "data/../secret",
            "data-other/file",
            "../data/file",
            "/data/file",
        ] {
            assert!(guard.filesystem(Path::new(path)).is_err(), "{path}");
        }
        assert!(guard.filesystem(&root.path().join("data/new.txt")).is_err());
        assert!(!guard
            .check(&Capability::Filesystem("data/missing/file".into()))
            .is_allowed());
    }

    #[cfg(unix)]
    #[test]
    fn symlink_escape_is_denied_by_actual_io() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("data")).unwrap();
        std::fs::write(root.path().join("secret"), "unchanged").unwrap();
        std::os::unix::fs::symlink("../secret", root.path().join("data/link")).unwrap();
        let guard = PermissionGuard::new(test_config(), root.path().into());
        let file = guard.filesystem(Path::new("data/link")).unwrap();
        assert!(file.read_text().is_err());
        assert!(file.write_text("changed").is_err());
        assert!(!guard
            .check(&Capability::Filesystem("data/link".into()))
            .is_allowed());
        assert_eq!(
            std::fs::read_to_string(root.path().join("secret")).unwrap(),
            "unchanged"
        );
    }
}
