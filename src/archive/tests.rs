use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use super::commands::safe_relative;
use super::*;

const SEVEN_Z_SAMPLE: &str = "Path = sub\n\
Size = 0\n\
Packed Size = 0\n\
Modified = 2026-11-22 15:04:49.9963601\n\
Attributes = D drwxr-xr-x\n\
CRC = \n\
Encrypted = -\n\
Method = \n\
Block = \n\
\n\
Path = sub/a.txt\n\
Size = 12\n\
Packed Size = 200082\n\
Modified = 2024-03-04 05:06:07.0000000\n\
Attributes = A -rw-r--r--\n\
CRC = AF083B2D\n\
Encrypted = -\n\
Method = LZMA2:18\n\
Block = 0\n\
\n";

#[test]
fn parses_7z_slt_listing() {
    let entries = parse_slt(SEVEN_Z_SAMPLE).unwrap();
    assert_eq!(entries.len(), 2);
    assert!(entries[0].is_dir);
    assert_eq!(entries[0].display_path, "sub");
    assert_eq!(entries[1].display_path, "sub/a.txt");
    assert_eq!(entries[1].size, 12);
    assert_eq!(entries[1].packed, Some(200082));
    assert_eq!(entries[1].modified.as_deref(), Some("2024-03-04 05:06:07"));
    assert!(!entries[1].encrypted);
}

#[test]
fn parses_tar_symlink_and_skips_dot() {
    let sample = "Path = .\nFolder = +\nSize = 0\nAttributes = D drwxr-xr-x\n\n\
Path = ./link\nFolder = -\nSize = 5\nSymbolic Link = a.txt\nMode = lrwxrwxrwx\n\n";
    let entries = parse_slt(sample).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].display_path, "link");
    assert!(entries[0].is_symlink);
    assert_eq!(entries[0].symlink_target.as_deref(), Some("a.txt"));
}

#[test]
fn detects_compound_extensions() {
    assert_eq!(Format::detect(Path::new("x.tar.gz")), Format::TarGz);
    assert_eq!(Format::detect(Path::new("x.TGZ")), Format::TarGz);
    assert_eq!(Format::detect(Path::new("x.tar.zst")), Format::TarZst);
    assert_eq!(Format::detect(Path::new("x.zip")), Format::Zip);
    assert_eq!(Format::detect(Path::new("x.7z.001")), Format::SevenZ);
    assert_eq!(Format::detect(Path::new("x.zip.001")), Format::Zip);
    assert_eq!(Format::detect(Path::new("x.rar")), Format::Rar);
    assert_eq!(Format::detect(Path::new("x.r00")), Format::Rar);
    assert_eq!(Format::detect(Path::new("x.part01.rar")), Format::Rar);
    assert_eq!(Format::detect(Path::new("x.bin")), Format::Other);
}

#[test]
fn builds_directory_children_and_synthesizes_dirs() {
    let entries = vec![
        Entry {
            raw_path: "a.txt".into(),
            display_path: "a.txt".into(),
            is_dir: false,
            is_symlink: false,
            symlink_target: None,
            size: 1,
            packed: None,
            modified: None,
            method: None,
            encrypted: false,
        },
        Entry {
            raw_path: "sub/deep/c.txt".into(),
            display_path: "sub/deep/c.txt".into(),
            is_dir: false,
            is_symlink: false,
            symlink_target: None,
            size: 1,
            packed: None,
            modified: None,
            method: None,
            encrypted: false,
        },
    ];
    let root = children(&entries, "");
    assert_eq!(root.len(), 2);
    assert_eq!(root[0].name, "sub");
    assert!(root[0].is_dir);
    let sub = children(&entries, "sub");
    assert_eq!(sub.len(), 2);
    assert!(sub[0].is_parent);
    assert_eq!(sub[0].name, "..");
    assert_eq!(sub[1].name, "deep");
    assert!(sub[1].is_dir);
    let deep = children(&entries, "sub/deep");
    assert_eq!(deep.len(), 2);
    assert!(deep[0].is_parent);
    assert!(!deep[1].is_dir);
    assert_eq!(parent_inner("sub/deep"), "sub");
    assert_eq!(parent_inner("sub"), "");
}

#[test]
fn rejects_unsafe_tar_paths() {
    assert_eq!(safe_relative("./a/b"), Some(PathBuf::from("a/b")));
    assert_eq!(safe_relative("../etc/passwd"), None);
    assert_eq!(safe_relative("/etc/passwd"), None);
    assert_eq!(safe_relative("."), None);
}

#[test]
fn recognizes_archive_extensions() {
    assert!(is_archive_name("foo.zip"));
    assert!(is_archive_name("foo.tar.gz"));
    assert!(is_archive_name("FOO.7Z"));
    assert!(is_archive_name("foo.7z.001"));
    assert!(is_archive_name("foo.rar"));
    assert!(is_archive_name("foo.r00"));
    assert!(!is_archive_name("foo.txt"));
    assert!(!is_archive_name("foo"));
}

