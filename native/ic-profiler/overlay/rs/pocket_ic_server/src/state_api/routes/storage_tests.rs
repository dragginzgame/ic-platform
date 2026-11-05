use super::validate_instance_state_storage;
use pocket_ic::common::storage;

#[test]
fn default_state_and_normal_mode_preserve_existing_path_semantics() {
    assert!(validate_instance_state_storage(None, true).is_ok());
    assert!(
        validate_instance_state_storage(Some(std::path::Path::new("missing/state")), false).is_ok()
    );
}

#[cfg(target_os = "linux")]
#[test]
fn ram_mode_rejects_disk_destinations_before_creating_state() {
    let disk = std::env::current_dir().unwrap();
    if storage::is_ram_backed(&disk).unwrap() {
        return;
    }
    assert!(
        validate_instance_state_storage(Some(&disk), true)
            .unwrap_err()
            .contains("tmpfs")
    );
    let fresh = disk
        .join("pocket-ic-storage-test-do-not-create")
        .join("state");
    assert!(
        validate_instance_state_storage(Some(&fresh), true)
            .unwrap_err()
            .contains("tmpfs")
    );
    assert!(!fresh.exists());
}

#[cfg(target_os = "linux")]
#[test]
fn ram_mode_accepts_new_tmpfs_destinations_and_rejects_disk_symlinks() {
    let Ok(root) = storage::new_ram_tempdir() else {
        return;
    };
    let fresh = root.path().join("new").join("state");
    assert!(validate_instance_state_storage(Some(&fresh), true).is_ok());
    assert!(!fresh.exists());
    let disk = std::env::current_dir().unwrap();
    if !storage::is_ram_backed(&disk).unwrap() {
        let link = root.path().join("disk-link");
        std::os::unix::fs::symlink(&disk, &link).unwrap();
        assert!(validate_instance_state_storage(Some(&link), true).is_err());
    }
}
