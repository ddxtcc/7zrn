use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::job::{self, SharedJob};

use super::{Format, NewFormat, SEVEN_ZIP, TAR};

pub(crate) fn output_with_password(
    mut cmd: Command,
    password: Option<&str>,
) -> std::io::Result<std::process::Output> {
    use std::io::Write;
    cmd.stdin(if password.is_some() {
        std::process::Stdio::piped()
    } else {
        std::process::Stdio::null()
    })
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped());
    let mut child = cmd.spawn()?;
    if let Some(password) = password
        && let Some(mut stdin) = child.stdin.take()
    {
        let _ = stdin.write_all(password.as_bytes());
        let _ = stdin.write_all(b"\n");
    }
    child.wait_with_output()
}
pub fn extract_command(
    archive: &Path,
    format: Format,
    dest: &Path,
    raw_names: &[String],
    _password: Option<&str>,
) -> Result<Command, String> {
    if format.is_tar_family() {
        let mut cmd = Command::new(TAR);
        cmd.arg("-xf").arg(archive).arg("-C").arg(dest);
        if !raw_names.is_empty() {
            cmd.arg("--");
            for name in raw_names {
                cmd.arg(name);
            }
        }
        Ok(cmd)
    } else {
        let mut cmd = Command::new(SEVEN_ZIP);
        cmd.args(["x", "-y", "-bsp1", "-bso1", "-bse1", "-spd", "-sccUTF-8"]);
        let mut output_arg = OsString::from("-o");
        output_arg.push(dest.as_os_str());
        cmd.arg(output_arg).arg("--").arg(archive);
        for name in raw_names {
            cmd.arg(name);
        }
        Ok(cmd)
    }
}

pub fn add_commands(
    archive: &Path,
    paths: &[PathBuf],
    password: Option<&str>,
    compression: u8,
) -> Result<Vec<Command>, String> {
    let format = Format::detect(archive);
    if is_split_volume(archive) {
        return Err("Split-volume archives cannot be modified. Merge the volumes first.".into());
    }
    if format.is_tar_family() {
        return Err("tar archives are updated by repacking".into());
    }
    if !matches!(format, Format::SevenZ | Format::Zip) {
        return Err(format!("{} archives are read-only", format.label()));
    }
    let archive_canonical = fs::canonicalize(archive).unwrap_or_else(|_| archive.to_path_buf());
    let mut groups: BTreeMap<PathBuf, Vec<OsString>> = BTreeMap::new();
    for path in paths {
        if fs::canonicalize(path).ok().as_deref() == Some(archive_canonical.as_path()) {
            continue;
        }
        let parent = path
            .parent()
            .unwrap_or_else(|| Path::new("/"))
            .to_path_buf();
        let Some(name) = path.file_name() else {
            continue;
        };
        groups.entry(parent).or_default().push(name.to_os_string());
    }
    let mut commands = Vec::new();
    for (parent, names) in groups {
        let mut cmd = Command::new(SEVEN_ZIP);
        cmd.args(["a", "-y", "-bsp1", "-bso1", "-bse1", "-spd", "-sccUTF-8"]);
        cmd.arg(format!("-mx={}", compression.min(9)));
        if password.is_some() && format == Format::Zip {
            cmd.arg("-mem=AES256");
        }
        cmd.arg("--").arg(archive);
        for name in names {
            cmd.arg(name);
        }
        cmd.current_dir(parent);
        commands.push(cmd);
    }
    if commands.is_empty() {
        Err("Nothing to add (the archive itself was selected).".into())
    } else {
        Ok(commands)
    }
}

pub fn delete_command(
    archive: &Path,
    raw_names: &[String],
    _password: Option<&str>,
) -> Result<Command, String> {
    let format = Format::detect(archive);
    if is_split_volume(archive) {
        return Err("Split-volume archives cannot be modified. Merge the volumes first.".into());
    }
    if !matches!(format, Format::SevenZ | Format::Zip) {
        return Err(format!(
            "{} archives are updated by repacking",
            format.label()
        ));
    }
    if raw_names.is_empty() {
        return Err("No entries selected".into());
    }
    let mut cmd = Command::new(SEVEN_ZIP);
    cmd.args(["d", "-y", "-bsp1", "-bso1", "-bse1", "-spd", "-sccUTF-8"]);
    cmd.arg("--").arg(archive);
    for name in raw_names {
        cmd.arg(name);
    }
    Ok(cmd)
}

