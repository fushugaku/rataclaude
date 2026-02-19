use std::path::Path;

use anyhow::{Context, Result};

pub fn copy_entry(src: &Path, dest_dir: &Path) -> Result<()> {
    let file_name = src.file_name().context("no file name")?;
    let dest = dest_dir.join(file_name);

    if src.is_dir() {
        copy_dir_recursive(src, &dest)?;
    } else {
        std::fs::copy(src, &dest).context("copy file")?;
    }
    Ok(())
}

pub fn move_entry(src: &Path, dest_dir: &Path) -> Result<()> {
    let file_name = src.file_name().context("no file name")?;
    let dest = dest_dir.join(file_name);
    std::fs::rename(src, &dest).context("move/rename entry")?;
    Ok(())
}

pub fn delete_path(path: &Path) -> Result<()> {
    if path.is_dir() {
        std::fs::remove_dir_all(path).context("delete directory")?;
    } else {
        std::fs::remove_file(path).context("delete file")?;
    }
    Ok(())
}

pub fn rename_entry(path: &Path, new_name: &str) -> Result<()> {
    let parent = path.parent().context("no parent directory")?;
    let dest = parent.join(new_name);
    std::fs::rename(path, &dest).context("rename entry")?;
    Ok(())
}

pub fn create_dir(parent: &Path, name: &str) -> Result<()> {
    let path = parent.join(name);
    std::fs::create_dir(&path).context("create directory")?;
    Ok(())
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest).context("create dest dir")?;
    for entry in std::fs::read_dir(src).context("read source dir")? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            std::fs::copy(&src_path, &dest_path)?;
        }
    }
    Ok(())
}
