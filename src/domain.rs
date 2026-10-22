//! Install and run the peer grok binary, and back up its auth files.
//! requirement-domain-grok-cli — verbs, deposit, sudoer JSON, setup channel.
//! requirement-grok-auth-backup — session words and the logged-in sync skip.
//! requirement-grok-setup — vendor channel, no shell installer.
//! requirement-grok-crontab — backup every 30 minutes and sync-auth at minute 45.
//! requirement-sudoer-json-file — backup-only grant. This process does not write /etc.
//! requirement-shell-startup-files — user rc stickers.
//! Peer binary stays grok. Auth files stay under ~/.grok.

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use crate::host::Host;
use crate::identity::{self, CONSOLE_NAME, DEPOSIT, GLOBAL_BIN, VERSION};
use crate::output::Output;
use crate::shell_startup::ShellStartup;
use crate::storage::Storage;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

const VENDOR_BASE: &str = "https://x.ai/cli";
const VENDOR_FALLBACK: &str = "https://storage.googleapis.com/grok-build-public-artifacts/cli";
const CHANNEL: &str = "stable";
const SESSION_WORDS: &[&str] = &["valid", "timeout", "invalid", "missing"];

const WRAPPER_TEMPLATE: &str = include_str!("wrapper_template.sh");
const ADMIN_TEMPLATE: &str = include_str!("admin_sudoers.sh");

pub enum RunReply {
    Code(i32),
    Text(String),
    Yes(bool),
}

pub type Runner = Box<dyn FnMut(&[String]) -> RunReply>;

pub struct Domain {
    pub output: Output,
    pub storage: Storage,
    pub host: Host,
    pub environ: HashMap<String, String>,
    pub session_override: Option<String>,
    pub deposit_root: String,
    pub grok_home: Option<String>,
    pub runner: Option<Runner>,
    pub startup: ShellStartup,
    pub euid: u32,
    last_stdout: String,
    last_stderr: String,
}

impl Domain {
    pub fn from_process() -> Self {
        Self::new(std::env::vars().collect(), None, None, None, None)
    }

    pub fn new(
        environ: HashMap<String, String>,
        session_override: Option<String>,
        deposit_root: Option<String>,
        grok_home: Option<String>,
        euid: Option<u32>,
    ) -> Self {
        let deposit_root = deposit_root.unwrap_or_else(|| {
            let forced = environ.get("RUSTAGENT_DEPOSIT").cloned().unwrap_or_default();
            if forced.is_empty() {
                DEPOSIT.to_string()
            } else {
                forced
            }
        });
        let euid = euid.unwrap_or_else(current_euid);
        let host = Host::from_map(environ.clone());
        let storage = Storage::new(environ.clone(), None, None);
        let startup = ShellStartup::new(VERSION, environ.clone(), Some(euid));
        Self {
            output: Output::new(),
            storage,
            host,
            environ,
            session_override,
            deposit_root,
            grok_home,
            runner: None,
            startup,
            euid,
            last_stdout: String::new(),
            last_stderr: String::new(),
        }
    }

    fn env(&self, key: &str) -> String {
        self.environ.get(key).cloned().unwrap_or_default()
    }

    fn env_first(&self, keys: &[&str]) -> String {
        for key in keys {
            let value = self.env(key);
            if !value.is_empty() {
                return value;
            }
        }
        String::new()
    }

    pub fn global_bin(&self) -> String {
        let root = self.env("RUSTAGENT_GLOBAL_BIN");
        let root = if root.is_empty() { GLOBAL_BIN.to_string() } else { root };
        Path::new(&root).join(CONSOLE_NAME).to_string_lossy().into_owned()
    }

    fn invoking_home(&self) -> String {
        let sudo_user = self.env("SUDO_USER");
        if self.euid == 0 && !sudo_user.is_empty() {
            if let Some(home) = passwd_home(&sudo_user) {
                return home;
            }
        }
        let home = self.env("HOME");
        if home.is_empty() { "/tmp".to_string() } else { home }
    }

    pub fn auth_home(&self) -> String {
        if let Some(path) = &self.grok_home {
            return path.clone();
        }
        if self.euid == 0 && !self.env("SUDO_USER").is_empty() {
            return Path::new(&self.invoking_home()).join(".grok").to_string_lossy().into_owned();
        }
        Path::new(&self.invoking_home()).join(".grok").to_string_lossy().into_owned()
    }

    pub fn peer_ready(&self) -> bool {
        is_exec(&self.peer_path())
    }

    pub fn peer_path(&self) -> String {
        let override_path = self.env("RUSTAGENT_GROK_BIN");
        if !override_path.is_empty() {
            return override_path;
        }
        if let Some(found) = self.which("grok") {
            return found;
        }
        Path::new(&self.auth_home()).join("bin").join("grok").to_string_lossy().into_owned()
    }

    pub fn session_word(&mut self) -> String {
        if let Some(word) = &self.session_override {
            if SESSION_WORDS.contains(&word.as_str()) {
                return word.clone();
            }
        }
        let forced = self.env("RUSTAGENT_SESSION");
        if SESSION_WORDS.contains(&forced.as_str()) {
            return forced;
        }
        if self.uses_local_auth() {
            return self.session_from_local_auth();
        }
        self.session_from_probe()
    }

    pub fn logged_in(&mut self) -> bool {
        matches!(self.session_word().as_str(), "valid" | "timeout")
    }

    fn uses_local_auth(&self) -> bool {
        if self.host.is_termux() {
            return true;
        }
        if matches!(self.host.override_name().as_str(), "multiuser" | "gitbash" | "windows-cmd") {
            return false;
        }
        self.which("proot").is_some()
    }

    fn session_from_local_auth(&self) -> String {
        let path = Path::new(&self.auth_home()).join("auth.json");
        let Ok(bytes) = fs::read(&path) else {
            return "invalid".to_string();
        };
        if bytes.is_empty() {
            return "invalid".to_string();
        }
        let Ok(data) = serde_json::from_slice::<Value>(&bytes) else {
            return "invalid".to_string();
        };
        if cookies_still_good(&data) {
            "valid".to_string()
        } else {
            "invalid".to_string()
        }
    }

