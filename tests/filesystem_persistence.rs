use mem8_fs_lite::Mem8Fs;
use tempfile::tempdir;

#[test]
fn reads_persisted_bytes_after_reopening() {
    let root = tempdir().unwrap();
    let bytes: Vec<u8> = (0..=255).collect();
    {
        let fs = Mem8Fs::new(root.path()).unwrap();
        fs.write("all-bytes.bin", &bytes).unwrap();
        fs.write("empty.bin", b"").unwrap();
    }
    let reopened = Mem8Fs::new(root.path()).unwrap();
    assert_eq!(reopened.read("all-bytes.bin").unwrap(), bytes);
    assert_eq!(reopened.read("empty.bin").unwrap(), b"");
}

#[test]
fn reads_data_written_by_another_process() {
    let root = tempdir().unwrap();
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "subprocess_writer", "--ignored"])
        .env("MEM8_TEST_ROOT", root.path())
        .status()
        .unwrap();
    assert!(status.success());
    let fs = Mem8Fs::new(root.path()).unwrap();
    assert_eq!(
        fs.read("child.bin").unwrap(),
        (0..10000).map(|i| i as u8).collect::<Vec<_>>()
    );
}

#[test]
#[ignore = "invoked by the fresh-process regression with a temporary directory"]
fn subprocess_writer() {
    let root = std::env::var_os("MEM8_TEST_ROOT").unwrap();
    Mem8Fs::new(root)
        .unwrap()
        .write(
            "child.bin",
            &(0..10000).map(|i| i as u8).collect::<Vec<_>>(),
        )
        .unwrap();
}

#[test]
fn overwrite_delete_and_append_after_a_cold_read_survive_reopen() {
    let root = tempdir().unwrap();
    {
        let fs = Mem8Fs::new(root.path()).unwrap();
        fs.write("updated", b"old").unwrap();
        fs.write("removed", b"delete me").unwrap();
    }
    {
        let fs = Mem8Fs::new(root.path()).unwrap();
        assert_eq!(fs.read("updated").unwrap(), b"old");
        fs.write("updated", b"new").unwrap();
        fs.delete("removed").unwrap();
        fs.copy("updated", "copy").unwrap();
    }
    let fs = Mem8Fs::new(root.path()).unwrap();
    assert_eq!(fs.read("updated").unwrap(), b"new");
    assert_eq!(fs.read("copy").unwrap(), b"new");
    assert!(!fs.exists("removed"));
    assert!(fs.read("removed").is_err());
}

#[test]
fn empty_filesystem_reopens_including_legacy_empty_index() {
    let root = tempdir().unwrap();
    drop(Mem8Fs::new(root.path()).unwrap());
    drop(Mem8Fs::new(root.path()).unwrap());
    std::fs::write(root.path().join(".mem8/index.m8"), []).unwrap();
    assert!(Mem8Fs::new(root.path())
        .unwrap()
        .list("/")
        .unwrap()
        .is_empty());
}

#[test]
fn rejects_truncated_records_and_forged_lengths_without_repairing_originals() {
    for len in [1, 35, 36, 51] {
        let root = tempdir().unwrap();
        Mem8Fs::new(root.path())
            .unwrap()
            .write("one", b"x")
            .unwrap();
        let path = root.path().join(".mem8/data.m8");
        let mut bytes = std::fs::read(&path).unwrap();
        bytes.truncate(len);
        std::fs::write(&path, &bytes).unwrap();
        assert!(Mem8Fs::new(root.path()).is_err());
        assert_eq!(std::fs::read(path).unwrap(), bytes);
    }
    let root = tempdir().unwrap();
    Mem8Fs::new(root.path())
        .unwrap()
        .write("one", b"x")
        .unwrap();
    let path = root.path().join(".mem8/data.m8");
    let mut bytes = std::fs::read(&path).unwrap();
    bytes[32..36].copy_from_slice(&u32::MAX.to_be_bytes());
    std::fs::write(path, bytes).unwrap();
    assert!(Mem8Fs::new(root.path()).is_err());
}

#[test]
fn rejects_modified_and_nonfinite_wave_payloads() {
    for value in [0.25_f64, f64::NAN, f64::INFINITY] {
        let root = tempdir().unwrap();
        Mem8Fs::new(root.path())
            .unwrap()
            .write("one", b"x")
            .unwrap();
        let path = root.path().join(".mem8/data.m8");
        let mut bytes = std::fs::read(&path).unwrap();
        bytes[36..44].copy_from_slice(&value.to_be_bytes());
        std::fs::write(path, bytes).unwrap();
        assert!(Mem8Fs::new(root.path()).unwrap().read("one").is_err());
    }
}

#[test]
fn rejects_missing_records_and_empty_index_with_existing_data() {
    let root = tempdir().unwrap();
    Mem8Fs::new(root.path())
        .unwrap()
        .write("one", b"x")
        .unwrap();
    let path = root.path().join(".mem8/data.m8");
    let mut bytes = std::fs::read(&path).unwrap();
    bytes[0] ^= 1;
    std::fs::write(path, bytes).unwrap();
    assert!(Mem8Fs::new(root.path()).unwrap().read("one").is_err());
    std::fs::write(root.path().join(".mem8/index.m8"), []).unwrap();
    assert!(Mem8Fs::new(root.path()).is_err());
}