pub fn test_command(archive: &Path, format: Format, _password: Option<&str>) -> Command {
    if format.is_tar_family() {
        let mut cmd = Command::new(TAR);
        cmd.arg("-tf").arg(archive);
        cmd
    } else {
        let mut cmd = Command::new(SEVEN_ZIP);
        cmd.args(["t", "-y", "-bsp1", "-bso1", "-bse1"]);
        cmd.arg("--").arg(archive);
        cmd
    }
}

pub fn create_command(
    output: &Path,
    format: NewFormat,
    cwd: &Path,
    names: &[OsString],
    options: &CreateOptions,
) -> Result<Command, String> {
    if names.is_empty() {
        return Err("Nothing to archive".into());
    }
    if let Some(kind) = format.seven_zip_type() {
        if options.split.is_some() && format != NewFormat::SevenZ {
            return Err("Split volumes are only supported for 7z archives.".into());
        }
        let mut cmd = Command::new(SEVEN_ZIP);
        cmd.args(["a", "-y", "-bsp1", "-bso1", "-bse1", "-sccUTF-8"])
            .arg(format!("-t{kind}"))
            .arg(format!("-mx={}", options.compression.min(9)));
        if options
            .password
            .as_deref()
            .is_some_and(|value| !value.is_empty())
        {
            cmd.arg("-p");
            if format == NewFormat::SevenZ {
                cmd.arg("-mhe=on");
            } else if format == NewFormat::Zip {
                cmd.arg("-mem=AES256");
            }
        }
        if let Some(split) = options.split.as_deref().filter(|value| !value.is_empty()) {
            cmd.arg(format!("-v{split}"));
        }
        for exclude in options.excludes.iter().filter(|value| !value.is_empty()) {
            cmd.arg(format!("-x!{exclude}"));
        }
        cmd.arg("--").arg(output);
        for name in names {
            cmd.arg(name);
        }
        cmd.current_dir(cwd);
        Ok(cmd)
    } else {
        if options
            .password
            .as_deref()
            .is_some_and(|value| !value.is_empty())
        {
            return Err("Passwords are only supported for 7z and zip archives.".into());
        }
        if options
            .split
            .as_deref()
            .is_some_and(|value| !value.is_empty())
        {
            return Err("Split volumes are only supported for 7z archives.".into());
        }
        let mut cmd = Command::new(TAR);
        cmd.arg("-caf").arg(output);
        for exclude in options.excludes.iter().filter(|value| !value.is_empty()) {
            cmd.arg(format!("--exclude={exclude}"));
        }
        cmd.arg("-C").arg(cwd).arg("--");
        for name in names {
            cmd.arg(name);
        }
        Ok(cmd)
    }
}

pub fn tar_add(
    archive: &Path,
    format: Format,
    paths: &[PathBuf],
    backup: bool,
    job: &SharedJob,
) -> Result<(), String> {
    if !format.is_tar_family() {
        return Err("Not a tar archive".into());
    }
    let archive_canonical = fs::canonicalize(archive).unwrap_or_else(|_| archive.to_path_buf());
    let paths: Vec<PathBuf> = paths
        .iter()
        .filter(|path| fs::canonicalize(path).ok().as_deref() != Some(archive_canonical.as_path()))
        .cloned()
        .collect();
    if paths.is_empty() {
        return Err("Nothing to add (the archive itself was selected).".into());
    }
    let count = paths.len();
    repack(archive, format, backup, job, move |root| {
        let mut cmd = Command::new("cp");
        cmd.arg("-a").arg("--");
        for path in &paths {
            cmd.arg(path);
        }
        cmd.arg(root);
        job::note(job, format!("Copying {count} item(s) into the archive"));
        job::run_command(&mut cmd, job)
    })
}

