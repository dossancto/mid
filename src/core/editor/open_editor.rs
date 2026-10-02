use std::{
    env,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::core::query::Error;

static TEMP_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub fn open_editor_recover_text(query: &str) -> color_eyre::Result<Option<String>> {
    let editor = env::var("EDITOR")
        .ok()
        .filter(|editor| !editor.trim().is_empty())
        .ok_or(Error::EditorNotConfigured())?;

    let (path, mut file) = create_query_temp_file()?;
    file.write_all(query.as_bytes())?;
    file.flush()?;
    drop(file);

    let result = (|| -> color_eyre::Result<Option<String>> {
        let mut editor_parts = editor.split_whitespace();
        let program = editor_parts.next().ok_or(Error::EditorNotConfigured())?;
        let status = Command::new(program)
            .args(editor_parts)
            .arg(&path)
            .status()
            .map_err(|_| Error::OpenEditor())?;
        if !status.success() {
            return Err(Error::OpenEditor().into());
        }

        Ok(Some(fs::read_to_string(&path)?))
    })();

    let _ = fs::remove_file(&path);
    Ok(result?)
}

pub fn open_editor_in_file(path: &Path) -> color_eyre::Result<()> {
    let editor = get_editor()?;
    open_editor_with_args(&editor, path)
}

fn get_editor() -> color_eyre::Result<String> {
    env::var("EDITOR")
        .ok()
        .filter(|editor| !editor.trim().is_empty())
        .ok_or_else(|| Error::EditorNotConfigured().into())
}

fn open_editor_with_args(editor: &str, path: &Path) -> color_eyre::Result<()> {
    let mut editor_parts = editor.split_whitespace();
    let program = editor_parts.next().ok_or(Error::EditorNotConfigured())?;
    let status = Command::new(program)
        .args(editor_parts)
        .arg(path)
        .status()
        .map_err(|_| Error::OpenEditor())?;

    if !status.success() {
        return Err(Error::OpenEditor().into());
    }

    Ok(())
}

fn create_query_temp_file() -> std::result::Result<(PathBuf, fs::File), Error> {
    let sequence = TEMP_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);

    for attempt in 0..100 {
        let path = env::temp_dir().join(format!(
            "mid-query-{}-{sequence}-{attempt}.sql",
            std::process::id()
        ));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(Error::CreateTempFile(error)),
        }
    }

    Err(Error::CreateTempFile(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not create a unique query temporary file after 100 attempts",
    )))
}