fn run(mut cmd: Command) {
    let status = cmd.status().expect("tool should start");
    assert!(status.success(), "command failed: {cmd:?}");
}

fn run_password(cmd: Command, password: &str) {
    let output = output_with_password(cmd, Some(password)).expect("tool should start");
    assert!(
        output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn backend_roundtrip_for_7z_zip_and_targz() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("7zrn-test-{}-{stamp}", std::process::id()));
    let source = root.join("src");
    std::fs::create_dir_all(source.join("sub")).unwrap();
    std::fs::write(source.join("a.txt"), "alpha").unwrap();
    std::fs::write(source.join("sub/b.txt"), "beta").unwrap();

    let job = crate::job::new_job(crate::job::JobKind::Add, "test");

    let formats = [
        ("test.7z", NewFormat::SevenZ),
        ("test.zip", NewFormat::Zip),
        ("test.tar.gz", NewFormat::TarGz),
    ];
    for (name, format) in formats {
        let output = root.join(name);
        let names = vec![OsString::from("a.txt"), OsString::from("sub")];
        run(create_command(&output, format, &source, &names, &CreateOptions::default()).unwrap());
        let listing = list_archive(&output, None).unwrap();
        let detected = Format::detect(&output);
        let names: Vec<&str> = listing
            .entries
            .iter()
            .map(|entry| entry.display_path.as_str())
            .collect();
        assert!(names.contains(&"a.txt"), "{name}: {names:?}");
        assert!(names.contains(&"sub/b.txt"), "{name}: {names:?}");

        let dest = root.join(format!("out-{name}"));
        let extract = extract_command(&output, detected, &dest, &[], None).unwrap();
        std::fs::create_dir_all(&dest).unwrap();
        run(extract);
        assert_eq!(
            std::fs::read_to_string(dest.join("sub/b.txt")).unwrap(),
            "beta"
        );

        if name.ends_with(".tar.gz") {
            tar_add(&output, detected, &[source.join("a.txt")], false, &job).unwrap();
            tar_delete(&output, detected, &["sub".to_string()], false, &job).unwrap();
            let listing = list_archive(&output, None).unwrap();
            let names: Vec<&str> = listing
                .entries
                .iter()
                .map(|entry| entry.display_path.as_str())
                .collect();
            assert!(names.contains(&"a.txt"));
            assert!(!names.iter().any(|path| path.starts_with("sub")));
        } else {
            for cmd in add_commands(&output, &[source.join("sub/b.txt")], None, 5).unwrap() {
                run(cmd);
            }
            let mut delete = delete_command(
                &output,
                &[display_archive_path("sub/b.txt").to_string()],
                None,
            )
            .unwrap();
            run_capture_test(&mut delete);
            let listing = list_archive(&output, None).unwrap();
            assert!(
                !listing
                    .entries
                    .iter()
                    .any(|entry| entry.display_path == "sub/b.txt"),
                "{name}: b.txt should be deleted"
            );
        }
    }
    let _ = std::fs::remove_dir_all(&root);
}