    fn session_from_probe(&mut self) -> String {
        let peer = self.peer_path();
        if !is_exec(&peer) {
            return "missing".to_string();
        }
        if self.runner.is_some() {
            let reply = self.ask(&[ "probe".to_string(), peer ]);
            return match reply {
                Some(RunReply::Code(0)) => "valid".to_string(),
                Some(RunReply::Code(124 | 137)) => "timeout".to_string(),
                Some(RunReply::Code(_)) => "invalid".to_string(),
                Some(RunReply::Yes(true)) => "valid".to_string(),
                Some(RunReply::Yes(false)) => "invalid".to_string(),
                Some(RunReply::Text(text)) if text.trim() == "0" => "valid".to_string(),
                Some(RunReply::Text(_)) => "invalid".to_string(),
                None => "missing".to_string(),
            };
        }
        let mut child = match Command::new(&peer)
            .arg("-p")
            .arg("hello")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(child) => child,
            Err(_) => return "missing".to_string(),
        };
        let started = std::time::Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    let code = status.code().unwrap_or(1);
                    if code == 0 {
                        return "valid".to_string();
                    }
                    if code == 124 || code == 137 {
                        return "timeout".to_string();
                    }
                    return "invalid".to_string();
                }
                Ok(None) if started.elapsed().as_secs() >= 14 => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return "timeout".to_string();
                }
                Ok(None) => std::thread::sleep(std::time::Duration::from_millis(50)),
                Err(_) => return "missing".to_string(),
            }
        }
    }

    pub fn check_session(&mut self) -> i32 {
        let word = self.session_word();
        if word == "valid" {
            let message = if self.uses_local_auth() {
                "Grok is logged in (local auth cookies).".to_string()
            } else {
                "Grok is logged in (grok -p hello succeeded).".to_string()
            };
            return self.output.ok("check-session", &message, Some(json!({"session": word})));
        }
        if word == "missing" {
            return self.output.error(
                &format!("Grok is not installed (no grok on PATH or under {}/bin).", self.auth_home()),
                &format!("{CONSOLE_NAME} setup, then grok login, then {CONSOLE_NAME} check-session"),
            );
        }
        if word == "timeout" {
            return self.output.ok("check-session", "timeout", Some(json!({"session": "timeout"})));
        }
        self.output.error(
            "Grok is not logged in.",
            &format!("grok login, then {CONSOLE_NAME} check-session"),
        )
    }

    pub fn backup(&mut self) -> i32 {
        if self.euid != 0 && self.env("RUSTAGENT_BACKUP_AS_ROOT") != "1" {
            if self.env("RUSTAGENT_BACKUP_NO_SUDO") == "1" {
                return self.deposit_files();
            }
            return self.output.error(
                "backup deposits auth files as root.",
                &format!("sudo -n {} backup", self.global_bin()),
            );
        }
        self.deposit_files()
    }

    pub fn deposit_files(&mut self) -> i32 {
        let source = self.auth_home();
        let names = auth_files(&source);
        if names.is_empty() {
            return self.output.error(
                &format!("No auth files under {source}."),
                &format!("{CONSOLE_NAME} setup"),
            );
        }
        if fs::create_dir_all(&self.deposit_root).is_err() {
            return self.output.error(
                &format!("Cannot create deposit {}.", self.deposit_root),
                &format!("sudo -n {} backup", self.global_bin()),
            );
        }
        let _ = set_mode(Path::new(&self.deposit_root), 0o755);
        let mut copied = Vec::new();
        for path in names {
            let base = Path::new(&path).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            let dest = Path::new(&self.deposit_root).join(&base);
            if fs::copy(&path, &dest).is_err() || !set_mode(&dest, 0o644) {
                return self.output.error(
                    &format!("Cannot write {}.", dest.display()),
                    &format!("sudo -n {} backup", self.global_bin()),
                );
            }
            copied.push(base);
        }
        let message = format!("Deposited {}.", copied.join(", "));
        self.output.ok(
            "backup",
            &message,
            Some(json!({"deposit": self.deposit_root, "files": copied})),
        )
    }

    pub fn sync_auth(&mut self) -> i32 {
        if self.logged_in() {
            return self.output.ok(
                "sync-auth",
                "No sync-auth for logged-in environment.",
                Some(json!({"reason": "logged-in"})),
            );
        }
        self.copy_deposit_to_home()
    }

    fn copy_deposit_to_home(&mut self) -> i32 {
        let names = auth_files(&self.deposit_root);
        if names.is_empty() {
            return self.output.error(
                &format!("Deposit {} has no auth files.", self.deposit_root),
                &format!("{CONSOLE_NAME} backup"),
            );
        }
        let dest_root = self.auth_home();
        if fs::create_dir_all(&dest_root).is_err() {
            return self.output.error(
                &format!("Cannot create {dest_root}."),
                &format!("{CONSOLE_NAME} sync-auth"),
            );
        }
        let _ = set_mode(Path::new(&dest_root), 0o700);
        for path in names {
            let base = Path::new(&path).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            let dest = Path::new(&dest_root).join(&base);
            if fs::copy(&path, &dest).is_err() {
                return self.output.error(
                    &format!("Cannot write {}.", dest.display()),
                    &format!("{CONSOLE_NAME} sync-auth"),
                );
            }
            if base == "auth.json" {
                let _ = set_mode(&dest, 0o600);
            }
        }
        self.output.ok("sync-auth", &format!("Restored auth files into {dest_root}."), None)
    }

    fn which(&self, name: &str) -> Option<String> {
        let path_env = if self.environ.contains_key("PATH") {
            self.env("PATH")
        } else {
            std::env::var("PATH").unwrap_or_default()
        };
        which_in(name, &path_env)
    }

    fn tool(&self, keys: &[&str], fallback: &str) -> String {
        let forced = self.env_first(keys);
        if !forced.is_empty() {
            return forced;
        }
        self.which(fallback).unwrap_or_default()
    }

    fn ask(&mut self, argv: &[String]) -> Option<RunReply> {
        let mut runner = self.runner.take()?;
        let reply = runner(argv);
        self.runner = Some(runner);
        Some(reply)
    }

    fn run_captured(&mut self, argv: &[String], timeout_secs: u64) -> i32 {
        if self.runner.is_some() {
            let reply = self.ask(argv);
            return match reply {
                Some(RunReply::Code(code)) => code,
                Some(RunReply::Text(text)) => {
                    self.last_stdout = text;
                    0
                }
                Some(RunReply::Yes(true)) => 0,
                Some(RunReply::Yes(false)) => 1,
                None => 1,
            };
        }
        if argv.is_empty() {
            return 1;
        }
        let mut command = Command::new(&argv[0]);
        command.args(&argv[1..]);
        for (key, value) in &self.environ {
            command.env(key, value);
        }
        let Ok(mut child) = command.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() else {
            return 1;
        };
        let started = std::time::Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    self.last_stdout = read_pipe(child.stdout.take());
                    self.last_stderr = read_pipe(child.stderr.take());
                    return status.code().unwrap_or(1);
                }
                Ok(None) if started.elapsed().as_secs() >= timeout_secs => {
                    let _ = child.kill();
                    let _ = child.wait();
                    self.last_stdout.clear();
                    self.last_stderr.clear();
                    return 124;
                }
                Ok(None) => std::thread::sleep(std::time::Duration::from_millis(20)),
                Err(_) => return 1,
            }
        }
    }

    pub fn parse_remote_spec(&self, spec: &str) -> Option<(String, String)> {
        if spec.is_empty() || spec.starts_with('-') {
            return None;
        }
        if spec.chars().any(|ch| ";&|$`()<>\\/ \t*[]".contains(ch)) {
            return None;
        }
        if spec.matches('@').count() > 1 {
            return None;
        }
        let (user, host) = if let Some((user, host)) = spec.split_once('@') {
            if user.is_empty() || host.is_empty() {
                return None;
            }
            if !user.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '-')) {
                return None;
            }
            let first = user.chars().next()?;
            if !(first.is_ascii_alphanumeric() || first == '_') {
                return None;
            }
            (user.to_string(), host.to_string())
        } else {
            (String::new(), spec.to_string())
        };
        if host_is_ipv4(&host) || host_is_dns(&host) {
            Some((user, host))
        } else {
            None
        }
    }

    fn preferred_remote_path(&self) -> String {
        Path::new(&self.storage.persistent_dir())
            .join("preferred-remote")
            .to_string_lossy()
            .into_owned()
    }

    pub fn preferred_remote_load(&self) -> String {
        fs::read_to_string(self.preferred_remote_path())
            .unwrap_or_default()
            .trim()
            .to_string()
    }

    pub fn preferred_remote_save(&mut self, spec: &str) -> bool {
        let Ok(folder) = self.storage.resolve_persistent() else {
            return false;
        };
        let path = Path::new(&folder).join("preferred-remote");
        let mut file = match fs::File::create(&path) {
            Ok(file) => file,
            Err(_) => return false,
        };
        if writeln!(file, "{spec}").is_err() {
            return false;
        }
        let _ = set_mode(&path, 0o600);
        true
    }

    fn remote_root(&self) -> String {
        let value = self.env("RUSTAGENT_REMOTE_ROOT");
        if value.is_empty() { self.deposit_root.clone() } else { value }
    }

    fn remote_target(user: &str, host: &str) -> String {
        if user.is_empty() { host.to_string() } else { format!("{user}@{host}") }
    }

    pub fn sync_from_remote(&mut self, spec: &str) -> i32 {
        if self.logged_in() {
            return self.output.ok(
                "sync-auth-from-remote",
                "No sync-auth for logged-in environment.",
                Some(json!({"reason": "logged-in"})),
            );
        }
        let mut spec = spec.to_string();
        if spec.is_empty() {
            spec = self.preferred_remote_load();
        }
        let Some((user, host)) = self.parse_remote_spec(&spec) else {
            return self.output.error(
                &format!("Cannot sync-auth-from-remote: remote '{spec}' is not user@host, IPv4, domain, or user@domain."),
                &format!("{CONSOLE_NAME} sync-auth-from-remote USER@HOST"),
            );
        };
        let scp = self.tool(&["RUSTAGENT_SCP", "GROK_CLI_SCP"], "scp");
        if !is_exec(&scp) {
            return self.output.error(
                "Cannot sync-auth-from-remote: scp not found.",
                &format!("install openssh-client, then {CONSOLE_NAME} sync-auth-from-remote {spec}"),
            );
        }
        let dest_root = self.auth_home();
        if fs::create_dir_all(&dest_root).is_err() {
            return self.output.error(
                &format!("Cannot create {dest_root}."),
                &format!("{CONSOLE_NAME} sync-auth-from-remote {spec}"),
            );
        }
        let _ = set_mode(Path::new(&dest_root), 0o700);
        let label = Self::remote_target(&user, &host);
        let remote_root = self.remote_root();
        let remote = format!("{label}:{remote_root}/auth.json");
        let Ok(tmp) = self.storage.mktemp("authjson") else {
            return self.output.error(
                "Cannot create a scratch file for auth.json.",
                &format!("{CONSOLE_NAME} sync-auth-from-remote {spec}"),
            );
        };
        self.output.info(&format!("Fetching auth.json from {label}"));
        let code = self.run_captured(
            &[scp.clone(), "-o".into(), "BatchMode=yes".into(), "-o".into(), "ConnectTimeout=15".into(), remote, tmp.clone()],
            30,
        );
        let size = fs::metadata(&tmp).map(|meta| meta.len()).unwrap_or(0);
        if code != 0 || size == 0 {
            let _ = fs::remove_file(&tmp);
            return self.output.error(
                &format!("Cannot sync-auth-from-remote: scp from {label} failed."),
                &format!("ssh {label}, then {CONSOLE_NAME} sync-auth-from-remote {spec}"),
            );
        }
        let dest = Path::new(&dest_root).join("auth.json");
        if fs::copy(&tmp, &dest).is_err() {
            let _ = fs::remove_file(&tmp);
            return self.output.error(
                &format!("Cannot write {}.", dest.display()),
                &format!("{CONSOLE_NAME} sync-auth-from-remote {spec}"),
            );
        }
        let _ = set_mode(&dest, 0o600);
        let _ = fs::remove_file(&tmp);
        let mut copied = vec!["auth.json".to_string()];
        if let Ok(lock_tmp) = self.storage.mktemp("authlock") {
            let lock_remote = format!("{label}:{remote_root}/auth.json.lock");
            let lock_code = self.run_captured(
                &[scp, "-o".into(), "BatchMode=yes".into(), "-o".into(), "ConnectTimeout=15".into(), lock_remote, lock_tmp.clone()],
                30,
            );
            let lock_size = fs::metadata(&lock_tmp).map(|meta| meta.len()).unwrap_or(0);
            if lock_code == 0 && lock_size > 0 {
                let lock_dest = Path::new(&dest_root).join("auth.json.lock");
                if fs::copy(&lock_tmp, &lock_dest).is_ok() {
                    let _ = set_mode(&lock_dest, 0o600);
                    copied.push("auth.json.lock".to_string());
                }
            }
            let _ = fs::remove_file(&lock_tmp);
        }
        let _ = self.preferred_remote_save(&spec);
        let message = format!(
            "sync-auth-from-remote complete: {label}:{remote_root} → {dest_root} ({} file(s): {})",
            copied.len(),
            copied.join(" ")
        );
        self.output.ok(
            "sync-auth-from-remote",
            &message,
            Some(json!({"remote": label, "files": copied, "count": copied.len()})),
        )
    }

    pub fn sync_to_remote(&mut self, spec: &str) -> i32 {
        let mut spec = spec.to_string();
        if spec.is_empty() {
            spec = self.preferred_remote_load();
        }
        let Some((user, host)) = self.parse_remote_spec(&spec) else {
            return self.output.error(
                &format!("Cannot sync-auth-to-remote: remote '{spec}' is not user@host, IPv4, domain, or user@domain."),
                &format!("{CONSOLE_NAME} sync-auth-to-remote USER@HOST"),
            );
        };
        let source = self.auth_home();
        let auth = Path::new(&source).join("auth.json");
        if !plain_nonempty(&auth) {
            return self.output.error(
                &format!("Cannot sync-auth-to-remote: no auth.json under {source}."),
                &format!("grok login, then {CONSOLE_NAME} sync-auth-to-remote USER@HOST"),
            );
        }
        let scp = self.tool(&["RUSTAGENT_SCP", "GROK_CLI_SCP"], "scp");
        let ssh = self.tool(&["RUSTAGENT_SSH", "GROK_CLI_SSH"], "ssh");
        if !is_exec(&scp) {
            return self.output.error(
                "Cannot sync-auth-to-remote: scp not found.",
                &format!("install openssh-client, then {CONSOLE_NAME} sync-auth-to-remote {spec}"),
            );
        }
        if !is_exec(&ssh) {
            return self.output.error(
                "Cannot sync-auth-to-remote: ssh not found.",
                &format!("install openssh-client, then {CONSOLE_NAME} sync-auth-to-remote {spec}"),
            );
        }
        let target = Self::remote_target(&user, &host);
        self.output.info(&format!("Sending auth.json to {target}"));
        let opts = ["-o", "BatchMode=yes", "-o", "ConnectTimeout=15"];
        if self.run_captured(
            &ssh_argv(&ssh, &target, "mkdir -p .grok && chmod 0700 .grok", &opts),
            30,
        ) != 0
        {
            return self.output.error(
                &format!("Cannot sync-auth-to-remote: cannot prepare ~/.grok on {target}."),
                &format!("ssh {target}, then {CONSOLE_NAME} sync-auth-to-remote {spec}"),
            );
        }
        let auth_s = auth.to_string_lossy().into_owned();
        if self.run_captured(
            &scp_argv(&scp, &auth_s, &format!("{target}:.grok/auth.json"), &opts),
            30,
        ) != 0
        {
            return self.output.error(
                &format!("Cannot sync-auth-to-remote: scp to {target} failed."),
                &format!("ssh {target}, then {CONSOLE_NAME} sync-auth-to-remote {spec}"),
            );
        }
        if self.run_captured(
            &ssh_argv(&ssh, &target, "chmod 0600 .grok/auth.json", &opts),
            30,
        ) != 0
        {
            return self.output.error(
                &format!("Cannot sync-auth-to-remote: cannot set mode 0600 on {target}:.grok/auth.json."),
                &format!("ssh {target}, then {CONSOLE_NAME} sync-auth-to-remote {spec}"),
            );
        }
        let mut copied = vec!["auth.json".to_string()];
        let lock = Path::new(&source).join("auth.json.lock");
        if plain_nonempty(&lock)
            && self.run_captured(
                &scp_argv(&scp, &lock.to_string_lossy(), &format!("{target}:.grok/auth.json.lock"), &opts),
                30,
            ) == 0
            && self.run_captured(
                &ssh_argv(&ssh, &target, "chmod 0600 .grok/auth.json.lock", &opts),
                30,
            ) == 0
        {
            copied.push("auth.json.lock".to_string());
        }
        let _ = self.preferred_remote_save(&spec);
        let message = format!(
            "sync-auth-to-remote complete: {source} → {target}:.grok ({} file(s): {})",
            copied.len(),
            copied.join(" ")
        );
        self.output.ok(
            "sync-auth-to-remote",
            &message,
            Some(json!({"remote": target, "files": copied, "count": copied.len()})),
        )
    }

    pub fn detect_platform(&self) -> String {
        let forced = self.env("RUSTAGENT_PLATFORM");
        if !forced.is_empty() {
            return forced;
        }
        let os_name = match std::env::consts::OS {
            "macos" => "macos",
            "linux" | "android" => "linux",
            "windows" => "windows",
            _ => "",
        };
        if os_name.is_empty() {
            return String::new();
        }
        let arch = match std::env::consts::ARCH {
            "x86_64" => "x86_64",
            "aarch64" => "aarch64",
            _ => "",
        };
        if arch.is_empty() {
            return String::new();
        }
        format!("{os_name}-{arch}")
    }

    pub fn setup(&mut self, force: bool) -> i32 {
        let token = self.detect_platform();
        if token.is_empty() {
            return self.output.error(
                "This host is not linux, macos, or windows on x86_64 or aarch64.",
                &format!("{CONSOLE_NAME} about"),
            );
        }
        let peer = self.peer_path();
        if is_exec(&peer) && !force {
            return self.output.ok(
                "setup",
                &format!("grok is already installed at {peer}."),
                Some(json!({"peer": peer, "skipped": true})),
            );
        }
        if self.which("curl").is_none() && self.runner.is_none() {
            return self.output.error(
                "curl is required to download grok.",
                &format!("Install curl, then {CONSOLE_NAME} setup"),
            );
        }
        let version = {
            let primary = self.read_pointer(VENDOR_BASE);
            if primary.is_empty() { self.read_pointer(VENDOR_FALLBACK) } else { primary }
        };
        if version.is_empty() {
            return self.output.error(
                "The grok version pointer could not be read.",
                &format!("{CONSOLE_NAME} setup --force"),
            );
        }
        let dest_dir = Path::new(&self.auth_home()).join("downloads");
        if fs::create_dir_all(&dest_dir).is_err() {
            return self.output.error(
                &format!("Cannot create {}.", dest_dir.display()),
                &format!("{CONSOLE_NAME} setup"),
            );
        }
        let _ = set_mode(&dest_dir, 0o700);
        let artifact = format!("grok-{version}-{token}");
        let saved = self.fetch_artifact(&version, &artifact, &dest_dir);
        if saved.is_empty() {
            return self.output.error(
                "The grok artifact could not be downloaded.",
                &format!("{CONSOLE_NAME} setup --force"),
            );
        }
        if !self.smoke(&saved) {
            return self.output.error(
                "The downloaded grok binary did not run.",
                &format!("{CONSOLE_NAME} setup --force"),
            );
        }
        let bin_dir = Path::new(&self.auth_home()).join("bin");
        if fs::create_dir_all(&bin_dir).is_err() {
            return self.output.error(
                &format!("Cannot create {}.", bin_dir.display()),
                &format!("{CONSOLE_NAME} setup"),
            );
        }
        let _ = set_mode(&bin_dir, 0o700);
        let vendor = dest_dir.join(format!("grok-{token}"));
        if Path::new(&saved) != vendor.as_path() && fs::copy(&saved, &vendor).is_err() {
            return self.output.error(
                &format!("Cannot write {}.", vendor.display()),
                &format!("{CONSOLE_NAME} setup --force"),
            );
        }
        let _ = set_mode(&vendor, 0o755);
        let link = bin_dir.join("grok");
        let link_s = link.to_string_lossy().into_owned();
        let vendor_s = vendor.to_string_lossy().into_owned();
        if self.host.is_termux() || self.is_android() {
            if !self.write_exec_wrapper(&link_s, &vendor_s, "") {
                return self.output.error(
                    "The Android grok wrapper could not be written.",
                    &format!("{CONSOLE_NAME} setup --force"),
                );
            }
        } else {
            if link.exists() || link.symlink_metadata().is_ok() {
                let _ = fs::remove_file(&link);
            }
            if fs::copy(&vendor, &link).is_err() {
                return self.output.error(
                    &format!("Cannot write {link_s}."),
                    &format!("{CONSOLE_NAME} setup --force"),
                );
            }
            let _ = set_mode(&link, 0o755);
        }
        self.output.ok("setup", &format!("Installed grok {version}."), Some(json!({"peer": link_s})))
    }

    fn read_pointer(&mut self, base: &str) -> String {
        let url = format!("{}/{CHANNEL}", base.trim_end_matches('/'));
        if self.runner.is_some() {
            let reply = self.ask(&["pointer".into(), url]);
            return match reply {
                Some(RunReply::Text(text)) => text.lines().next().unwrap_or("").trim().to_string(),
                _ => String::new(),
            };
        }
        let code = self.run_captured(&["curl".into(), "-fsSL".into(), url], 30);
        if code != 0 {
            return String::new();
        }
        self.last_stdout.lines().next().unwrap_or("").trim().to_string()
    }

    fn fetch_artifact(&mut self, version: &str, artifact: &str, dest_dir: &Path) -> String {
        for suffix in [".zst", ".gz", ""] {
            let name = format!("{artifact}{suffix}");
            for base in [VENDOR_BASE, VENDOR_FALLBACK] {
                let url = format!("{}/{version}/{name}", base.trim_end_matches('/'));
                let dest = dest_dir.join(&name);
                let dest_s = dest.to_string_lossy().into_owned();
                if self.runner.is_some() {
                    let reply = self.ask(&["fetch".into(), url, dest_s.clone()]);
                    let ok = match reply {
                        Some(RunReply::Yes(flag)) => flag,
                        Some(RunReply::Code(0)) => true,
                        Some(RunReply::Text(text)) => !text.trim().is_empty(),
                        _ => false,
                    };
                    if ok {
                        return dest_s;
                    }
                    continue;
                }
                let code = self.run_captured(
                    &["curl".into(), "-fsSL".into(), "-o".into(), dest_s.clone(), url],
                    120,
                );
                if code == 0 && fs::metadata(&dest).map(|meta| meta.len() > 0).unwrap_or(false) {
                    return dest_s;
                }
            }
        }
        String::new()
    }

    fn smoke(&mut self, path: &str) -> bool {
        if self.runner.is_some() {
            let reply = self.ask(&["smoke".into(), path.to_string()]);
            return match reply {
                Some(RunReply::Yes(flag)) => flag,
                Some(RunReply::Code(0)) => true,
                Some(RunReply::Text(text)) => !text.trim().is_empty(),
                _ => false,
            };
        }
        self.run_captured(&[path.to_string(), "--version".into()], 30) == 0
    }

    fn is_android(&self) -> bool {
        if std::env::consts::OS == "android" {
            return true;
        }
        fs::read_to_string("/proc/sys/kernel/osrelease")
            .unwrap_or_default()
            .to_ascii_lowercase()
            .contains("android")
    }

    pub fn write_exec_wrapper(&self, path: &str, vendor: &str, resolv: &str) -> bool {
        if path.is_empty() || vendor.is_empty() {
            return false;
        }
        let quoted = quote_sh(vendor);
        let bind = if resolv.is_empty() {
            String::new()
        } else {
            format!("-b {}:{} ", quote_sh(resolv), "/etc/resolv.conf")
        };
        let body = WRAPPER_TEMPLATE.replace("__QUOTED__", &quoted).replace("__BIND__", &bind);
        let parent = Path::new(path).parent();
        if let Some(parent) = parent {
            if !parent.as_os_str().is_empty() && fs::create_dir_all(parent).is_err() {
                return false;
            }
            let _ = set_mode(parent, 0o700);
        }
        if fs::write(path, body).is_err() {
            return false;
        }
        set_mode(Path::new(path), 0o755) && fs::metadata(path).map(|meta| meta.len() > 0).unwrap_or(false)
    }

    pub fn update_grok(&mut self) -> i32 {
        let peer = self.peer_path();
        if !is_exec(&peer) {
            return self.output.error(
                &format!("grok is not installed, so {CONSOLE_NAME} cannot update it."),
                &format!("{CONSOLE_NAME} setup"),
            );
        }
        self.setup(true)
    }

    pub fn reinstall(&mut self) -> i32 {
        let peer = self.peer_path();
        let link = Path::new(&self.auth_home()).join("bin").join("grok");
        for path in [PathBuf::from(&peer), link] {
            let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            if path.symlink_metadata().is_ok() && !name.starts_with("auth.") {
                if fs::remove_file(&path).is_err() {
                    return self.output.error(
                        &format!("Cannot remove {}.", path.display()),
                        &format!("{CONSOLE_NAME} reinstall"),
                    );
                }
            }
        }
        self.setup(true)
    }

    pub fn run(&mut self, args: &[String]) -> i32 {
        if self.output.json_mode {
            return self.output.error(
                "run starts grok in the terminal, so --json cannot be used.",
                &format!("{CONSOLE_NAME} run"),
            );
        }
        let peer = self.peer_path();
        if !is_exec(&peer) {
            return self.output.error(
                &format!("grok is not installed, so {CONSOLE_NAME} cannot start it."),
                &format!("{CONSOLE_NAME} setup"),
            );
        }
        let mut argv = vec![peer.clone()];
        if !args.iter().any(|arg| arg == "--no-auto-update") {
            argv.push("--no-auto-update".to_string());
        }
        argv.extend(args.iter().cloned());
        if self.runner.is_some() {
            let reply = self.ask(&argv);
            return match reply {
                Some(RunReply::Code(code)) => code,
                Some(RunReply::Yes(false)) => 1,
                Some(_) => 0,
                None => 1,
            };
        }
        let code = self.run_status(&argv);
        if code < 0 {
            return self.output.error(
                &format!("Could not start grok at {peer}."),
                &format!("{CONSOLE_NAME} setup"),
            );
        }
        code
    }

    fn run_status(&self, argv: &[String]) -> i32 {
        if argv.is_empty() {
            return -1;
        }
        let mut command = Command::new(&argv[0]);
        command.args(&argv[1..]);
        for (key, value) in &self.environ {
            command.env(key, value);
        }
        match command.status() {
            Ok(status) => status.code().unwrap_or(1),
            Err(_) => -1,
        }
    }

    pub fn crontab_lines(&self) -> (String, String) {
        let binary = self.global_bin();
        let backup = self.env("RUSTAGENT_CRON_BACKUP");
        let backup = if backup.is_empty() {
            format!("*/30 * * * * sudo {binary} backup")
        } else {
            backup
        };
        let sync = self.env("RUSTAGENT_CRON_SYNC");
        let sync = if sync.is_empty() {
            format!("45 * * * * {binary} sync-auth")
        } else {
            sync
        };
        (backup, sync)
    }

    fn grant_present(&self) -> bool {
        if self.env("RUSTAGENT_SUDO_GRANT") == "1" {
            return true;
        }
        let user = self.login_user();
        let path = self.env("RUSTAGENT_SUDOERS_FILE");
        let path = if path.is_empty() {
            format!("/etc/sudoers.d/{CONSOLE_NAME}-{user}")
        } else {
            path
        };
        let needle = format!("NOPASSWD: {} backup", self.global_bin());
        fs::read_to_string(path).map(|text| text.contains(&needle)).unwrap_or(false)
    }

    pub fn add_crontab(&mut self) -> i32 {
        let binary = self.global_bin();
        let user = self.login_user();
        if !is_exec(&binary) && self.env("RUSTAGENT_ASSUME_BIN") != "1" {
            return self.output.error(
                &format!("Cannot add crontab: need the global binary {binary}."),
                &format!("cargo install {crate}, then {CONSOLE_NAME} add-crontab", crate = identity::CRATE_NAME),
            );
        }
        if !self.grant_present() {
            return self.output.error(
                &format!("Cannot add crontab: no passwordless sudo {binary} backup grant for this login ({user})."),
                &format!("{CONSOLE_NAME} generate-sudoer-request"),
            );
        }
        let cron = self.tool(&["RUSTAGENT_CRONTAB", "GROK_CLI_CRONTAB"], "crontab");
        if !is_exec(&cron) {
            return self.output.error(
                "Cannot add crontab: crontab command not found.",
                &format!("install cron, then {CONSOLE_NAME} add-crontab"),
            );
        }
        let (backup, sync) = self.crontab_lines();
        let mut listed = String::new();
        if self.runner.is_some() {
            if let Some(RunReply::Text(text)) = self.ask(&["crontab-list".into(), cron.clone()]) {
                listed = text;
            }
        } else {
            let code = self.run_captured(&[cron.clone(), "-l".into()], 15);
            if code == 0 {
                listed = self.last_stdout.clone();
            }
        }
        if listed.starts_with("no crontab") {
            listed.clear();
        }
        let have_backup = listed.contains(&format!("{binary} backup")) || listed.contains(&format!("sudo {binary} backup"));
        let have_sync = listed.contains(&format!("{binary} sync-auth"));
        if have_backup && have_sync {
            return self.output.ok(
                "add-crontab",
                &format!("add-crontab: backup and sync-auth jobs already present for {user}"),
                Some(json!({"already_present": true, "backup": backup, "sync": sync})),
            );
        }
        let mut lines: Vec<String> = listed.lines().filter(|line| !line.trim().is_empty()).map(|s| s.to_string()).collect();
        if !have_backup {
            lines.push(backup.clone());
        }
        if !have_sync {
            lines.push(sync.clone());
        }
        let Ok(tmp) = self.storage.mktemp("crontab") else {
            return self.output.error(
                "Cannot add crontab: scratch file failed.",
                &format!("{CONSOLE_NAME} add-crontab"),
            );
        };
        if fs::write(&tmp, format!("{}\n", lines.join("\n"))).is_err() {
            return self.output.error(
                "Cannot add crontab: scratch file failed.",
                &format!("{CONSOLE_NAME} add-crontab"),
            );
        }
        let code = if self.runner.is_some() {
            match self.ask(&["crontab-install".into(), cron, tmp.clone()]) {
                Some(RunReply::Code(code)) => code,
                Some(_) => 0,
                None => 1,
            }
        } else {
            self.run_captured(&[cron, tmp.clone()], 15)
        };
        let _ = fs::remove_file(&tmp);
        if code != 0 {
            return self.output.error(
                "Cannot add crontab: crontab install failed.",
                &format!("check crontab -l, then {CONSOLE_NAME} add-crontab"),
            );
        }
        self.output.ok(
            "add-crontab",
            &format!("add-crontab complete for {user}: backup every 30 minutes; sync-auth at minute 45"),
            Some(json!({"backup": backup, "sync": sync, "already_present": false})),
        )
    }

    pub fn sudoer_document(&self, action: &str, username: Option<&str>) -> Value {
        let user = username.unwrap_or("").to_string();
        let user = if user.is_empty() { self.login_user() } else { user };
        let action = if action == "update" { "update" } else { "add" };
        json!({
            "schema_version": 1,
            "purpose": format!("Allow {user} to run {CONSOLE_NAME} backup as root."),
            "username": user,
            "service": CONSOLE_NAME,
            "action": action,
            "commands": [{
                "runas": "root",
                "tags": ["NOPASSWD"],
                "path": self.global_bin(),
                "args": ["backup"],
            }],
        })
    }

    pub fn sudoer_text(&self, username: Option<&str>) -> String {
        let doc = self.sudoer_document("add", username);
        let purpose = doc["purpose"].as_str().unwrap_or("");
        let user = doc["username"].as_str().unwrap_or("");
        let path = doc["commands"][0]["path"].as_str().unwrap_or("");
        format!("# Purpose: {purpose}\n{user} ALL=(root) NOPASSWD: {path} backup\n")
    }

    pub fn print_sudoers(&mut self) -> i32 {
        let text = self.sudoer_text(None);
        if self.output.json_mode {
            return self.output.ok("print-sudoers", text.trim(), None);
        }
        self.output.raw(&text);
        0
    }

    fn login_user(&self) -> String {
        let user = self.env("USER");
        if !user.is_empty() {
            return user;
        }
        let user = self.env("USERNAME");
        if user.is_empty() { "unknown".to_string() } else { user }
    }

    fn sudoers_d(&self) -> String {
        let value = self.env("RUSTAGENT_SUDOERS_D");
        if value.is_empty() { "/etc/sudoers.d".to_string() } else { value }
    }

    fn installed_sudoers_path(&self) -> String {
        Path::new(&self.sudoers_d())
            .join(format!("{CONSOLE_NAME}-{}", self.login_user()))
            .to_string_lossy()
            .into_owned()
    }

    fn config_dir(&self) -> String {
        let home = self.env("HOME");
        let home = if home.is_empty() { "/tmp".to_string() } else { home };
        Path::new(&home).join(".config").join(CONSOLE_NAME).to_string_lossy().into_owned()
    }

    fn draft_path(&self) -> String {
        Path::new(&self.config_dir())
            .join(format!("sudoers.fragment-{}", self.login_user()))
            .to_string_lossy()
            .into_owned()
    }

    fn list_drafts(&self) -> Vec<String> {
        let folder = self.config_dir();
        let mut found = Vec::new();
        let legacy = Path::new(&folder).join("sudoers.fragment");
        if plain_file(&legacy) {
            found.push(legacy.to_string_lossy().into_owned());
        }
        if let Ok(entries) = fs::read_dir(&folder) {
            let mut names: Vec<String> = entries
                .flatten()
                .map(|ent| ent.file_name().to_string_lossy().into_owned())
                .filter(|name| name.starts_with("sudoers.fragment-") && !name.ends_with(".json"))
                .collect();
            names.sort();
            for name in names {
                let path = Path::new(&folder).join(name);
                if plain_file(&path) {
                    found.push(path.to_string_lossy().into_owned());
                }
            }
        }
        found
    }

    pub fn grant_allows_backup(&self, path: &str) -> bool {
        let Ok(text) = fs::read_to_string(path) else {
            return false;
        };
        let stripped = text.trim_start();
        if stripped.starts_with('{') {
            let Ok(data) = serde_json::from_str::<Value>(&text) else {
                return false;
            };
            let Some(commands) = data.get("commands").and_then(|v| v.as_array()) else {
                return false;
            };
            if commands.is_empty() {
                return false;
            }
            for command in commands {
                let Some(command) = command.as_object() else {
                    return false;
                };
                let args = command.get("args").and_then(|v| v.as_array());
                let Some(args) = args else {
                    return false;
                };
                let args: Vec<&str> = args.iter().filter_map(|v| v.as_str()).collect();
                if args != ["backup"] {
                    return false;
                }
                let target = command.get("path").and_then(|v| v.as_str()).unwrap_or("");
                if target != CONSOLE_NAME && !target.ends_with(&format!("/{CONSOLE_NAME}")) {
                    return false;
                }
            }
            return true;
        }
        let lines: Vec<&str> = text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .collect();
        if lines.len() != 1 {
            return false;
        }
        lines[0].ends_with(&format!(" NOPASSWD: {} backup", self.global_bin()))
            || lines[0].ends_with(&format!(" NOPASSWD: {CONSOLE_NAME} backup"))
    }

    fn request_id(text: &str) -> String {
        if let Some(pos) = text.find("\"request_id\"") {
            let rest = &text[pos..];
            if let Some(start) = rest.find(':') {
                let rest = rest[start + 1..].trim_start();
                if let Some(rest) = rest.strip_prefix('"') {
                    if let Some(end) = rest.find('"') {
                        return rest[..end].to_string();
                    }
                }
            }
        }
        for token in text.split_whitespace() {
            if let Some(value) = token.strip_prefix("request_id=") {
                return value.trim_matches('"').to_string();
            }
        }
        String::new()
    }

    fn admin_script(&self, user: &str, draft: &str, installed: &str, legacy: &str) -> String {
        ADMIN_TEMPLATE
            .replace("__APP__", CONSOLE_NAME)
            .replace("__USER__", user)
            .replace("__DRAFT__", draft)
            .replace("__INSTALLED__", installed)
            .replace("__LEGACY__", legacy)
            .replace("__DEPOSIT__", &self.deposit_root)
    }

    pub fn print_sudoers_install_script(&mut self, script_out: &str) -> i32 {
        if script_out.starts_with("/etc") {
            return self.output.error(
                "Refusing to write admin script under /etc (Type 0).",
                &format!("{CONSOLE_NAME} print-sudoers-install-script"),
            );
        }
        let user = self.login_user();
        let draft = self.draft_path();
        if fs::create_dir_all(self.config_dir()).is_err()
            || fs::write(&draft, self.sudoer_text(Some(&user))).is_err()
        {
            return self.output.error(
                "Cannot write the project sudoers draft.",
                &format!("{CONSOLE_NAME} print-sudoers"),
            );
        }
        let _ = set_mode(Path::new(&self.config_dir()), 0o700);
        let _ = set_mode(Path::new(&draft), 0o600);
        let script_out = if script_out.is_empty() {
            match self.storage.mktemp("sudoers-admin") {
                Ok(path) => path,
                Err(_) => {
                    return self.output.error(
                        "Cannot create the admin script.",
                        &format!("{CONSOLE_NAME} print-sudoers-install-script"),
                    );
                }
            }
        } else {
            let parent = Path::new(script_out).parent().unwrap_or(Path::new("."));
            if fs::create_dir_all(parent).is_err() {
                return self.output.error(
                    &format!("Cannot create {}.", parent.display()),
                    &format!("{CONSOLE_NAME} print-sudoers-install-script"),
                );
            }
            script_out.to_string()
        };
        let installed = self.installed_sudoers_path();
        let legacy = Path::new(&self.sudoers_d()).join(CONSOLE_NAME).to_string_lossy().into_owned();
        let body = self.admin_script(&user, &draft, &installed, &legacy);
        if fs::write(&script_out, body).is_err() {
            return self.output.error(
                &format!("Cannot write {script_out}."),
                &format!("{CONSOLE_NAME} print-sudoers-install-script"),
            );
        }
        let _ = set_mode(Path::new(&script_out), 0o750);
        let message = format!(
            "Wrote admin sudoers script: {script_out}\n\
Project sudoers file (draft): {draft}\n\
Installed path (admin only, per-user): {installed}\n\
sudo sh {script_out} install\n\
sudo sh {script_out} uninstall\n\
sudo sh {script_out} replace\n\
sudo sh {script_out} status"
        );
        self.output.ok(
            "print-sudoers-install-script",
            &message,
            Some(json!({
                "script_path": script_out,
                "project_sudoers_file": draft,
                "installed_path": installed,
                "user": user,
            })),
        )
    }

    pub fn generate_sudoer_request(&mut self, directory: Option<&str>, action: &str) -> i32 {
        let doc = self.sudoer_document(action, None);
        let folder = directory.unwrap_or(".").to_string();
        let stamp = local_ymd();
        let user = doc["username"].as_str().unwrap_or("unknown");
        let action = doc["action"].as_str().unwrap_or("add");
        let mut number = 1i32;
        let path = loop {
            let base = format!("sudoer-{stamp}-{CONSOLE_NAME}-{user}-{action}-{number}.json");
            let path = Path::new(&folder).join(base);
            if !path.exists() {
                break path;
            }
            number += 1;
        };
        let body = match serde_json::to_string_pretty(&doc) {
            Ok(text) => format!("{text}\n"),
            Err(_) => {
                return self.output.error(
                    "Cannot write the sudoer request.",
                    &format!("{CONSOLE_NAME} generate-sudoer-request"),
                );
            }
        };
        if fs::write(&path, body).is_err() {
            return self.output.error(
                "Cannot write the sudoer request.",
                &format!("{CONSOLE_NAME} generate-sudoer-request"),
            );
        }
        let shown = path.to_string_lossy().into_owned();
        self.output.ok("generate-sudoer-request", &shown, Some(json!({"path": shown})))
    }

    pub fn submit_sudoer_request(&mut self, path: &str) -> i32 {
        let user = self.login_user();
        let cli = self.tool(&["RUSTAGENT_SUDOER_CLI", "SUDOER_CLI"], "sudoer-cli");
        if !is_exec(&cli) {
            return self.output.error(
                "sudoer-cli not found (set SUDOER_CLI or install sudoer-cli). Submit does not write /etc.",
                &format!("{CONSOLE_NAME} print-sudoers-install-script"),
            );
        }
        let mut inbound = self.env_first(&["RUSTAGENT_SUDOER_INBOUND", "SUDOER_QUEUE_INBOUND"]);
        if inbound.is_empty() && Path::new("/var/sudoer-cli/sudoer-request").is_dir() {
            inbound = "/var/sudoer-cli/sudoer-request".to_string();
        }
        if inbound.is_empty() || !Path::new(&inbound).is_dir() {
            return self.output.error(
                "sudoer inbound queue not found (preferred /var/sudoer-cli/sudoer-request).",
                "Ask an admin to run: sudo sudoer-cli setup",
            );
        }
        if !dir_writable(&inbound) {
            return self.output.error(
                &format!("sudoer inbound not writable: {inbound}."),
                "Check mode 3773 on public inbound.",
            );
        }
        let host_path = self.installed_sudoers_path();
        let host_present = Path::new(&host_path).is_file();
        let mut action = self.env("RUSTAGENT_SUBMIT_ACTION");
        if action.is_empty() {
            action = if host_present { "update".to_string() } else { "add".to_string() };
        }
        if action != "add" && action != "update" {
            return self.output.error(
                &format!("submit-sudoer-request action must be add or update (got {action})."),
                &format!("{CONSOLE_NAME} submit-sudoer-request"),
            );
        }
        let mut cleanup = false;
        let source = if !path.is_empty() {
            if Path::new(path).symlink_metadata().map(|m| m.file_type().is_symlink()).unwrap_or(false) {
                return self.output.error(
                    &format!("refusing symlink sudoers input: {path}"),
                    &format!("{CONSOLE_NAME} generate-sudoer-request"),
                );
            }
            if !Path::new(path).is_file() {
                return self.output.error(
                    &format!("sudoers file not found: {path}"),
                    &format!("{CONSOLE_NAME} generate-sudoer-request"),
                );
            }
            path.to_string()
        } else {
            let Ok(source) = self.storage.mktemp("sudoer-json") else {
                return self.output.error(
                    "Cannot write temp sudoer JSON.",
                    &format!("{CONSOLE_NAME} generate-sudoer-request"),
                );
            };
            let document = self.sudoer_document(&action, Some(&user));
            let body = match serde_json::to_string(&document) {
                Ok(text) => format!("{text}\n"),
                Err(_) => {
                    return self.output.error(
                        "Cannot write temp sudoer JSON.",
                        &format!("{CONSOLE_NAME} generate-sudoer-request"),
                    );
                }
            };
            if fs::write(&source, body).is_err() {
                return self.output.error(
                    "Cannot write temp sudoer JSON.",
                    &format!("{CONSOLE_NAME} generate-sudoer-request"),
                );
            }
            cleanup = true;
            source
        };
        if !self.grant_allows_backup(&source) {
            if cleanup {
                let _ = fs::remove_file(&source);
            }
            return self.output.error(
                "Queued sudo request is incomplete: it does not allow backup.",
                &format!("{CONSOLE_NAME} generate-sudoer-request && {CONSOLE_NAME} submit-sudoer-request <path>"),
            );
        }
        let verb = if action == "update" { "update-sudoer-request" } else { "add-sudoer-request" };
        let purpose = format!("Allow {user} to run {CONSOLE_NAME} backup as root.");
        let mut argv = vec![
            cli.clone(),
            verb.to_string(),
            "--file".into(),
            source.clone(),
            "--purpose".into(),
            purpose,
            "--service".into(),
            CONSOLE_NAME.to_string(),
        ];
        if self.output.json_mode {
            argv.insert(1, "--json".into());
        }
        let code = self.run_captured(&argv, 30);
        let stdout = self.last_stdout.clone();
        let stderr = self.last_stderr.clone();
        if cleanup {
            let _ = fs::remove_file(&source);
        }
        if code != 0 {
            let detail = if !stderr.trim().is_empty() {
                stderr.trim().to_string()
            } else if !stdout.trim().is_empty() {
                stdout.trim().to_string()
            } else {
                format!("exit {code}")
            };
            return self.output.error(
                &format!("sudoer-cli {verb} failed: {detail}"),
                &format!("{CONSOLE_NAME} generate-sudoer-request"),
            );
        }
        let request_id = Self::request_id(&stdout);
        if !request_id.is_empty() {
            let queued = Path::new(&inbound).join(&request_id);
            if queued.is_file() && !self.grant_allows_backup(&queued.to_string_lossy()) {
                return self.output.error(
                    &format!("Queued sudo request is incomplete: it does not allow backup. Do not approve {request_id}."),
                    &format!("{CONSOLE_NAME} generate-sudoer-request && {CONSOLE_NAME} submit-sudoer-request <path>"),
                );
            }
        }
        let mut message = format!("Submitted {action} request via {cli}");
        if !request_id.is_empty() {
            message = format!("{message}\nrequest_id={request_id}");
        }
        if host_present {
            message = format!("{message}\nhost_fragment={host_path}");
        }
        message = format!("{message}\ninbound={inbound}\nThis CLI did not write /etc. Approver must approve the request.");
        self.output.ok(
            "submit-sudoer-request",
            &message,
            Some(json!({
                "request_id": request_id,
                "action": action,
                "inbound": inbound,
                "host_fragment_present": host_present,
            })),
        )
    }

    pub fn remove_project_sudoers(&mut self, path: &str, force: bool) -> i32 {
        if path == "/etc" || path.starts_with("/etc/") {
            return self.output.error(
                "Refusing to remove paths under /etc as Type 0. This command only deletes the user draft.",
                &format!("{CONSOLE_NAME} print-sudoers-install-script"),
            );
        }
        let drafts = self.list_drafts();
        let path = if path.is_empty() {
            if drafts.len() > 1 {
                return self.output.error(
                    &format!("remove-project-sudoers: multiple drafts present; specify path argument (non-interactive).\n{}", drafts.join("\n")),
                    &format!("{CONSOLE_NAME} remove-project-sudoers --force <path>"),
                );
            }
            if drafts.is_empty() { self.draft_path() } else { drafts[0].clone() }
        } else {
            path.to_string()
        };
        let installed = self.installed_sudoers_path();
        if fs::symlink_metadata(&path).is_err() {
            let mut message = format!("Project sudoers draft not present (nothing to remove): {path}");
            if Path::new(&installed).is_file() {
                message.push_str(&format!("\nHost elevation STILL ACTIVE (separate from draft): {installed}"));
            }
            return self.output.ok(
                "remove-project-sudoers",
                &message,
                Some(json!({"path": path, "status": "already_absent", "installed_path": installed})),
            );
        }
        if !force {
            return self.output.error(
                "remove-project-sudoers requires confirmation.",
                &format!("{CONSOLE_NAME} remove-project-sudoers --force"),
            );
        }
        if fs::remove_file(&path).is_err() {
            return self.output.error(
                &format!("Failed to remove {path}"),
                &format!("{CONSOLE_NAME} remove-project-sudoers --force {path}"),
            );
        }
        let mut message = format!("Removed project-sudoers-file draft: {path}");
        if Path::new(&installed).is_file() {
            message.push_str(&format!("\nHost elevation STILL ACTIVE (separate from draft): {installed}"));
            message.push_str("\nremove-project-sudoers only deletes the user draft.");
        }
        self.output.ok(
            "remove-project-sudoers",
            &message,
            Some(json!({"path": path, "status": "removed", "installed_path": installed})),
        )
    }

    fn sync_startup(&mut self) {
        self.startup.environ = self.environ.clone();
        self.startup.host = Host::from_map(self.environ.clone());
        self.startup.output.json_mode = self.output.json_mode;
        self.startup.output.quiet = self.output.quiet;
        self.startup.output.capture = true;
        self.startup.euid = Some(self.euid);
    }

    fn merge_startup(&mut self) {
        self.output.out.push_str(&self.startup.output.out);
        self.output.err.push_str(&self.startup.output.err);
        self.startup.output.out.clear();
        self.startup.output.err.clear();
    }

    pub fn install_startup(&mut self) -> i32 {
        self.sync_startup();
        let code = self.startup.path_add_shell();
        self.merge_startup();
        code
    }

    pub fn uninstall_startup(&mut self, force: bool) -> i32 {
        if !force {
            return self.output.error(
                "uninstall edits this login's startup-file stickers.",
                &format!("{CONSOLE_NAME} uninstall --force"),
            );
        }
        self.sync_startup();
        let _ = self.startup.cleanup_path();
        self.merge_startup();
        self.output.ok(
            "uninstall",
            &format!("Removed this product's startup-file stickers. Type: {CONSOLE_NAME} self-uninstall --force"),
            None,
        )
    }

    pub fn rc_test(&mut self, root: &str, file_name: &str, case_name: &str, feature: &str) -> i32 {
        self.sync_startup();
        let file_name = if file_name.is_empty() { "bashrc" } else { file_name };
        let case_name = if case_name.is_empty() { "create" } else { case_name };
        let code = self.startup.path_rc_test(root, file_name, case_name, feature);
        self.merge_startup();
        code
    }

    pub fn where_is_me(&mut self) -> i32 {
        let binary = self.global_bin();
        let peer = self.peer_path();
        let message = format!("{CONSOLE_NAME}: {binary}\ngrok: {peer}");
        self.output.ok(
            "where-is-me",
            &message,
            Some(json!({"rust_agent": binary, "grok": peer})),
        )
    }

    pub fn menu_text(&mut self, kind: &str) -> String {
        match kind {
            "sync-auth-from-remote" | "sync-auth-to-remote" => format!("Type: {CONSOLE_NAME} {kind} user@host"),
            "setup" => {
                let peer = self.peer_path();
                if Path::new(&peer).is_file() {
                    format!("grok is already installed. Choose keep, reinstall, or exit.\nType: {CONSOLE_NAME} setup")
                } else {
                    format!("Type: {CONSOLE_NAME} setup")
                }
            }
            "run" => format!("Type: {CONSOLE_NAME} run"),
            "backup" => format!("Type: sudo -n {CONSOLE_NAME} backup"),
            "add-crontab" => {
                let (backup, sync) = self.crontab_lines();
                format!("{backup}\n{sync}")
            }
            "where-is-me" => format!("{CONSOLE_NAME}: {}\ngrok: {}", self.global_bin(), self.peer_path()),
            "check-session" => self.session_word(),
            "reinstall" => format!("Type: {CONSOLE_NAME} reinstall"),
            "update-grok" => format!("Type: {CONSOLE_NAME} update-grok"),
            _ => format!("Type: {CONSOLE_NAME} {kind}"),
        }
    }

    pub fn captured(&mut self, verb: &str, args: &[String], force: bool) -> String {
        let code = self.dispatch(verb, args, force);
        let out = self.output.out.trim().to_string();
        let err = self.output.err.trim().to_string();
        self.output.out.clear();
        self.output.err.clear();
        let mut parts = Vec::new();
        if !out.is_empty() {
            parts.push(out);
        }
        if !err.is_empty() {
            parts.push(err);
        }
        if parts.is_empty() {
            if code == 0 { "ok".to_string() } else { "failed".to_string() }
        } else {
            parts.join("\n")
        }
    }

    pub fn dispatch(&mut self, verb: &str, args: &[String], force: bool) -> i32 {
        self.output.out.clear();
        self.output.err.clear();
        match verb {
            "check-session" => self.check_session(),
            "backup" => self.backup(),
            "sync-auth" => self.sync_auth(),
            "sync-auth-from-remote" => self.sync_from_remote(args.first().map(String::as_str).unwrap_or("")),
            "sync-auth-to-remote" => self.sync_to_remote(args.first().map(String::as_str).unwrap_or("")),
            "setup" => self.setup(force),
            "update-grok" => self.update_grok(),
            "reinstall" => self.reinstall(),
            "run" => self.run(args),
            "add-crontab" => self.add_crontab(),
            "print-sudoers" => self.print_sudoers(),
            "print-sudoers-install-script" => {
                self.print_sudoers_install_script(args.first().map(String::as_str).unwrap_or(""))
            }
            "remove-project-sudoers" => {
                self.remove_project_sudoers(args.first().map(String::as_str).unwrap_or(""), force)
            }
            "generate-sudoer-request" => self.generate_sudoer_request(None, "add"),
            "submit-sudoer-request" => self.submit_sudoer_request(args.first().map(String::as_str).unwrap_or("")),
            "where-is-me" => self.where_is_me(),
            "ensure" => self.output.ok(verb, &format!("Type: {CONSOLE_NAME} self-install"), None),
            "install" => self.install_startup(),
            "uninstall" => self.uninstall_startup(force),
            "rc-test" => self.output.error(
                "rc-test requires --root a tmp/cache folder.",
                &format!("{CONSOLE_NAME} rc-test --root \"$tmpdir\" --file zprofile --case create"),
            ),
            _ => self.output.error(
                &format!("Unknown verb '{verb}'."),
                &format!("{CONSOLE_NAME} help"),
            ),
        }
    }
}

