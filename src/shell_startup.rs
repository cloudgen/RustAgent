//! Shell startup files: shared USER_BIN line and the missing-file matrix.
//! requirement-shell-startup-files, requirement-shell-zshenv,
//! requirement-shell-path-and-shell-support, requirement-shell-alpine.
//! This module writes this login's notes only. It does not delete the files.

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

use crate::host::Host;
use crate::identity::CONSOLE_NAME;
use crate::output::Output;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub struct ShellStartup {
    pub version: String,
    pub output: Output,
    pub environ: HashMap<String, String>,
    pub host: Host,
    pub euid: Option<u32>,
}

impl ShellStartup {
    pub fn new(version: &str, environ: HashMap<String, String>, euid: Option<u32>) -> Self {
        let host = Host::from_map(environ.clone());
        Self {
            version: version.to_string(),
            output: Output::new(),
            environ,
            host,
            euid,
        }
    }

    fn plain(&mut self, text: &str) {
        if self.output.json_mode {
            return;
        }
        self.output.raw(text);
    }

    fn warn(&mut self, text: &str) {
        if self.output.json_mode {
            return;
        }
        self.output.raw(&format!("[WARN] {text}"));
    }

    fn home(&self) -> String {
        let value = self.environ.get("HOME").cloned().unwrap_or_default();
        if value.is_empty() {
            "/tmp".to_string()
        } else {
            value
        }
    }

    fn user_bin(&self) -> String {
        let value = self.environ.get("USER_BIN").cloned().unwrap_or_default();
        if !value.is_empty() {
            return value;
        }
        Path::new(&self.home())
            .join(".local")
            .join("bin")
            .to_string_lossy()
            .into_owned()
    }

    fn path_line(&self) -> String {
        format!("export PATH=\"{}:$PATH\"", self.user_bin())
    }

    fn fish_line(&self) -> String {
        format!("set -gx PATH {} $PATH", self.user_bin())
    }

    fn comment(&self) -> String {
        format!("# Added by {CONSOLE_NAME} installer ({})", self.version)
    }

    fn is_root(&self) -> bool {
        if let Some(euid) = self.euid {
            return euid == 0;
        }
        current_euid() == 0
    }

    fn shell_is_zsh(&self) -> bool {
        Path::new(self.environ.get("SHELL").map(|s| s.as_str()).unwrap_or(""))
            .file_name()
            .and_then(|name| name.to_str())
            == Some("zsh")
    }

    fn target(&self, name: &str, default: &str) -> (bool, String) {
        let explicit = self.environ.contains_key(name);
        let value = self.environ.get(name).cloned().unwrap_or_default();
        if explicit && !value.is_empty() {
            (true, value)
        } else {
            (explicit, default.to_string())
        }
    }

    fn contains(&self, path: &str, text: &str) -> bool {
        fs::read_to_string(path).map(|body| body.contains(text)).unwrap_or(false)
    }

    fn create(&self, path: &str, header: &str) -> bool {
        let mut body = header.to_string();
        if !body.ends_with('\n') {
            body.push('\n');
        }
        if fs::write(path, body).is_err() {
            return false;
        }
        #[cfg(unix)]
        {
            if fs::set_permissions(path, fs::Permissions::from_mode(0o644)).is_err() {
                return false;
            }
        }
        true
    }

    fn append(&self, path: &str, lines: &[&str]) -> bool {
        let mut file = match fs::OpenOptions::new().create(true).append(true).open(path) {
            Ok(file) => file,
            Err(_) => return false,
        };
        if file.write_all(b"\n").is_err() {
            return false;
        }
        for line in lines {
            if file.write_all(line.as_bytes()).is_err() || file.write_all(b"\n").is_err() {
                return false;
            }
        }
        true
    }

    fn note_or_add(&mut self, path: &str, added: &str, noted: &str, created: bool) -> bool {
        let path_line = self.path_line();
        let comment = self.comment();
        if !self.contains(path, &path_line) {
            if !self.append(path, &[&comment, &path_line]) {
                return false;
            }
            self.plain(added);
            return true;
        }
        if !self.contains(path, &comment) {
            if !self.append(path, &[&comment]) {
                return false;
            }
            self.plain(noted);
            return true;
        }
        created
    }

    fn create_failure(&mut self, path: &str) -> bool {
        self.warn(&format!(
            "Could not create {path}. Next: create that file as this login, then re-run {CONSOLE_NAME} install."
        ));
        false
    }