pub fn tar_delete(
    archive: &Path,
    format: Format,
    display_paths: &[String],
    backup: bool,
    job: &SharedJob,
) -> Result<(), String> {
    if !format.is_tar_family() {
        return Err("Not a tar archive".into());
    }
    if display_paths.is_empty() {
        return Err("No entries selected".into());
    }
    repack(archive, format, backup, job, |root| {
        for path in display_paths {
            let Some(relative) = safe_relative(path) else {
                continue;
            };
            let target = root.join(&relative);
            let Ok(meta) = fs::symlink_metadata(&target) else {
                continue;
            };
            job::note(job, format!("Removing {}", relative.display()));
            if meta.is_dir() && !meta.file_type().is_symlink() {
                fs::remove_dir_all(&target)
                    .map_err(|e| format!("Could not remove {}: {e}", target.display()))?;
            } else {
                fs::remove_file(&target)
                    .map_err(|e| format!("Could not remove {}: {e}", target.display()))?;
            }
        }
        Ok(())
    })
}

#[derive(Clone, Debug)]
pub struct CreateOptions {
    pub compression: u8,
    pub password: Option<String>,
    pub split: Option<String>,
    pub excludes: Vec<String>,
}

impl Default for CreateOptions {
    fn default() -> Self {
        Self {
            compression: 5,
            password: None,
            split: None,
            excludes: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ArchiveEdit {
    Rename { from: String, to: String },
    Copy { keys: Vec<String>, dest: String },
    Move { keys: Vec<String>, dest: String },
    NewFolder { path: String },
}

pub fn apply_edit(
    archive: &Path,
    format: Format,
    edit: ArchiveEdit,
    password: Option<&str>,
    compression: u8,
    backup: bool,
    job: &SharedJob,
) -> Result<(), String> {
    if is_split_volume(archive) {
        return Err("Split-volume archives cannot be modified. Merge the volumes first.".into());
    }
    if !format.can_modify() {
        return Err(format!("{} archives are read-only", format.label()));
    }
    repack_any(
        archive,
        format,
        password,
        compression,
        backup,
        job,
        |root| apply_edit_in(root, edit, job),
    )
}

fn apply_edit_in(root: &Path, edit: ArchiveEdit, job: &SharedJob) -> Result<(), String> {
    match edit {
        ArchiveEdit::Rename { from, to } => {
            let source = safe_relative(&from).ok_or_else(|| "Invalid source path".to_string())?;
            let target = safe_relative(&to).ok_or_else(|| "Invalid target path".to_string())?;
            ensure_source(root, &source)?;
            if root.join(&target).exists() {
                return Err(format!("{} already exists", target.display()));
            }
            job::note(
                job,
                format!("Renaming {} to {}", source.display(), target.display()),
            );
            move_path(root, &source, &target)
        }
        ArchiveEdit::Copy { keys, dest } => copy_or_move(root, &keys, &dest, false, job),
        ArchiveEdit::Move { keys, dest } => copy_or_move(root, &keys, &dest, true, job),
        ArchiveEdit::NewFolder { path } => {
            let relative = safe_relative(&path).ok_or_else(|| "Invalid folder path".to_string())?;
            let target = root.join(&relative);
            if target.exists() {
                return Err(format!("{} already exists", relative.display()));
            }
            job::note(job, format!("Creating folder {}", relative.display()));
            fs::create_dir_all(&target)
                .map_err(|error| format!("Could not create {}: {error}", relative.display()))
        }
    }
}

fn copy_or_move(
    root: &Path,
    keys: &[String],
    dest: &str,
    remove: bool,
    job: &SharedJob,
) -> Result<(), String> {
    let folder = if dest.trim().is_empty() {
        PathBuf::new()
    } else {
        safe_relative(dest).ok_or_else(|| "Invalid destination folder".to_string())?
    };
    let mut sources: Vec<PathBuf> = keys.iter().filter_map(|key| safe_relative(key)).collect();
    sources.sort();
    let mut selected = Vec::new();
    for source in sources {
        if selected
            .iter()
            .any(|parent: &PathBuf| source.starts_with(parent))
        {
            continue;
        }
        selected.push(source);
    }
    if selected.is_empty() {
        return Err("No entries selected".into());
    }
    for source in selected {
        ensure_source(root, &source)?;
        let Some(name) = source.file_name() else {
            continue;
        };
        let target = folder.join(name);
        if target == source {
            return Err(format!("{} is already in that folder", source.display()));
        }
        if root.join(&source).is_dir() && target.starts_with(&source) {
            return Err(format!("Cannot place {} inside itself", source.display()));
        }
        if root.join(&target).exists() {
            return Err(format!("{} already exists", target.display()));
        }
        job::note(
            job,
            format!(
                "{} {} to {}",
                if remove { "Moving" } else { "Copying" },
                source.display(),
                folder.display()
            ),
        );
        copy_path(root, &source, &target)?;
        if remove {
            remove_path(root, &source)?;
        }
    }
    Ok(())
}

fn ensure_source(root: &Path, relative: &Path) -> Result<(), String> {
    fs::symlink_metadata(root.join(relative))
        .map(|_| ())
        .map_err(|_| format!("{} does not exist", relative.display()))
}

fn copy_path(root: &Path, source: &Path, target: &Path) -> Result<(), String> {
    copy_entry(&root.join(source), &root.join(target))
}

fn copy_entry(source: &Path, target: &Path) -> Result<(), String> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Could not create {}: {error}", parent.display()))?;
    }
    let meta = fs::symlink_metadata(source)
        .map_err(|error| format!("Could not read {}: {error}", source.display()))?;
    if meta.file_type().is_symlink() {
        let link = fs::read_link(source)
            .map_err(|error| format!("Could not read link {}: {error}", source.display()))?;
        std::os::unix::fs::symlink(link, target)
            .map_err(|error| format!("Could not create link {}: {error}", target.display()))?;
    } else if meta.is_dir() {
        fs::create_dir_all(target)
            .map_err(|error| format!("Could not create {}: {error}", target.display()))?;
        let entries = fs::read_dir(source)
            .map_err(|error| format!("Could not read {}: {error}", source.display()))?;
        for entry in entries {
            let entry = entry.map_err(|error| format!("Could not read directory: {error}"))?;
            copy_entry(&entry.path(), &target.join(entry.file_name()))?;
        }
    } else {
        fs::copy(source, target).map_err(|error| {
            format!(
                "Could not copy {} to {}: {error}",
                source.display(),
                target.display()
            )
        })?;
    }
    Ok(())
}

fn move_path(root: &Path, source: &Path, target: &Path) -> Result<(), String> {
    copy_path(root, source, target)?;
    remove_path(root, source)
}

fn remove_path(root: &Path, relative: &Path) -> Result<(), String> {
    let path = root.join(relative);
    let meta = fs::symlink_metadata(&path)
        .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
    if meta.is_dir() && !meta.file_type().is_symlink() {
        fs::remove_dir_all(&path)
            .map_err(|error| format!("Could not remove {}: {error}", path.display()))
    } else {
        fs::remove_file(&path)
            .map_err(|error| format!("Could not remove {}: {error}", path.display()))
    }
}

fn repack_any<F>(
    archive: &Path,
    format: Format,
    password: Option<&str>,
    compression: u8,
    backup: bool,
    job: &SharedJob,
    mutate: F,
) -> Result<(), String>
where
    F: FnOnce(&Path) -> Result<(), String>,
{
    let suffix = format
        .tar_suffix()
        .or(match format {
            Format::SevenZ => Some("7z"),
            Format::Zip => Some("zip"),
            _ => None,
        })
        .ok_or_else(|| "Archive format cannot be edited".to_string())?;
    let parent = archive
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id();
    let work_dir = crate::fsops::private_cache_root().join(format!("repack-{pid}-{stamp}"));
    let out = parent.join(format!(".7zrn-{pid}-{stamp}.{suffix}"));

    let result = (|| {
        crate::fsops::ensure_private_dir(&work_dir)?;
        job::note(job, "Extracting archive to a temporary directory");
        let mut extract = extract_command(archive, format, &work_dir, &[], password)?;
        job::run_command_with_password(&mut extract, job, password)?;
        mutate(&work_dir)?;
        job::note(job, format!("Rebuilding {} archive", format.label()));
        if format.is_tar_family() {
            let mut create = Command::new(TAR);
            create
                .arg("-caf")
                .arg(&out)
                .arg("-C")
                .arg(&work_dir)
                .arg(".");
            job::run_command_with_password(&mut create, job, password)?;
        } else {
            let kind = if format == Format::Zip { "zip" } else { "7z" };
            let mut create = Command::new(SEVEN_ZIP);
            create.args(["a", "-y", "-bsp1", "-bso1", "-bse1", "-spd", "-sccUTF-8"]);
            create.arg(format!("-mx={}", compression.min(9)));
            create.arg(format!("-t{kind}"));
            if password.is_some() {
                create.arg("-p");
                if format == Format::SevenZ {
                    create.arg("-mhe=on");
                } else if format == Format::Zip {
                    create.arg("-mem=AES256");
                }
            }
            create.arg("--").arg(&out);
            let names = fs::read_dir(&work_dir)
                .map_err(|error| format!("Could not read temporary directory: {error}"))?;
            for entry in names {
                let entry = entry
                    .map_err(|error| format!("Could not read temporary directory: {error}"))?;
                create.arg(entry.file_name());
            }
            create.current_dir(&work_dir);
            job::run_command_with_password(&mut create, job, password)?;
        }
        if !out.is_file() {
            return Err("Archive rebuild produced no output".into());
        }
        if backup {
            let name = archive
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("archive");
            let backup_path = archive.with_file_name(format!("{name}.bak"));
            fs::copy(archive, &backup_path).map_err(|error| {
                format!("Could not write backup {}: {error}", backup_path.display())
            })?;
            job::note(job, format!("Backup written to {}", backup_path.display()));
        }
        fs::rename(&out, archive).map_err(|error| {
            format!(
                "Could not replace {} with the rebuilt archive: {error}",
                archive.display()
            )
        })?;
        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&out);
    }
    let _ = fs::remove_dir_all(&work_dir);
    result
}

pub fn is_split_volume(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    name.ends_with(".7z.001") || name.ends_with(".zip.001")
}

fn backup_archive(archive: &Path, job: &SharedJob) -> Result<(), String> {
    let name = archive
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("archive");
    let backup_path = archive.with_file_name(format!("{name}.bak"));
    fs::copy(archive, &backup_path)
        .map_err(|error| format!("Could not write backup {}: {error}", backup_path.display()))?;
    job::note(job, format!("Backup written to {}", backup_path.display()));
    Ok(())
}

pub(crate) fn safe_relative(path: &str) -> Option<PathBuf> {
    let mut out = PathBuf::new();
    for component in Path::new(path).components() {
        match component {
            Component::Normal(part) => out.push(part),
            Component::CurDir => {}
            Component::RootDir | Component::ParentDir | Component::Prefix(_) => return None,
        }
    }
    if out.as_os_str().is_empty() {
        None
    } else {
        Some(out)
    }
}

fn repack<F>(
    archive: &Path,
    format: Format,
    backup: bool,
    job: &SharedJob,
    mutate: F,
) -> Result<(), String>
where
    F: FnOnce(&Path) -> Result<(), String>,
{
    let suffix = format
        .tar_suffix()
        .ok_or_else(|| "Not a tar archive".to_string())?;
    let parent = archive
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id();
    let work_dir = crate::fsops::private_cache_root().join(format!("repack-{pid}-{stamp}"));
    let out = parent.join(format!(".7zrn-{pid}-{stamp}.{suffix}"));

    let result = (|| {
        crate::fsops::ensure_private_dir(&work_dir)?;

        job::note(job, "Extracting archive to a temporary directory");
        let mut extract = Command::new(TAR);
        extract.arg("-xf").arg(archive).arg("-C").arg(&work_dir);
        job::run_command(&mut extract, job)?;

        mutate(&work_dir)?;

        job::note(job, format!("Rebuilding {} archive", format.label()));
        let mut create = Command::new(TAR);
        create
            .arg("-caf")
            .arg(&out)
            .arg("-C")
            .arg(&work_dir)
            .arg(".");
        job::run_command(&mut create, job)?;

        if !out.is_file() {
            return Err("Archive rebuild produced no output".into());
        }
        if backup {
            backup_archive(archive, job)?;
        }
        fs::rename(&out, archive).map_err(|e| {
            format!(
                "Could not replace {} with the rebuilt archive: {e}",
                archive.display()
            )
        })?;
        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&out);
    }
    let _ = fs::remove_dir_all(&work_dir);
    result
}