pub fn is_menu_kind(kind: &str) -> bool {
    matches!(
        kind,
        "backup"
            | "sync-auth"
            | "sync-auth-from-remote"
            | "sync-auth-to-remote"
            | "add-crontab"
            | "setup"
            | "reinstall"
            | "update-grok"
            | "run"
            | "check-session"
            | "where-is-me"
            | "print-sudoers"
            | "print-sudoers-install-script"
            | "remove-project-sudoers"
            | "generate-sudoer-request"
            | "submit-sudoer-request"
    )
}

pub fn unavailable_lines(host: &Host, logged_in: bool) -> Vec<String> {
    let mut lines = Vec::new();
    if host.normal_user_only() {
        lines.push(format!(
            "backup, sync-auth and sudoers features are not available in {}.",
            host.label()
        ));
    }
    if logged_in {
        lines.push(
            "sync-auth and sync-auth-from-remote features are not available for logged-in environment."
                .to_string(),
        );
    } else {
        lines.push("backup features are not available for logged-out environment.".to_string());
    }
    lines
}

fn ssh_argv(ssh: &str, target: &str, command: &str, opts: &[&str]) -> Vec<String> {
    let mut argv = vec![ssh.to_string()];
    for opt in opts {
        argv.push((*opt).to_string());
    }
    argv.push(target.to_string());
    argv.push(command.to_string());
    argv
}