fn run_capture_test(cmd: &mut Command) {
    let output = cmd.output().expect("tool should start");
    assert!(
        output.status.success(),
        "command failed: {cmd:?}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn edits_archive_entries() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("7zrn-edit-{}-{stamp}", std::process::id()));
    let source = root.join("src");
    std::fs::create_dir_all(source.join("sub")).unwrap();
    std::fs::write(source.join("a.txt"), "alpha").unwrap();
    std::fs::write(source.join("sub/b.txt"), "beta").unwrap();
    let job = crate::job::new_job(crate::job::JobKind::Modify, "edit test");
    for (name, format) in [
        ("edit.7z", NewFormat::SevenZ),
        ("edit.zip", NewFormat::Zip),
        ("edit.tar.gz", NewFormat::TarGz),
    ] {
        let archive = root.join(name);
        let names = vec![OsString::from("a.txt"), OsString::from("sub")];
        run(create_command(&archive, format, &source, &names, &CreateOptions::default()).unwrap());
        let detected = Format::detect(&archive);
        apply_edit(
            &archive,
            detected,
            ArchiveEdit::Rename {
                from: "sub/b.txt".into(),
                to: "sub/c.txt".into(),
            },
            None,
            5,
            false,
            &job,
        )
        .unwrap();
        let listing = list_archive(&archive, None).unwrap();
        let paths: Vec<&str> = listing
            .entries
            .iter()
            .map(|entry| entry.display_path.as_str())
            .collect();
        assert!(paths.contains(&"sub/c.txt"), "{name}: {paths:?}");
        assert!(!paths.contains(&"sub/b.txt"), "{name}: {paths:?}");

        apply_edit(
            &archive,
            detected,
            ArchiveEdit::Copy {
                keys: vec!["a.txt".into()],
                dest: "sub".into(),
            },
            None,
            5,
            false,
            &job,
        )
        .unwrap();
        let listing = list_archive(&archive, None).unwrap();
        assert!(
            listing
                .entries
                .iter()
                .any(|entry| entry.display_path == "sub/a.txt"),
            "{name}: copy failed"
        );

        apply_edit(
            &archive,
            detected,
            ArchiveEdit::Move {
                keys: vec!["a.txt".into()],
                dest: "moved".into(),
            },
            None,
            5,
            false,
            &job,
        )
        .unwrap();
        let listing = list_archive(&archive, None).unwrap();
        let paths: Vec<&str> = listing
            .entries
            .iter()
            .map(|entry| entry.display_path.as_str())
            .collect();
        assert!(paths.contains(&"moved/a.txt"), "{name}: {paths:?}");
        assert!(!paths.contains(&"a.txt"), "{name}: {paths:?}");

        apply_edit(
            &archive,
            detected,
            ArchiveEdit::NewFolder {
                path: "empty/nested".into(),
            },
            None,
            5,
            false,
            &job,
        )
        .unwrap();
        let listing = list_archive(&archive, None).unwrap();
        assert!(
            listing
                .entries
                .iter()
                .any(|entry| entry.is_dir && entry.display_path == "empty/nested"),
            "{name}: new folder failed"
        );
    }
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn create_options_apply_password_and_excludes() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("7zrn-options-{}-{stamp}", std::process::id()));
    let source = root.join("src");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::write(source.join("keep.txt"), "keep").unwrap();
    std::fs::write(source.join("drop.tmp"), "drop").unwrap();
    let archive = root.join("options.7z");
    let options = CreateOptions {
        compression: 9,
        password: Some("secret".into()),
        split: None,
        excludes: vec!["*.tmp".into()],
    };
    run_password(
        create_command(
            &archive,
            NewFormat::SevenZ,
            &source,
            &[OsString::from(".")],
            &options,
        )
        .unwrap(),
        "secret",
    );
    let listing = list_archive(&archive, Some("secret")).unwrap();
    assert!(
        listing
            .entries
            .iter()
            .any(|entry| entry.display_path == "keep.txt")
    );
    assert!(
        !listing
            .entries
            .iter()
            .any(|entry| entry.display_path == "drop.tmp")
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn encrypted_archive_requires_password() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("7zrn-enc-{}-{stamp}", std::process::id()));
    let source = root.join("src");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::write(source.join("secret.txt"), "classified").unwrap();
    let archive = root.join("enc.7z");
    let mut create = Command::new(SEVEN_ZIP);
    create
        .args(["a", "-y", "-psecret", "-mhe=on"])
        .arg("--")
        .arg(&archive)
        .arg("secret.txt")
        .current_dir(&source);
    run(create);

    let missing = list_archive(&archive, None).unwrap_err();
    assert!(password_required(&missing), "{missing}");
    let wrong = list_archive(&archive, Some("wrong")).unwrap_err();
    assert!(password_required(&wrong), "{wrong}");

    let listing = list_archive(&archive, Some("secret")).unwrap();
    assert!(
        listing
            .entries
            .iter()
            .any(|entry| entry.display_path == "secret.txt")
    );

    let dest = root.join("out");
    std::fs::create_dir_all(&dest).unwrap();
    let extract = extract_command(&archive, Format::SevenZ, &dest, &[], Some("secret")).unwrap();
    run_password(extract, "secret");
    assert_eq!(
        std::fs::read_to_string(dest.join("secret.txt")).unwrap(),
        "classified"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn finds_nested_extracted_file() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("7zrn-extracted-{}-{stamp}", std::process::id()));
    let nested = root.join("sub/deep");
    std::fs::create_dir_all(&nested).unwrap();
    let file = nested.join("file.txt");
    std::fs::write(&file, b"content").unwrap();
    assert_eq!(
        find_extracted(&root, "sub/deep/file.txt"),
        Some(file.clone())
    );
    assert_eq!(find_extracted(&root, "file.txt"), Some(file));
    assert_eq!(find_extracted(&root, "../file.txt"), None);
    #[cfg(unix)]
    {
        let link = root.join("outside");
        std::os::unix::fs::symlink("/etc/passwd", &link).unwrap();
        assert_eq!(find_extracted(&root, "outside"), None);
        std::fs::remove_file(link).unwrap();
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn detects_split_volume_names() {
    assert!(is_split_volume(Path::new("data.7z.001")));
    assert!(is_split_volume(Path::new("data.zip.001")));
    assert!(!is_split_volume(Path::new("data.7z")));
    assert!(!is_split_volume(Path::new("data.7z.002")));
}

#[test]
fn create_commands_do_not_expose_passwords() {
    let options = CreateOptions {
        compression: 5,
        password: Some("topsecret".into()),
        split: None,
        excludes: Vec::new(),
    };
    let command = create_command(
        Path::new("/tmp/example.7z"),
        NewFormat::SevenZ,
        Path::new("/tmp"),
        &[OsString::from("file.txt")],
        &options,
    )
    .unwrap();
    assert!(!format!("{command:?}").contains("topsecret"));
}
