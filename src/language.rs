//! Menu language. Thirteen codes, the leaf, and the words the menu prints.
//! requirement-rust-cli-language

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use crate::identity::{APP_DIR, LANG_ENV};
use crate::lang_catalog;

#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

pub struct MenuLanguage {
    pub home: String,
    code: String,
}

impl MenuLanguage {
    pub fn new(home: Option<String>) -> Self {
        let home = match home {
            Some(value) => value,
            None => std::env::var("HOME").unwrap_or_default(),
        };
        let mut language = Self {
            home,
            code: "en".to_string(),
        };
        language.load();
        language
    }

    pub fn codes(&self) -> Vec<&'static str> {
        lang_catalog::CODES.iter().map(|item| item.code).collect()
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn endonym(&self, code: &str) -> &str {
        lang_catalog::CODES
            .iter()
            .find(|item| item.code == code)
            .map(|item| item.endonym)
            .unwrap_or("")
    }

    pub fn leaf_path(&self) -> String {
        if self.home.is_empty() {
            return String::new();
        }
        Path::new(&self.home)
            .join(".local")
            .join(APP_DIR)
            .join("language")
            .to_string_lossy()
            .into_owned()
    }

    pub fn load(&mut self) {
        if let Ok(forced) = std::env::var(LANG_ENV) {
            if self.codes().iter().any(|code| *code == forced) {
                self.code = forced;
                return;
            }
        }
        self.code = self.read_leaf();
    }

    fn read_leaf(&self) -> String {
        let path = self.leaf_path();
        if path.is_empty() {
            return "en".to_string();
        }
        let text = match fs::read_to_string(&path) {
            Ok(body) => body,
            Err(_) => return "en".to_string(),
        };
        let line = text.lines().next().unwrap_or("").replace('\r', "");
        let line = line.trim();
        if self.codes().iter().any(|code| *code == line) {
            return line.to_string();
        }
        "en".to_string()
    }

    pub fn save(&mut self, code: &str) -> String {
        if !self.codes().iter().any(|item| *item == code) {
            return self.text("save_failed");
        }
        let path = self.leaf_path();
        if path.is_empty() {
            return self.text("save_failed");
        }
        let previous = self.code.clone();
        if self.write_leaf(&path, code).is_err() {
            self.code = previous;
            return self.text("save_failed");
        }
        self.code = code.to_string();
        self.text("saved")
    }

    fn write_leaf(&self, path: &str, code: &str) -> Result<(), ()> {
        let folder = Path::new(path).parent().ok_or(())?;
        fs::create_dir_all(folder).map_err(|_| ())?;
        #[cfg(unix)]
        {
            let mut perms = fs::metadata(folder).map_err(|_| ())?.permissions();
            perms.set_mode(0o700);
            fs::set_permissions(folder, perms).map_err(|_| ())?;
        }
        let mut options = OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        options.mode(0o600);
        let mut file = options.open(path).map_err(|_| ())?;
        file.write_all(format!("{code}\n").as_bytes())
            .map_err(|_| ())?;
        #[cfg(unix)]
        {
            let mut perms = fs::metadata(path).map_err(|_| ())?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(path, perms).map_err(|_| ())?;
        }
        Ok(())
    }

    pub fn text(&self, key: &str) -> String {
        let found = lang_catalog::text(&self.code, key);
        if found.is_empty() {
            return lang_catalog::text("en", key).to_string();
        }
        found.to_string()
    }

    pub fn path_label(&self) -> String {
        self.text("path")
    }

    pub fn lang_long(&self, endonym: &str) -> String {
        self.text("lang_long").replace("{name}", endonym)
    }

    pub fn row_short(&self, kind: &str, english: &str) -> String {
        match short_key(kind) {
            Some(key) => self.text(key),
            None => english.to_string(),
        }
    }

    pub fn row_long(&self, kind: &str, english: &str) -> String {
        match long_key(kind) {
            Some(key) => self.text(key),
            None => english.to_string(),
        }
    }

    pub fn token_kind(&self, token: &str, layer: &str) -> Option<String> {
        if layer == "lang" {
            return lang_catalog::lang_token(token).map(|code| format!("set-{code}"));
        }
        if layer == "front" {
            return lang_catalog::front_token(token).map(|kind| kind.to_string());
        }
        None
    }
}

fn short_key(kind: &str) -> Option<&'static str> {
    match kind {
        "language" => Some("language_short"),
        "system-log" => Some("system_log_short"),
        "self-management" => Some("self_short"),
        "exit" => Some("exit_short"),
        "back" => Some("back_short"),
        _ => None,
    }
}

fn long_key(kind: &str) -> Option<&'static str> {
    match kind {
        "hello" => Some("hello_long"),
        "language" => Some("language_long"),
        "system-log" => Some("system_log_long"),
        "self-management" => Some("self_long"),
        "exit" => Some("exit_long"),
        "back" => Some("back_long"),
        "view-log" => Some("view_long"),
        "clear-log" => Some("clear_long"),
        "log-folder" => Some("folder_long"),
        "version" => Some("version_long"),
        "about" => Some("about_long"),
        "version-check" => Some("version_check_long"),
        "self-update" => Some("self_update_long"),
        "self-uninstall" => Some("self_uninstall_long"),
        "self-install" => Some("self_install_long"),
        _ => None,
    }
}
