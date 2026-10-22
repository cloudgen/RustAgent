//! This-login-only host detect.
//! requirement-shell-cli-default-interaction — Termux, Git Bash, Windows cmd.
//! requirement-rust-modules

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Host {
    pub environ: HashMap<String, String>,
}

impl Host {
    pub fn from_process() -> Self {
        Self {
            environ: std::env::vars().collect(),
        }
    }

    pub fn from_map(environ: HashMap<String, String>) -> Self {
        Self { environ }
    }

    pub fn get(&self, key: &str) -> String {
        self.environ.get(key).cloned().unwrap_or_default()
    }

    pub fn override_name(&self) -> String {
        self.get("RUSTAGENT_HOST_OVERRIDE")
    }

    pub fn is_termux(&self) -> bool {
        if !self.get("TERMUX_VERSION").is_empty() {
            return true;
        }
        let forced = self.override_name();
        if matches!(forced.as_str(), "multiuser" | "gitbash" | "windows-cmd") {
            return false;
        }
        if forced == "termux" {
            return true;
        }
        if !self.get("PREFIX").is_empty() {
            return true;
        }
        uname_says_android()
    }

    pub fn is_git_bash(&self) -> bool {
        let forced = self.override_name();
        if forced == "gitbash" {
            return true;
        }
        if matches!(forced.as_str(), "multiuser" | "termux" | "windows-cmd") {
            return false;
        }
        if !self.get("MSYSTEM").is_empty() {
            return true;
        }
        let system = std::env::consts::OS;
        system.starts_with("mingw") || system.starts_with("msys")
    }

    pub fn is_windows_cmd(&self) -> bool {
        let forced = self.override_name();
        if forced == "windows-cmd" {
            return true;
        }
        if matches!(forced.as_str(), "multiuser" | "termux" | "gitbash") {
            return false;
        }
        if self.get("OS") != "Windows_NT" {
            return false;
        }
        if self.is_git_bash() {
            return false;
        }
        let system = std::env::consts::OS;
        if system.starts_with("cygwin") || system.starts_with("mingw") || system.starts_with("msys") {
            return false;
        }
        if !self.get("WSL_DISTRO_NAME").is_empty() || !self.get("WSL_INTEROP").is_empty() {
            return false;
        }
        true
    }

    pub fn normal_user_only(&self) -> bool {
        if self.override_name() == "multiuser" {
            return false;
        }
        self.is_termux() || self.is_git_bash() || self.is_windows_cmd()
    }

    pub fn label(&self) -> String {
        if self.is_termux() {
            return "termux".to_string();
        }
        if self.is_git_bash() {
            return "gitbash".to_string();
        }
        if self.is_windows_cmd() {
            return "windows-cmd".to_string();
        }
        String::new()
    }
}

fn uname_says_android() -> bool {
    let release = std::fs::read_to_string("/proc/sys/kernel/osrelease").unwrap_or_default();
    release.to_ascii_lowercase().contains("android")
}
