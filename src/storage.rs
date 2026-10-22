//! Cache folder and persistence storage.
//! requirement-shell-cli-storage — one resolver, per-login per-process leaves.
//! requirement-rust-modules
//! A missed tier is silent. The preferred leaf is not /dev/shm/${APP_NAME}.

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

use crate::identity::APP_DIR;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[derive(Clone, Debug)]
pub struct Storage {
    pub app_name: String,
    pub home: String,
    pub login_name: String,
    pub pid: u32,
    pub environ: HashMap<String, String>,
    pub used: String,
}

impl Storage {
    pub fn new(environ: HashMap<String, String>, login: Option<String>, pid: Option<u32>) -> Self {
        let home = environ
            .get("HOME")
            .cloned()
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "/tmp".to_string());
        Self {
            app_name: "rust-agent".to_string(),
            home,
            login_name: login.unwrap_or_default(),
            pid: pid.unwrap_or_else(|| std::process::id()),
            environ,
            used: String::new(),
        }
    }

    fn env(&self, key: &str) -> String {
        self.environ.get(key).cloned().unwrap_or_default()
    }

    pub fn host_kind(&self) -> String {
        let forced = self.env("RUSTAGENT_CACHE_HOST");
        if matches!(forced.as_str(), "linux" | "gitbash" | "mac") {
            return forced;
        }
        if !self.env("MSYSTEM").is_empty() {
            return "gitbash".to_string();
        }
        match std::env::consts::OS {
            "macos" => "mac".to_string(),
            "windows" => "gitbash".to_string(),
            _ => "linux".to_string(),
        }
    }

    pub fn login(&self) -> String {
        let raw = if !self.login_name.is_empty() {
            self.login_name.clone()
        } else if !self.env("USERNAME").is_empty() {
            self.env("USERNAME")
        } else if !self.env("USER").is_empty() {
            self.env("USER")
        } else {
            "unknown".to_string()
        };
        let mut cleaned = String::new();
        for ch in raw.chars() {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-') {
                cleaned.push(ch);
            } else {
                cleaned.push('_');
            }
        }
        if cleaned.is_empty() {
            "unknown".to_string()
        } else {
            cleaned
        }
    }

    pub fn volatile_leaf(&self) -> String {
        format!("cache-{}-{}-{}", self.app_name, self.login(), self.pid)
    }

    pub fn home_leaf(&self) -> String {
        format!("cache-{}-{}", self.app_name, self.pid)
    }

    pub fn preferred_dir(&self) -> String {
        let leaf = self.volatile_leaf();
        if matches!(self.host_kind().as_str(), "gitbash" | "mac") {
            format!("/tmp/cache/{leaf}")
        } else {
            format!("/dev/shm/cache/{leaf}")
        }
    }

    pub fn fallback_dir(&self) -> String {
        match self.host_kind().as_str() {
            "gitbash" => Path::new(&self.home)
                .join("AppData")
                .join("Local")
                .join("Temp")
                .join(self.home_leaf())
                .to_string_lossy()
                .into_owned(),
            "mac" => Path::new(&self.home)
                .join("Library")
                .join("Caches")
                .join(self.home_leaf())
                .to_string_lossy()
                .into_owned(),
            _ => format!("/tmp/cache/{}", self.volatile_leaf()),
        }
    }

    pub fn fallback2_dir(&self) -> String {
        match self.host_kind().as_str() {
            "gitbash" => String::new(),
            "mac" => Path::new(&self.home)
                .join("cache")
                .join(self.home_leaf())
                .to_string_lossy()
                .into_owned(),
            _ => Path::new(&self.home)
                .join(".cache")
                .join(self.home_leaf())
                .to_string_lossy()
                .into_owned(),
        }
    }

    pub fn try_dir(&self, path: &str) -> bool {
        if path.is_empty() {
            return false;
        }
        let parent = Path::new(path).parent().map(|p| p.to_path_buf());
        let Some(parent) = parent else {
            return false;
        };
        let parent_text = parent.to_string_lossy();
        if parent_text == "/dev/shm/cache" || parent_text == "/tmp/cache" {
            if fs::create_dir_all(&parent).is_err() {
                return false;
            }
            #[cfg(unix)]
            {
                let _ = fs::set_permissions(&parent, fs::Permissions::from_mode(0o1777));
            }
        } else if fs::create_dir_all(&parent).is_err() {
            return false;
        }
        if fs::create_dir_all(path).is_err() {
            return false;
        }
        #[cfg(unix)]
        {
            if fs::set_permissions(path, fs::Permissions::from_mode(0o700)).is_err() {
                return false;
            }
        }
        let meta = match fs::metadata(path) {
            Ok(meta) => meta,
            Err(_) => return false,
        };
        if !meta.is_dir() {
            return false;
        }
        let probe = Path::new(path).join(".write-probe");
        match fs::File::create(&probe) {
            Ok(mut file) => {
                if file.write_all(b"").is_err() {
                    return false;
                }
            }
            Err(_) => return false,
        }
        fs::remove_file(probe).is_ok()
    }

    pub fn persistent_dir(&self) -> String {
        Path::new(&self.home)
            .join(".local")
            .join(APP_DIR)
            .to_string_lossy()
            .into_owned()
    }

    pub fn resolve_persistent(&self) -> Result<String, String> {
        let path = self.persistent_dir();
        let trimmed = path.trim_end_matches('/');
        if trimmed.ends_with("/.local/bin") {
            return Err(format!("Persistence storage must not be the install bin directory {path}"));
        }
        fs::create_dir_all(&path).map_err(|err| err.to_string())?;
        #[cfg(unix)]
        {
            let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o700));
        }
        Ok(path)
    }

    pub fn resolve(&mut self) -> Result<String, String> {
        let mut chosen = String::new();
        if self.env("RUSTAGENT_CACHE_SKIP") != "preferred" {
            let preferred = self.preferred_dir();
            if self.try_dir(&preferred) {
                chosen = preferred;
            }
        }
        if chosen.is_empty() {
            let first = self.fallback_dir();
            if self.try_dir(&first) {
                chosen = first;
            }
        }
        if chosen.is_empty() {
            let second = self.fallback2_dir();
            if !second.is_empty() && self.try_dir(&second) {
                chosen = second;
            }
        }
        if chosen.is_empty() {
            return Err("Cannot create cache folder".to_string());
        }
        self.used = chosen.clone();
        Ok(chosen)
    }

    pub fn mktemp(&mut self, suffix: &str) -> Result<String, String> {
        if suffix.contains("$$") {
            return Err("refuse predictable $$ name template".to_string());
        }
        let root = if self.used.is_empty() {
            self.resolve()?
        } else {
            self.used.clone()
        };
        let name = format!(
            "{}.{}.{}.{}",
            self.app_name,
            suffix,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        );
        let path = Path::new(&root).join(name);
        fs::File::create(&path).map_err(|err| err.to_string())?;
        Ok(path.to_string_lossy().into_owned())
    }

    pub fn about_lines(&mut self) -> Vec<String> {
        let used = if self.used.is_empty() {
            self.resolve().unwrap_or_else(|_| "(unavailable)".to_string())
        } else {
            self.used.clone()
        };
        let mut lines = vec![
            format!("Cache folder used: {used}"),
            format!("Cache folder (preferred): {}", self.preferred_dir()),
            format!("Cache folder (1st fallback): {}", self.fallback_dir()),
        ];
        let second = self.fallback2_dir();
        if !second.is_empty() {
            lines.push(format!("Cache folder (2nd fallback): {second}"));
        }
        lines.push(format!("Persistence storage: {}", self.persistent_dir()));
        lines
    }
}