fn scp_argv(scp: &str, source: &str, dest: &str, opts: &[&str]) -> Vec<String> {
    let mut argv = vec![scp.to_string()];
    for opt in opts {
        argv.push((*opt).to_string());
    }
    argv.push(source.to_string());
    argv.push(dest.to_string());
    argv
}

fn read_pipe(pipe: Option<impl std::io::Read>) -> String {
    let mut pipe = match pipe {
        Some(pipe) => pipe,
        None => return String::new(),
    };
    let mut buf = String::new();
    let _ = std::io::Read::read_to_string(&mut pipe, &mut buf);
    buf
}

fn cookies_still_good(data: &Value) -> bool {
    let Some(object) = data.as_object() else {
        return false;
    };
    if object.is_empty() {
        return false;
    }
    if secret_still_good(data) {
        return true;
    }
    object.values().any(|value| value.is_object() && secret_still_good(value))
}

fn secret_still_good(node: &Value) -> bool {
    if let Some(refresh) = node.get("refresh_token").and_then(|v| v.as_str()) {
        if !refresh.is_empty() {
            return true;
        }
    }
    let Some(key) = node.get("key").and_then(|v| v.as_str()) else {
        return false;
    };
    if key.is_empty() {
        return false;
    }
    match node.get("expires_at") {
        None | Some(Value::Null) => true,
        Some(Value::String(value)) if value.is_empty() => true,
        Some(value) => expires_in_future(&value.to_string().trim_matches('"').to_string()),
    }
}

