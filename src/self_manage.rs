//! Cargo lifecycle for the self-management verbs.
//! requirement-rust-cli-interface, requirement-rust-tui

use std::process::{Command, Stdio};

use crate::identity::{APP_NAME, CRATE_NAME};

pub struct SelfManage {
    pub app_name: String,
    pub version: String,
    runner: Box<dyn FnMut(&[String]) -> (i32, String, String)>,
}

impl SelfManage {
    pub fn new(app_name: &str, version: &str) -> Self {
        Self {
            app_name: app_name.to_string(),
            version: version.to_string(),
            runner: Box::new(default_runner),
        }
    }

    pub fn with_runner<F>(app_name: &str, version: &str, runner: F) -> Self
    where
        F: FnMut(&[String]) -> (i32, String, String) + 'static,
    {
        Self {
            app_name: app_name.to_string(),
            version: version.to_string(),
            runner: Box::new(runner),
        }
    }

    pub fn local_version(&self) -> String {
        format!("{} {}", self.app_name, self.version)
    }

    pub fn argv_for(&self, verb: &str) -> Result<Vec<String>, String> {
        let dist = CRATE_NAME.to_string();
        let argv = match verb {
            "version-check" => vec!["cargo".into(), "search".into(), dist, "--limit".into(), "1".into()],
            "self-update" => vec!["cargo".into(), "install".into(), dist, "--force".into()],
            "self-install" => vec!["cargo".into(), "install".into(), dist],
            "self-uninstall" => vec!["cargo".into(), "uninstall".into(), dist],
            _ => return Err(verb.to_string()),
        };
        Ok(argv)
    }

    pub fn run_cargo(&mut self, verb: &str) -> (i32, String) {
        let argv = match self.argv_for(verb) {
            Ok(argv) => argv,
            Err(_) => return (1, format!("Unknown lifecycle verb {verb}")),
        };
        let (code, out, err) = (self.runner)(&argv);
        let mut chunks = vec![format!("$ {}", argv.join(" "))];
        if verb == "version-check" {
            chunks.insert(0, self.local_version());
        }
        if !out.trim().is_empty() {
            chunks.push(out.trim().to_string());
        }
        if !err.trim().is_empty() {
            chunks.push(err.trim().to_string());
        }
        if code != 0 && out.trim().is_empty() && err.trim().is_empty() {
            chunks.push(format!("cargo exited {code}"));
        }
        (code, chunks.join("\n"))
    }
}

fn default_runner(argv: &[String]) -> (i32, String, String) {
    if argv.is_empty() {
        return (1, String::new(), "empty command".to_string());
    }
    match Command::new(&argv[0])
        .args(&argv[1..])
        .stdin(Stdio::null())
        .output()
    {
        Ok(done) => (
            done.status.code().unwrap_or(1),
            String::from_utf8_lossy(&done.stdout).into_owned(),
            String::from_utf8_lossy(&done.stderr).into_owned(),
        ),
        Err(err) => (1, String::new(), err.to_string()),
    }
}

pub fn welcome(version: &str) -> String {
    format!("Welcome to {APP_NAME} {version}.")
}