    pub fn path_add_bashrc(&mut self) -> bool {
        let default = Path::new(&self.home()).join(".bashrc").to_string_lossy().into_owned();
        let (_explicit, path) = self.target("BASHRC", &default);
        let mut created = false;
        if !Path::new(&path).is_file() {
            let header = format!("# Interactive rc created by {CONSOLE_NAME} installer ({})\n", self.version);
            if !self.create(&path, &header) {
                return self.create_failure(&path);
            }
            created = true;
            let message = format!("→ Created {path}");
            self.plain(&message);
        }
        let added = format!("→ Added {} to PATH for bash", self.user_bin());
        let noted = format!("→ Noted {CONSOLE_NAME} installer on bash PATH");
        self.note_or_add(&path, &added, &noted, created)
    }

    pub fn path_add_zshenv(&mut self) -> bool {
        let default = Path::new(&self.home()).join(".zshenv").to_string_lossy().into_owned();
        let (explicit, path) = self.target("ZSHENV", &default);
        let mut created = false;
        if !Path::new(&path).is_file() {
            if self.host.normal_user_only() && !explicit {
                return false;
            }
            if !explicit && !self.shell_is_zsh() {
                return false;
            }
            let header = format!("# Always-on zsh rc created by {CONSOLE_NAME} installer ({})\n", self.version);
            if !self.create(&path, &header) {
                return self.create_failure(&path);
            }
            created = true;
            let message = format!("→ Created {path}");
            self.plain(&message);
        }
        let added = format!("→ Added {} to PATH for zsh (.zshenv)", self.user_bin());
        let noted = format!("→ Noted {CONSOLE_NAME} installer on zsh PATH");
        self.note_or_add(&path, &added, &noted, created)
    }

    pub fn path_add_zprofile(&mut self) -> bool {
        let default = Path::new(&self.home()).join(".zprofile").to_string_lossy().into_owned();
        let (explicit, path) = self.target("ZPROFILE", &default);
        let mut created = false;
        if !Path::new(&path).is_file() {
            if self.host.normal_user_only() && !explicit {
                return false;
            }
            if !explicit && !self.shell_is_zsh() {
                return false;
            }
            let header = format!("# Zsh login rc created by {CONSOLE_NAME} installer ({})\n", self.version);
            if !self.create(&path, &header) {
                return self.create_failure(&path);
            }
            created = true;
            let message = format!("→ Created {path}");
            self.plain(&message);
        }
        let added = format!("→ Added {} to PATH for zsh (.zprofile)", self.user_bin());
        let noted = format!("→ Noted {CONSOLE_NAME} installer on zsh login PATH");
        self.note_or_add(&path, &added, &noted, created)
    }

    pub fn path_add_zshrc(&self) -> bool {
        false
    }

    pub fn path_add_fish(&mut self) -> bool {
        let default = Path::new(&self.home())
            .join(".config")
            .join("fish")
            .join("config.fish")
            .to_string_lossy()
            .into_owned();
        let (_explicit, path) = self.target("FISH_CONFIG", &default);
        if let Some(directory) = Path::new(&path).parent() {
            if !directory.as_os_str().is_empty() && fs::create_dir_all(directory).is_err() {
                return false;
            }
        }
        let fish_line = self.fish_line();
        let comment = self.comment();
        if Path::new(&path).is_file() && self.contains(&path, &fish_line) {
            if !self.contains(&path, &comment) {
                return self.append(&path, &[&comment]);
            }
            return false;
        }
        if !self.append(&path, &[&comment, &fish_line]) {
            return false;
        }
        let message = format!("→ Added {} to PATH for fish shell", self.user_bin());
        self.plain(&message);
        true
    }

