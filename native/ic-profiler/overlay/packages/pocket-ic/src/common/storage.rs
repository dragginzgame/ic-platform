//! RAM-backed temporary storage for PocketIC state and compilation files.
//!
//! This changes the filesystem backing the usual checkpoint and PageMap files;
//! it does not change canister memory tracking or instruction accounting.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

/// Storage used by a running PocketIC server, available from `GET /storage`.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
pub struct StorageInfo {
    pub ram_backed: bool,
    pub temporary_directory: PathBuf,
}

/// Read the opt-in RAM-storage flag without changing the process environment.
pub fn ram_requested_from_env() -> Result<bool, String> {
    parse_ram_flag(std::env::var_os("POCKET_IC_RAM").as_deref())
}

fn parse_ram_flag(value: Option<&OsStr>) -> Result<bool, String> {
    match value {
        None => Ok(false),
        Some(value) => match value.to_str() {
            Some("1" | "true") => Ok(true),
            Some("0" | "false") => Ok(false),
            _ => Err("POCKET_IC_RAM must be 1, true, 0, or false".into()),
        },
    }
}

/// Parent directory for private RAM workspaces. Linux commonly mounts tmpfs here.
pub fn ram_parent() -> PathBuf {
    std::env::var_os("POCKET_IC_RAM_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/dev/shm"))
}

/// Create a private workspace on validated RAM-backed storage.
pub fn new_ram_tempdir() -> Result<TempDir, String> {
    new_ram_tempdir_in(ram_parent())
}

/// Create a private workspace under a caller-selected tmpfs mount.
pub fn new_ram_tempdir_in(parent: impl AsRef<Path>) -> Result<TempDir, String> {
    let parent = parent.as_ref();
    validate_ram_directory(parent)?;
    let mut builder = tempfile::Builder::new();
    // Keep Unix-domain socket paths short, including under a custom parent.
    builder.prefix("pic-");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        builder.permissions(std::fs::Permissions::from_mode(0o700));
    }
    builder
        .tempdir_in(parent)
        .map_err(|err| format!("Cannot create RAM workspace in {}: {err}", parent.display()))
}

/// Whether an existing path is on a supported RAM-backed filesystem.
#[cfg(target_os = "linux")]
pub fn is_ram_backed(path: &Path) -> Result<bool, String> {
    use std::{ffi::CString, mem::MaybeUninit, os::unix::ffi::OsStrExt};

    let path_c = CString::new(path.as_os_str().as_bytes())
        .map_err(|_| format!("Storage path contains a NUL byte: {}", path.display()))?;
    let mut info = MaybeUninit::<libc::statfs>::uninit();
    // SAFETY: the path is NUL-terminated and info points to writable statfs storage.
    if unsafe { libc::statfs(path_c.as_ptr(), info.as_mut_ptr()) } != 0 {
        return Err(format!(
            "Cannot inspect storage at {}: {}",
            path.display(),
            std::io::Error::last_os_error()
        ));
    }
    // SAFETY: statfs succeeded and initialized info above.
    let info = unsafe { info.assume_init() };
    Ok(info.f_type == libc::TMPFS_MAGIC)
}

#[cfg(not(target_os = "linux"))]
pub fn is_ram_backed(_path: &Path) -> Result<bool, String> {
    Err("PocketIC RAM-backed storage currently requires Linux tmpfs".into())
}

/// Validate tmpfs storage, including the executable mappings needed by Wasmtime.
///
/// A `noexec` tmpfs mount is unsuitable: compilation artifacts are mapped as
/// executable code. The probe maps a file but never runs its contents.
#[cfg(target_os = "linux")]
pub fn validate_ram_directory(path: &Path) -> Result<(), String> {
    use std::{io::Write, os::fd::AsRawFd};

    if !path.is_dir() {
        return Err(format!(
            "RAM storage directory does not exist or is not a directory: {}",
            path.display()
        ));
    }
    if !is_ram_backed(path)? {
        return Err(format!(
            "RAM storage directory {} is not on Linux tmpfs; select a tmpfs mount with --ram-dir or POCKET_IC_RAM_DIR",
            path.display()
        ));
    }

    let mut probe = tempfile::Builder::new()
        .prefix(".pic-probe-")
        .tempfile_in(path)
        .map_err(|err| format!("Cannot write to RAM storage {}: {err}", path.display()))?;
    // A real write also detects full tmpfs mounts before server initialization.
    probe
        .write_all(&[0; 4096])
        .map_err(|err| format!("Cannot write to RAM storage {}: {err}", path.display()))?;
    // SAFETY: the file contains 4096 initialized bytes; mmap does not run code.
    let mapping = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            4096,
            libc::PROT_READ | libc::PROT_EXEC,
            libc::MAP_PRIVATE,
            probe.as_raw_fd(),
            0,
        )
    };
    if mapping == libc::MAP_FAILED {
        return Err(format!(
            "RAM storage {} does not support executable file mappings required by Wasmtime: {}; use a tmpfs mount without noexec",
            path.display(),
            std::io::Error::last_os_error()
        ));
    }
    // SAFETY: mapping is the successful mmap result with the same length.
    if unsafe { libc::munmap(mapping, 4096) } != 0 {
        return Err(format!(
            "Cannot release RAM storage probe at {}: {}",
            path.display(),
            std::io::Error::last_os_error()
        ));
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn validate_ram_directory(_path: &Path) -> Result<(), String> {
    Err("PocketIC RAM-backed storage currently requires Linux tmpfs".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ram_flag_requires_an_explicit_valid_boolean() {
        assert_eq!(parse_ram_flag(None), Ok(false));
        for value in ["1", "true"] {
            assert_eq!(parse_ram_flag(Some(OsStr::new(value))), Ok(true));
        }
        for value in ["0", "false"] {
            assert_eq!(parse_ram_flag(Some(OsStr::new(value))), Ok(false));
        }
        for value in ["", "yes", "TRUE", " 1", "false "] {
            assert!(parse_ram_flag(Some(OsStr::new(value))).is_err());
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn missing_ram_directory_is_rejected() {
        let parent = tempfile::tempdir().unwrap();
        let missing = parent.path().join("missing");
        assert!(
            validate_ram_directory(&missing)
                .unwrap_err()
                .contains("does not exist")
        );
        assert!(!missing.exists());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn disk_directory_cannot_silently_be_used_for_ram_storage() {
        let parent = tempfile::tempdir().unwrap();
        // Some test hosts put their entire temporary directory on tmpfs.
        if is_ram_backed(parent.path()).unwrap() {
            return;
        }
        let error = new_ram_tempdir_in(parent.path()).unwrap_err();
        assert!(error.contains("not on Linux tmpfs"), "{error}");
        assert_eq!(std::fs::read_dir(parent.path()).unwrap().count(), 0);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn ram_workspace_is_private_and_removed_on_drop() {
        use std::os::unix::fs::PermissionsExt;

        let parent = Path::new("/dev/shm");
        // Other hosts may omit /dev/shm or mount it noexec. Explicit RAM mode
        // returns that validation error; the portable test suite may still run.
        if validate_ram_directory(parent).is_err() {
            return;
        }
        let workspace = new_ram_tempdir_in(parent).unwrap();
        let path = workspace.path().to_path_buf();
        assert!(is_ram_backed(&path).unwrap());
        assert_eq!(path.metadata().unwrap().permissions().mode() & 0o777, 0o700);
        std::fs::write(path.join("state"), b"canister state").unwrap();
        drop(workspace);
        assert!(!path.exists());
    }
}
