//! Log folder actions for the system-log board.
//! requirement-rust-tui

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::logger::StatusLog;

pub struct SystemLog {
    logger: Option<StatusLog>,
}

impl SystemLog {
    pub fn new(logger: Option<StatusLog>) -> Self {
        Self { logger }
    }

    pub fn log_dir(&self) -> String {
        self.logger
            .as_ref()
            .map(|logger| logger.log_dir().to_string())
            .unwrap_or_default()
    }

    pub fn log_files(&self) -> Vec<PathBuf> {
        let folder = self.log_dir();
        if folder.is_empty() {
            return Vec::new();
        }
        let root = match Path::new(&folder).canonicalize() {
            Ok(path) if path.is_dir() => path,
            _ => return Vec::new(),
        };
        let mut found = Vec::new();
        let entries = match fs::read_dir(&root) {
            Ok(entries) => entries,
            Err(_) => return Vec::new(),
        };
        for entry in entries.flatten() {
            if let Some(owned) = self.owned(&entry.path()) {
                found.push(owned);
            }
        }
        found.sort_by(|left, right| left.file_name().cmp(&right.file_name()));
        found
    }

    pub fn read_log(&mut self, path: &Path) -> Option<String> {
        let owned = self.owned(path)?;
        self.note("read log", &owned);
        match fs::read(&owned) {
            Ok(bytes) => Some(String::from_utf8_lossy(&bytes).into_owned()),
            Err(_) => None,
        }
    }

    pub fn clear_log(&mut self, path: &Path) -> bool {
        let Some(owned) = self.owned(path) else {
            return false;
        };
        self.note("clear log", &owned);
        File::create(&owned)
            .and_then(|mut file| file.write_all(b""))
            .is_ok()
    }

    pub fn folder_text(&self) -> String {
        let folder = self.log_dir();
        if folder.is_empty() {
            return "No log folder.".to_string();
        }
        format!("Log folder: {folder}")
    }

    fn owned(&self, path: &Path) -> Option<PathBuf> {
        let folder = self.log_dir();
        if folder.is_empty() {
            return None;
        }
        let root = Path::new(&folder).canonicalize().ok()?;
        let candidate = path.canonicalize().ok()?;
        if candidate.parent()? != root {
            return None;
        }
        if candidate.extension().and_then(|ext| ext.to_str()) != Some("log") {
            return None;
        }
        if !candidate.is_file() {
            return None;
        }
        Some(candidate)
    }

    fn note(&mut self, operation: &str, path: &Path) {
        if let Some(logger) = self.logger.as_mut() {
            logger.log_message(
                &format!("{operation} path={}", path.display()),
                "INFO",
                "menu",
            );
        }
    }
}
