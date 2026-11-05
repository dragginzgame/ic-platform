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
