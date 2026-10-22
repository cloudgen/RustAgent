//! Shell startup stickers. requirement-shell-startup-files, requirement-shell-zshenv.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use rustagent::identity::{CONSOLE_NAME, VERSION};
use rustagent::shell_startup::ShellStartup;

fn temp_dir(prefix: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "{prefix}{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    fs::create_dir_all(&path).unwrap();
    path
}

fn writer(home: &PathBuf, shell: &str) -> ShellStartup {
    let mut env = std::collections::HashMap::new();
    env.insert("HOME".into(), home.to_string_lossy().into_owned());
    env.insert("SHELL".into(), shell.to_string());
    env.insert("USER".into(), "ada".into());
    ShellStartup::new(VERSION, env, Some(1000))
}

#[test]
fn zsh_creates_zprofile_and_zshenv_once() {
    let home = temp_dir("zstart-");
    let mut startup = writer(&home, "/bin/zsh");
    assert_eq!(startup.path_add_shell(), 0);
    let zprofile = home.join(".zprofile");
    let zshenv = home.join(".zshenv");
    assert!(zprofile.is_file());
    assert!(zshenv.is_file());
    assert!(!home.join(".zshrc").exists());
    let profile = fs::read_to_string(&zprofile).unwrap();
    let env_body = fs::read_to_string(&zshenv).unwrap();
    let line = format!("export PATH=\"{}/.local/bin:$PATH\"", home.display());
    assert!(profile.contains(&line));
    assert!(env_body.contains(&line));
    assert!(profile.contains(&format!("# Zsh login rc created by {CONSOLE_NAME} installer ({VERSION})")));
    assert_eq!(fs::metadata(&zprofile).unwrap().permissions().mode() & 0o777, 0o644);
    assert_eq!(startup.path_add_shell(), 0);
    assert_eq!(fs::read_to_string(&zprofile).unwrap().matches(&line).count(), 1);
}

#[test]
fn bash_does_not_invent_zsh_notes() {
    let home = temp_dir("bstart-");
    let mut startup = writer(&home, "/bin/bash");
    assert_eq!(startup.path_add_shell(), 0);
    assert!(!home.join(".zprofile").exists());
    assert!(!home.join(".zshenv").exists());
    let body = fs::read_to_string(home.join(".profile")).unwrap();
    assert!(body.contains(". \"${HOME}/.bashrc\"") || body.contains(".bashrc"));
}

#[test]
fn rc_test_zprofile_writes_the_fixture_only() {
    let home = temp_dir("rchome-");
    let root = temp_dir("rcroot-");
    let mut env = std::collections::HashMap::new();
    env.insert("HOME".into(), home.to_string_lossy().into_owned());
    env.insert("SHELL".into(), "/bin/bash".into());
    let mut startup = ShellStartup::new(VERSION, env, Some(1000));
    let code = startup.path_rc_test(root.to_str().unwrap(), "zprofile", "create", "");
    assert_eq!(code, 0, "{}", startup.output.err);
    let fixture = root.join(".zprofile");
    assert!(fixture.is_file());
    let body = fs::read_to_string(&fixture).unwrap();
    assert!(body.contains("export PATH="));
    assert!(startup.output.out.contains("rc-test zprofile create"));
    assert!(!root.join(".zshrc").exists());
    assert!(!home.join(".zprofile").exists());
}

#[test]
fn rc_test_zshrc_fails_closed() {
    let root = temp_dir("rcbad-");
    let mut startup = ShellStartup::new(VERSION, std::collections::HashMap::new(), Some(1000));
    let code = startup.path_rc_test(root.to_str().unwrap(), "zshrc", "create", "");
    assert_eq!(code, 1);
    assert!(startup.output.err.contains("zshrc"));
}
