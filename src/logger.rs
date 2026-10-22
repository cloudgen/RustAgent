//! One status logger, constructed in main before the parser.
//! requirement-rust-cli-logging
//!
//! The daily file and the quiet flag follow the same operator contract the
//! previous logger used: kebab name, base directory, log directory, and a
//! line that stays off the text screen when the run is quiet.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub const CLASS_NAME: &str = "StatusLog";
pub const CLASS_VERSION: &str = "StatusLog v1.0.0";

#[derive(Clone, Debug)]
pub struct StatusLog {
    pub log_name: String,
    pub base_dir: String,
    pub log_dir: String,
    quiet: bool,
    debug: bool,
    created_note: bool,
}

impl StatusLog {
    /// Construct one logger. `is_quiet` is known before the directory is created.
    pub fn new(logname: &str, is_quiet: bool, basedir: &str, logdir: &str) -> Self {
        let log_name = kebab(logname);
        let base_dir = if basedir.is_empty() {
            resolve_base(&log_name)
        } else {
            basedir.to_string()
        };
        let log_dir = if logdir.is_empty() {
            format!("{base_dir}/log")
        } else {
            logdir.to_string()
        };
        let debug = debug_from_env();
        let mut logger = Self {
            log_name,
            base_dir,
            log_dir,
            quiet: is_quiet,
            debug,
            created_note: false,
        };
        logger.ensure_log_dir();
        logger
    }

    pub fn class_version() -> &'static str {
        CLASS_VERSION
    }

    pub fn log_name(&self) -> &str {
        &self.log_name
    }

    pub fn base_dir(&self) -> &str {
        &self.base_dir
    }

    pub fn log_dir(&self) -> &str {
        &self.log_dir
    }

    pub fn is_debug(&self) -> bool {
        self.debug
    }

    pub fn is_quiet(&self) -> bool {
        self.quiet
    }

    pub fn in_venv() -> bool {
        nonempty_env("VIRTUAL_ENV")
    }

    pub fn in_pyenv() -> bool {
        nonempty_env("PYENV_VERSION") || nonempty_env("PYENV_ROOT")
    }

    pub fn in_conda() -> bool {
        nonempty_env("CONDA_PREFIX") || nonempty_env("CONDA_EXE")
    }

    pub fn log_message(&mut self, message: &str, level: &str, component: &str) {
        let level = if level.is_empty() {
            "INFO".to_string()
        } else {
            level.to_uppercase()
        };
        let stamp = stamp_seconds();
        let pid = std::process::id();
        let component_str = if component.is_empty() {
            String::new()
        } else {
            format!(" @{component}")
        };
        let line = format!("[{stamp}] pid:{pid} [{level}]{component_str} :] {message}");
        let path = self.daily_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
            let _ = writeln!(file, "{line}");
        }
        if self.quiet {
            return;
        }
        if level == "ERROR" || level == "CRITICAL" || level == "FATAL" {
            eprintln!("{line}");
        } else {
            println!("{line}");
        }
    }

    pub fn daily_path(&self) -> PathBuf {
        let day = day_stamp();
        PathBuf::from(&self.log_dir).join(format!("{}-{day}.log", self.log_name))
    }

    fn ensure_log_dir(&mut self) {
        let path = PathBuf::from(&self.log_dir);
        if path.is_dir() {
            return;
        }
        if fs::create_dir_all(&path).is_ok() && !self.created_note {
            self.created_note = true;
            if !self.quiet {
                println!("Created directory: {}", path.display());
            }
        }
    }
}

pub fn kebab(name: &str) -> String {
    let mut out = String::new();
    for (index, ch) in name.chars().enumerate() {
        if ch.is_uppercase() && index > 0 {
            out.push('-');
        }
        out.push(ch.to_ascii_lowercase());
    }
    out
}

fn nonempty_env(key: &str) -> bool {
    std::env::var(key).map(|value| !value.is_empty()).unwrap_or(false)
}

fn debug_from_env() -> bool {
    let raw = std::env::var("DEBUG")
        .or_else(|_| std::env::var("debug"))
        .unwrap_or_default();
    let raw = raw.to_lowercase();
    raw == "1" || raw == "true" || raw == "show"
}