    pub fn path_invoking_is_ash(&self) -> bool {
        let raw = self
            .environ
            .get("RUSTAGENT_ARGV0")
            .cloned()
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "rust-agent".to_string());
        let mut base = Path::new(&raw)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_string();
        if let Some(stripped) = base.strip_prefix('-') {
            base = stripped.to_string();
        }
        if base == "ash" {
            return true;
        }
        if base != "sh" {
            return false;
        }
        let release = if self.environ.contains_key("RUSTAGENT_ALPINE_RELEASE") {
            self.environ.get("RUSTAGENT_ALPINE_RELEASE").cloned().unwrap_or_default()
        } else {
            "/etc/alpine-release".to_string()
        };
        !release.is_empty() && Path::new(&release).is_file()
    }

    pub fn path_add_profile_ash(&mut self) -> bool {
        if !self.path_invoking_is_ash() {
            return false;
        }
        let default = Path::new(&self.home()).join(".profile").to_string_lossy().into_owned();
        let (_explicit, path) = self.target("PROFILE", &default);
        if !Path::new(&path).is_file() {
            return false;
        }
        let added = format!("→ Added {} to PATH for ash (.profile)", self.user_bin());
        let noted = format!("→ Noted {CONSOLE_NAME} installer on ash PATH");
        self.note_or_add(&path, &added, &noted, false)
    }

    pub fn path_ensure_profile(&mut self) -> bool {
        let default = Path::new(&self.home()).join(".profile").to_string_lossy().into_owned();
        let (_explicit, path) = self.target("PROFILE", &default);
        if Path::new(&path).is_file() {
            return self.path_add_profile_ash();
        }
        let header = format!(
            "# BEGIN {CONSOLE_NAME} profile source-bashrc\n\
# Created so a bash login shell sources interactive rc.\n\
if [ -n \"${{BASH_VERSION:-}}\" ]; then\n\
    if [ -f \"${{HOME}}/.bashrc\" ]; then\n\
        . \"${{HOME}}/.bashrc\"\n\
    fi\n\
fi\n\
# END {CONSOLE_NAME} profile source-bashrc\n"
        );
        if !self.create(&path, &header) {
            self.warn(&format!(
                "Could not create {path}. Next: create that file as this login so SSH login reads ~/.bashrc, then re-run {CONSOLE_NAME} install."
            ));
            return false;
        }
        if self.path_invoking_is_ash() {
            let message = format!("→ Created {path}");
            self.plain(&message);
            return self.path_add_profile_ash();
        }
        let message = format!("→ Created {path} (login shell sources ~/.bashrc)");
        self.plain(&message);
        true
    }

    fn remember(shell_config: &str, path: &str) -> String {
        if shell_config == "N/A" {
            path.to_string()
        } else {
            format!("{shell_config} {path}")
        }
    }

    pub fn path_add_shell(&mut self) -> i32 {
        if self.host.is_windows_cmd() || self.is_root() {
            return 0;
        }
        let mut added = false;
        let mut shell_config = "N/A".to_string();
        let home = self.home();
        if self.path_add_bashrc() {
            added = true;
            let default = Path::new(&home).join(".bashrc").to_string_lossy().into_owned();
            let (_explicit, path) = self.target("BASHRC", &default);
            shell_config = path;
        }
        if self.path_add_zshenv() {
            added = true;
            let default = Path::new(&home).join(".zshenv").to_string_lossy().into_owned();
            let (_explicit, path) = self.target("ZSHENV", &default);
            shell_config = Self::remember(&shell_config, &path);
        }
        if self.path_add_zprofile() {
            added = true;
            let default = Path::new(&home).join(".zprofile").to_string_lossy().into_owned();
            let (_explicit, path) = self.target("ZPROFILE", &default);
            shell_config = Self::remember(&shell_config, &path);
        }
        if self.path_add_fish() {
            added = true;
            let default = Path::new(&home)
                .join(".config")
                .join("fish")
                .join("config.fish")
                .to_string_lossy()
                .into_owned();
            let (_explicit, path) = self.target("FISH_CONFIG", &default);
            shell_config = Self::remember(&shell_config, &path);
        }
        if self.path_ensure_profile() {
            added = true;
            let default = Path::new(&home).join(".profile").to_string_lossy().into_owned();
            let (_explicit, path) = self.target("PROFILE", &default);
            shell_config = Self::remember(&shell_config, &path);
        }
        if added {
            let message = format!("→ Restart your terminal or run: source {shell_config}");
            self.plain(&message);
        }
        0
    }

    fn die(&mut self, message: &str, next: &str) -> i32 {
        self.output.error(message, next)
    }

    pub fn path_rc_test(&mut self, root: &str, file_name: &str, case_name: &str, feature: &str) -> i32 {
        let sample = format!("{CONSOLE_NAME} rc-test --root \"$tmpdir\" --file zprofile --case create");
        if root.is_empty() {
            return self.die("rc-test requires --root a tmp/cache folder.", &sample);
        }
        let allowed = ["bashrc", "zshenv", "zprofile", "profile", "fish", "ash"];
        if file_name == "zshrc" {
            return self.die(
                "rc-test zshrc is not claimed (zsh PATH is .zshenv and .zprofile).",
                &format!("{CONSOLE_NAME} rc-test --root \"$tmpdir\" --file zshenv --case create"),
            );
        }
        if !allowed.contains(&file_name) {
            return self.die(
                "rc-test --file must be bashrc, zshenv, zprofile, profile, ash, or fish.",
                &sample,
            );
        }
        if !matches!(case_name, "create" | "modify" | "noop") {
            return self.die("rc-test --case must be create, modify, or noop.", &sample);
        }
        if fs::create_dir_all(root).is_err() {
            return self.die(&format!("rc-test cannot create --root {root}."), &sample);
        }
        let home = self.home();
        let watched = [
            Path::new(&home).join(".bashrc"),
            Path::new(&home).join(".profile"),
            Path::new(&home).join(".zshenv"),
            Path::new(&home).join(".zprofile"),
        ];
        let before: Vec<Option<Vec<u8>>> = watched.iter().map(|path| fs::read(path).ok()).collect();
        let mut overlay = self.environ.clone();
        overlay.insert("BASHRC".into(), Path::new(root).join(".bashrc").to_string_lossy().into_owned());
        overlay.insert("ZSHENV".into(), Path::new(root).join(".zshenv").to_string_lossy().into_owned());
        overlay.insert("ZPROFILE".into(), Path::new(root).join(".zprofile").to_string_lossy().into_owned());
        overlay.insert("ZSHRC".into(), Path::new(root).join(".zshrc").to_string_lossy().into_owned());
        overlay.insert("PROFILE".into(), Path::new(root).join(".profile").to_string_lossy().into_owned());
        overlay.insert("FISH_CONFIG".into(), Path::new(root).join("config.fish").to_string_lossy().into_owned());
        let target = match file_name {
            "bashrc" => overlay["BASHRC"].clone(),
            "zshenv" => overlay["ZSHENV"].clone(),
            "zprofile" => overlay["ZPROFILE"].clone(),
            "fish" => overlay["FISH_CONFIG"].clone(),
            _ => overlay["PROFILE"].clone(),
        };
        if !under_root(root, &target) {
            return self.die("rc-test writes the fixture under --root only.", &sample);
        }
        let original = self.environ.clone();
        let original_host = self.host.environ.clone();
        self.environ = overlay.clone();
        self.host.environ = overlay;
        self.prepare_case(file_name, case_name, &target);
        self.run_case(file_name);
        self.environ = original;
        self.host.environ = original_host;
        let after: Vec<Option<Vec<u8>>> = watched.iter().map(|path| fs::read(path).ok()).collect();
        if after != before {
            return self.die("rc-test wrote this login's real home rc. Fixture --root only.", &sample);
        }
        let message = format!("rc-test {file_name} {case_name}");
        if self.output.json_mode {
            return self.output.ok(
                "rc-test",
                &message,
                Some(serde_json::json!({
                    "root": root,
                    "file": file_name,
                    "case": case_name,
                    "status": "ok",
                    "feature": feature,
                })),
            );
        }
        self.plain(&format!("[OK] {message} under {root}"));
        0
    }

    fn prepare_case(&mut self, file_name: &str, case_name: &str, target: &str) {
        let path_line = self.path_line();
        let comment = self.comment();
        if file_name == "fish" {
            if let Some(directory) = Path::new(target).parent() {
                let _ = fs::create_dir_all(directory);
            }
        }
        if case_name == "create" {
            if Path::new(target).is_file() {
                let _ = fs::remove_file(target);
            }
            return;
        }
        match file_name {
            "bashrc" => {
                if case_name == "modify" {
                    let _ = self.create(target, "# dongle-bashrc-keep\nalias dongle_probe=true\n");
                } else {
                    let body = format!("# dongle-already-good\n{comment}\n{path_line}\n");
                    let _ = self.create(target, &body);
                }
            }
            "zshenv" => {
                if case_name == "modify" {
                    let _ = self.create(target, "# dongle-zshenv-keep\n");
                } else {
                    let body = format!("# dongle-already-good\n{comment}\n{path_line}\n");
                    let _ = self.create(target, &body);
                }
            }
            "zprofile" => {
                if case_name == "modify" {
                    let _ = self.create(target, "# dongle-zprofile-keep\n");
                } else {
                    let body = format!("# dongle-already-good\n{comment}\n{path_line}\n");
                    let _ = self.create(target, &body);
                }
            }
            "profile" => {
                let _ = self.create(target, "# keep-existing-profile\n");
            }
            "ash" => {
                if case_name == "modify" {
                    let _ = self.create(target, "# keep-existing-profile\n");
                } else {
                    let body = format!("# keep-existing-profile\n{comment}\n{path_line}\n");
                    let _ = self.create(target, &body);
                }
            }
            "fish" => {
                if case_name == "modify" {
                    let _ = self.create(target, "# dongle-fish-keep\n");
                } else {
                    let fish = self.fish_line();
                    let body = format!("{comment}\n{fish}\n");
                    let _ = self.create(target, &body);
                }
            }
            _ => {}
        }
    }

    fn run_case(&mut self, file_name: &str) {
        match file_name {
            "bashrc" => {
                self.path_add_bashrc();
            }
            "zshenv" => {
                self.path_add_zshenv();
            }
            "zprofile" => {
                self.path_add_zprofile();
            }
            "profile" => {
                self.path_ensure_profile();
            }
            "ash" => {
                self.environ.insert("RUSTAGENT_ARGV0".into(), "ash".into());
                self.path_ensure_profile();
                self.path_add_profile_ash();
            }
            "fish" => {
                self.path_add_fish();
            }
            _ => {}
        }
    }

    pub fn cleanup_path(&mut self) -> i32 {
        if self.is_root() {
            return 0;
        }
        let home = self.home();
        let user_bin = self.user_bin();
        let names = [
            ("BASHRC", Path::new(&home).join(".bashrc").to_string_lossy().into_owned()),
            ("ZSHENV", Path::new(&home).join(".zshenv").to_string_lossy().into_owned()),
            ("ZPROFILE", Path::new(&home).join(".zprofile").to_string_lossy().into_owned()),
            ("ZSHRC", Path::new(&home).join(".zshrc").to_string_lossy().into_owned()),
            (
                "FISH_CONFIG",
                Path::new(&home)
                    .join(".config")
                    .join("fish")
                    .join("config.fish")
                    .to_string_lossy()
                    .into_owned(),
            ),
            ("PROFILE", Path::new(&home).join(".profile").to_string_lossy().into_owned()),
        ];
        let drop_path = !user_bin_has_files(&user_bin);
        if !drop_path {
            self.warn(&format!("Keeping {user_bin} in PATH because it still contains files."));
        }
        let prefix = format!("# Added by {CONSOLE_NAME} installer");
        let path_line = self.path_line();
        let fish_line = self.fish_line();
        for (name, default) in names {
            let (_explicit, path) = self.target(name, &default);
            if !Path::new(&path).is_file() {
                continue;
            }
            let body = match fs::read_to_string(&path) {
                Ok(body) => body,
                Err(_) => continue,
            };
            let kept: Vec<String> = body
                .lines()
                .filter(|line| {
                    if line.starts_with(&prefix) {
                        return false;
                    }
                    if drop_path && (*line == path_line || *line == fish_line) {
                        return false;
                    }
                    true
                })
                .map(|line| line.to_string())
                .collect();
            let _ = rewrite(&path, &kept);
        }
        0
    }
}

fn user_bin_has_files(user_bin: &str) -> bool {
    fs::read_dir(user_bin).map(|mut dir| dir.next().is_some()).unwrap_or(false)
}

fn rewrite(path: &str, lines: &[String]) -> bool {
    let body = if lines.is_empty() {
        String::new()
    } else {
        format!("{}\n", lines.join("\n"))
    };
    fs::write(path, body).is_ok()
}

fn under_root(root: &str, path: &str) -> bool {
    let root_real = fs::canonicalize(root).unwrap_or_else(|_| Path::new(root).to_path_buf());
    let parent = Path::new(path).parent().unwrap_or(Path::new(root));
    if !parent.is_dir() && fs::create_dir_all(parent).is_err() {
        return false;
    }
    let parent_real = fs::canonicalize(parent).unwrap_or_else(|_| parent.to_path_buf());
    parent_real == root_real || parent_real.starts_with(&root_real)
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
