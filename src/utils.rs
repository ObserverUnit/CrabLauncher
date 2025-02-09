use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use zip::{result::ZipError, ZipArchive};
pub const MULTI_PATH_SEPRATOR: &'static str = if cfg!(target_os = "windows") {
    ";"
} else {
    ":"
};

#[derive(Debug, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum OsName {
    Linux,
    Windows,
    Osx,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Arch {
    X86_64,
    X86,
    ARM64,
}

pub fn extract(jar: &[u8], output: &Path, exclude: Option<&[PathBuf]>) -> Result<(), ZipError> {
    let exclude = exclude.unwrap_or_default();

    let reader = Cursor::new(jar);
    let mut archive = ZipArchive::new(reader)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;

        let file_path = match file.enclosed_name() {
            Some(path) => path,
            None => continue,
        };

        if exclude.contains(&file_path)
            || file_path
                .parent()
                .is_some_and(|p| exclude.contains(&p.to_path_buf()))
        {
            continue;
        }

        let output = output.join(&file_path);
        if file_path.is_dir() {
            fs::create_dir_all(output).unwrap();
        } else {
            if let Some(p) = output.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(&p)?;
                }
            }

            let mut outfile = std::fs::File::create(&output)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}