fn resolve_base(log_name: &str) -> String {
    if let Some(root) = conda_base() {
        return format!("{root}/.app/{log_name}");
    }
    if let Some(root) = pyenv_venv() {
        return format!("{root}/.app/{log_name}");
    }
    if let Some(root) = venv_path() {
        return format!("{root}/.app/{log_name}");
    }
    if is_root() {
        return format!("/var/{log_name}");
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    format!("{home}/.app/{log_name}")
}

fn conda_base() -> Option<String> {
    let prefix = std::env::var("CONDA_PREFIX").ok()?;
    if prefix.is_empty() || !PathBuf::from(&prefix).is_dir() {
        return None;
    }
    Some(prefix)
}

fn pyenv_venv() -> Option<String> {
    let env = std::env::var("VIRTUAL_ENV").ok()?;
    if env.contains(".pyenv") && PathBuf::from(&env).is_dir() {
        return Some(env);
    }
    None
}

fn venv_path() -> Option<String> {
    let env = std::env::var("VIRTUAL_ENV").ok()?;
    if env.is_empty() || !PathBuf::from(&env).is_dir() {
        return None;
    }
    Some(env)
}

fn is_root() -> bool {
    #[cfg(unix)]
    {
        unsafe { effective_uid() == 0 }
    }
    #[cfg(not(unix))]
    {
        false
    }
}

#[cfg(unix)]
unsafe fn effective_uid() -> u32 {
    extern "C" {
        fn geteuid() -> u32;
    }
    geteuid()
}

fn stamp_seconds() -> String {
    format_local("%Y-%m-%d %H:%M:%S")
}

fn day_stamp() -> String {
    format_local("%Y%m%d")
}

fn format_local(fmt: &str) -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    local_format(secs, fmt)
}

fn local_format(secs: i64, fmt: &str) -> String {
    // Avoid the `time` crate. Use libc-free civil time from the `date` command only as fallback.
    // The process clock is formatted with a small civil conversion in UTC offset from the local `TZ`.
    let offset = local_offset_secs();
    let local = secs + offset;
    let days = local.div_euclid(86_400);
    let tod = local.rem_euclid(86_400) as u32;
    let hour = tod / 3600;
    let minute = (tod % 3600) / 60;
    let second = tod % 60;
    let (year, month, day) = civil_from_days(days);
    match fmt {
        "%Y%m%d" => format!("{year:04}{month:02}{day:02}"),
        _ => format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}"),
    }
}

fn local_offset_secs() -> i64 {
    // Compare UTC and local via the GNU date binary when it exists. Otherwise stay at UTC.
    let utc = std::process::Command::new("date")
        .args(["-u", "+%Y-%m-%d %H:%M:%S"])
        .output();
    let local = std::process::Command::new("date")
        .args(["+%Y-%m-%d %H:%M:%S"])
        .output();
    match (utc, local) {
        (Ok(u), Ok(l)) => {
            let us = String::from_utf8_lossy(&u.stdout).trim().to_string();
            let ls = String::from_utf8_lossy(&l.stdout).trim().to_string();
            match (parse_stamp(&us), parse_stamp(&ls)) {
                (Some(a), Some(b)) => b - a,
                _ => 0,
            }
        }
        _ => 0,
    }
}

fn parse_stamp(text: &str) -> Option<i64> {
    let mut parts = text.split(['-', ' ', ':']);
    let year: i32 = parts.next()?.parse().ok()?;
    let month: u32 = parts.next()?.parse().ok()?;
    let day: u32 = parts.next()?.parse().ok()?;
    let hour: u32 = parts.next()?.parse().ok()?;
    let minute: u32 = parts.next()?.parse().ok()?;
    let second: u32 = parts.next()?.parse().ok()?;
    Some(epoch_from_civil(year, month, day, hour, minute, second))
}

fn epoch_from_civil(year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32) -> i64 {
    let days = days_from_civil(year, month, day);
    days * 86_400 + hour as i64 * 3600 + minute as i64 * 60 + second as i64
}

fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = (y - era * 400) as u32;
    let mp = if month > 2 { month - 3 } else { month + 9 };
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era as i64 * 146_097 + doe as i64 - 719_468
}

fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = (z - era * 146_097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i32 + era as i32 * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { y + 1 } else { y };
    (year, month, day)
}
