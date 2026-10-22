//! Product identity. Version matches Cargo.toml.
//! requirement-rust-packaging

pub const APP_NAME: &str = "RustAgent";
pub const VERSION: &str = "1.2.0";
pub const CONSOLE_NAME: &str = "rust-agent";
pub const CRATE_NAME: &str = "rustagent";
pub const AUTHOR_NAME: &str = "cloudgen";
pub const AUTHOR_EMAIL: &str = "wongcf22@gmail.com";
pub const HOMEPAGE: &str = "https://github.com/cloudgen/RustAgent";
pub const LAST_UPDATE: &str = "2026-10-22";
pub const DOWNLOAD_URL: &str = "";
pub const BASIC_USAGE: &str = "rust-agent";
pub const LANG_ENV: &str = "RUSTAGENT_LANG";
pub const APP_DIR: &str = "RustAgent";
pub const DEPOSIT: &str = "/var/rust-agent";
pub const GLOBAL_BIN: &str = "/usr/local/bin";

pub const PRODUCT_VERBS: &[&str] = &[
    "help",
    "version",
    "about",
    "hello",
    "menu",
    "main",
    "ensure",
    "install",
    "uninstall",
    "rc-test",
    "where-is-me",
    "self-install",
    "version-check",
    "self-update",
    "self-uninstall",
    "setup",
    "update-grok",
    "reinstall",
    "run",
    "check-session",
    "backup",
    "sync-auth",
    "sync-auth-from-remote",
    "sync-auth-to-remote",
    "add-crontab",
    "print-sudoers",
    "print-sudoers-install-script",
    "remove-project-sudoers",
    "generate-sudoer-request",
    "submit-sudoer-request",
];
