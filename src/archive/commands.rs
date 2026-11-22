use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::job::{self, SharedJob};

use super::{Format, NewFormat, SEVEN_ZIP, TAR};

pub(crate) fn apply_password(cmd: &mut Command, password: Option<&str>) {
    let mut argument = OsString::from("-p");
    if let Some(password) = password {
        argument.push(password);
    }
    cmd.arg(argument);
}

fn apply_password_if_present(cmd: &mut Command, password: Option<&str>) {
    if let Some(password) = password.filter(|value| !value.is_empty()) {
        let mut argument = OsString::from("-p");
        argument.push(password);
        cmd.arg(argument);
    }
}
pub fn extract_command(
    archive: &Path,
    format: Format,
    dest: &Path,
    raw_names: &[String],
    password: Option<&str>,
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
        apply_password(&mut cmd, password);
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
) -> Result<Vec<Command>, String> {
    let format = Format::detect(archive);
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
        cmd.args([
            "a",
            "-y",
            "-bsp1",
            "-bso1",
            "-bse1",
            "-spd",
            "-mx=5",
            "-sccUTF-8",
        ]);
        apply_password_if_present(&mut cmd, password);
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
    password: Option<&str>,
) -> Result<Command, String> {
    let format = Format::detect(archive);
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
    apply_password_if_present(&mut cmd, password);
    cmd.arg("--").arg(archive);
    for name in raw_names {
        cmd.arg(name);
    }
    Ok(cmd)
}

pub fn test_command(archive: &Path, format: Format, password: Option<&str>) -> Command {
    if format.is_tar_family() {
        let mut cmd = Command::new(TAR);
        cmd.arg("-tf").arg(archive);
        cmd
    } else {
        let mut cmd = Command::new(SEVEN_ZIP);
        cmd.args(["t", "-y", "-bsp1", "-bso1", "-bse1"]);
        apply_password(&mut cmd, password);
        cmd.arg("--").arg(archive);
        cmd
    }
}

pub fn create_command(
    output: &Path,
    format: NewFormat,
    cwd: &Path,
    names: &[OsString],
) -> Result<Command, String> {
    if names.is_empty() {
        return Err("Nothing to archive".into());
    }
    if let Some(kind) = format.seven_zip_type() {
        let mut cmd = Command::new(SEVEN_ZIP);
        cmd.args([
            "a",
            "-y",
            "-bsp1",
            "-bso1",
            "-bse1",
            "-spd",
            "-mx=5",
            "-sccUTF-8",
        ])
        .arg(format!("-t{kind}"))
        .arg("--")
        .arg(output);
        for name in names {
            cmd.arg(name);
        }
        cmd.current_dir(cwd);
        Ok(cmd)
    } else {
        let mut cmd = Command::new(TAR);
        cmd.arg("-caf").arg(output).arg("-C").arg(cwd).arg("--");
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
    repack(archive, format, job, move |root| {
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
    job: &SharedJob,
) -> Result<(), String> {
    if !format.is_tar_family() {
        return Err("Not a tar archive".into());
    }
    if display_paths.is_empty() {
        return Err("No entries selected".into());
    }
    repack(archive, format, job, |root| {
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

fn repack<F>(archive: &Path, format: Format, job: &SharedJob, mutate: F) -> Result<(), String>
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
    let work_dir = std::env::temp_dir().join(format!("7zrn-{pid}-{stamp}"));
    let out = parent.join(format!(".7zrn-{pid}-{stamp}.{suffix}"));

    let result = (|| {
        fs::create_dir_all(&work_dir)
            .map_err(|e| format!("Could not create temporary directory: {e}"))?;

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
