use anyhow::{Context, Result, anyhow};
use directories::{BaseDirs, ProjectDirs};
use std::fs;
use std::path::{Path, PathBuf};

pub const APP_NAME: &str = "splitbot";
pub const LEGACY_APP_DIR: &str = ".splitbot";
pub const VAULT_FILE: &str = "vault.bin";
pub const LOG_FILE: &str = "splitbot.log";
pub const LAST_ORDER_FILE: &str = "last-order.json";
pub const EXPORTS_DIR: &str = "exports";

fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("", "", APP_NAME).ok_or_else(|| anyhow!("no home directory"))
}

fn legacy_app_dir() -> Result<PathBuf> {
    let base = BaseDirs::new().ok_or_else(|| anyhow!("no home directory"))?;
    Ok(base.home_dir().join(LEGACY_APP_DIR))
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst).with_context(|| format!("mkdir {}", dst.display()))?;
    for entry in fs::read_dir(src).with_context(|| format!("read {}", src.display()))? {
        let entry = entry.with_context(|| format!("iterate {}", src.display()))?;
        let file_type = entry
            .file_type()
            .with_context(|| format!("stat {}", entry.path().display()))?;
        let target = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &target)?;
        } else if file_type.is_file() {
            fs::copy(entry.path(), &target).with_context(|| {
                format!("copy {} -> {}", entry.path().display(), target.display())
            })?;
        }
    }
    Ok(())
}

fn migrate_legacy_app_dir(target: &Path) -> Result<()> {
    let legacy = legacy_app_dir()?;
    if target.exists() || !legacy.exists() {
        return Ok(());
    }

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).with_context(|| format!("mkdir {}", parent.display()))?;
    }

    match fs::rename(&legacy, target) {
        Ok(()) => Ok(()),
        Err(_) => copy_dir_all(&legacy, target),
    }
}

pub fn app_dir() -> Result<PathBuf> {
    let dir = project_dirs()?.data_local_dir().to_path_buf();
    migrate_legacy_app_dir(&dir)?;
    Ok(dir)
}

pub fn vault_path() -> Result<PathBuf> {
    Ok(app_dir()?.join(VAULT_FILE))
}

pub fn log_path() -> Result<PathBuf> {
    let dir = app_dir()?;
    std::fs::create_dir_all(&dir).with_context(|| format!("mkdir {}", dir.display()))?;
    Ok(dir.join(LOG_FILE))
}

pub fn last_order_path() -> Result<PathBuf> {
    let dir = app_dir()?;
    std::fs::create_dir_all(&dir).with_context(|| format!("mkdir {}", dir.display()))?;
    Ok(dir.join(LAST_ORDER_FILE))
}

pub fn exports_dir() -> Result<PathBuf> {
    let dir = app_dir()?.join(EXPORTS_DIR);
    fs::create_dir_all(&dir).with_context(|| format!("mkdir {}", dir.display()))?;
    Ok(dir)
}