fn expires_in_future(value: &str) -> bool {
    let text = value.trim().trim_matches('"').replace('Z', "+00:00");
    let Some((date, rest)) = text.split_once('T') else {
        return false;
    };
    let mut date_parts = date.split('-');
    let (Some(year), Some(month), Some(day)) = (date_parts.next(), date_parts.next(), date_parts.next()) else {
        return false;
    };
    let time = rest.split(['+', '-']).next().unwrap_or("");
    let mut time_parts = time.split(':');
    let (Some(hour), Some(minute), Some(second)) = (time_parts.next(), time_parts.next(), time_parts.next()) else {
        return false;
    };
    let second = second.split('.').next().unwrap_or(second);
    let Ok(year) = year.parse::<i32>() else { return false };
    let Ok(month) = month.parse::<u32>() else { return false };
    let Ok(day) = day.parse::<u32>() else { return false };
    let Ok(hour) = hour.parse::<u32>() else { return false };
    let Ok(minute) = minute.parse::<u32>() else { return false };
    let Ok(second) = second.parse::<u32>() else { return false };
    let Some(stamp) = civil_utc(year, month, day, hour, minute, second) else {
        return false;
    };
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
    stamp > now
}

fn civil_utc(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> Option<i64> {
    if !(1..=12).contains(&month) || day == 0 || hour > 23 || minute > 59 || second > 60 {
        return None;
    }
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u32;
    let mp = if month > 2 { month - 3 } else { month + 9 };
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = (era as i64) * 146097 + doe as i64 - 719468;
    Some(days * 86400 + hour as i64 * 3600 + minute as i64 * 60 + second as i64)
}

fn local_ymd() -> String {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
    let days = secs.div_euclid(86400);
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { y + 1 } else { y };
    format!("{year:04}{month:02}{day:02}")
}

fn host_is_ipv4(host: &str) -> bool {
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    parts.iter().all(|part| part.parse::<u16>().map(|n| n <= 255).unwrap_or(false) && !part.is_empty() && part.chars().all(|ch| ch.is_ascii_digit()))
}

fn host_is_dns(host: &str) -> bool {
    if host.is_empty() || host.starts_with('.') || host.ends_with('.') || host.contains("..") {
        return false;
    }
    if host.starts_with('-') || host.ends_with('-') {
        return false;
    }
    if host.chars().any(|ch| !(ch.is_ascii_alphanumeric() || ch == '.' || ch == '-')) {
        return false;
    }
    host.chars().any(|ch| ch.is_ascii_alphabetic())
}

fn which_in(name: &str, path_env: &str) -> Option<String> {
    if name.contains('/') {
        return if is_exec(name) { Some(name.to_string()) } else { None };
    }
    for dir in path_env.split(':') {
        if dir.is_empty() {
            continue;
        }
        let candidate = Path::new(dir).join(name);
        let text = candidate.to_string_lossy().into_owned();
        if is_exec(&text) {
            return Some(text);
        }
    }
    None
}

fn is_exec(path: &str) -> bool {
    if path.is_empty() {
        return false;
    }
    let Ok(meta) = fs::metadata(path) else {
        return false;
    };
    if !meta.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        meta.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

fn set_mode(path: &Path, mode: u32) -> bool {
    #[cfg(unix)]
    {
        fs::set_permissions(path, fs::Permissions::from_mode(mode)).is_ok()
    }
    #[cfg(not(unix))]
    {
        let _ = (path, mode);
        true
    }
}

fn auth_files(dir: &str) -> Vec<String> {
    let mut names = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return names;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with("auth.") {
            names.push(entry.path().to_string_lossy().into_owned());
        }
    }
    names.sort();
    names
}

fn plain_nonempty(path: &Path) -> bool {
    if path.symlink_metadata().map(|meta| meta.file_type().is_symlink()).unwrap_or(true) {
        return false;
    }
    fs::metadata(path).map(|meta| meta.is_file() && meta.len() > 0).unwrap_or(false)
}

fn plain_file(path: &Path) -> bool {
    if path.symlink_metadata().map(|meta| meta.file_type().is_symlink()).unwrap_or(true) {
        return false;
    }
    fs::metadata(path).map(|meta| meta.is_file()).unwrap_or(false)
}

fn dir_writable(path: &str) -> bool {
    let probe = Path::new(path).join(".write-probe");
    match fs::File::create(&probe) {
        Ok(_) => {
            let _ = fs::remove_file(probe);
            true
        }
        Err(_) => false,
    }
}

fn quote_sh(text: &str) -> String {
    format!("'{}'", text.replace('\'', "'\\''"))
}

fn passwd_home(user: &str) -> Option<String> {
    let text = fs::read_to_string("/etc/passwd").ok()?;
    for line in text.lines() {
        let mut parts = line.split(':');
        if parts.next() == Some(user) {
            return parts.nth(4).map(|home| home.to_string());
        }
    }
    None
}

fn current_euid() -> u32 {
    #[cfg(unix)]
    {
        extern "C" {
            fn geteuid() -> u32;
        }
        unsafe { geteuid() }
    }
    #[cfg(not(unix))]
    {
        1
    }
}
