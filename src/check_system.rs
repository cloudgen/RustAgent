//! Host check for the about page.
//! requirement-rust-about

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::identity::APP_NAME;
use crate::logger::StatusLog;

pub struct CheckSystem {
    pub app_name: String,
    pub version: String,
    pub console_name: String,
    pub tty: bool,
    location_override: Option<String>,
    stamp_override: Option<String>,
}

impl CheckSystem {
    pub fn new(app_name: &str, version: &str, console_name: &str, tty: bool) -> Self {
        let _ = StatusLog::in_venv();
        let _ = StatusLog::in_pyenv();
        let _ = StatusLog::in_conda();
        Self {
            app_name: app_name.to_string(),
            version: version.to_string(),
            console_name: console_name.to_string(),
            tty,
            location_override: None,
            stamp_override: None,
        }
    }

    pub fn with_location(mut self, path: &str) -> Self {
        self.location_override = Some(path.to_string());
        self
    }

    pub fn with_stamp(mut self, stamp: &str) -> Self {
        self.stamp_override = Some(stamp.to_string());
        self
    }

    pub fn self_location(&self) -> String {
        if let Some(path) = &self.location_override {
            return path.clone();
        }
        std::env::current_exe()
            .ok()
            .and_then(|path| path.canonicalize().ok())
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_default()
    }

    pub fn stamp(&self) -> String {
        if let Some(stamp) = &self.stamp_override {
            return stamp.clone();
        }
        local_stamp_micros()
    }

    pub fn check_system_lines(&self) -> Vec<String> {
        let (rustc, cargo, host) = toolchain();
        let arch = arch_label();
        let libc = libc_token(&host);
        let binary = binary_type(&arch, &libc);
        let user = current_user();
        let pid = std::process::id().to_string();
        let mut store = crate::storage::Storage::new(
            std::env::vars().collect(),
            Some(user.clone()),
            pid.parse::<u32>().ok(),
        );
        let preferred = store.preferred_dir();
        let first = store.fallback_dir();
        let second = store.fallback2_dir();
        let used = store.resolve().unwrap_or_else(|_| "(unavailable)".to_string());
        let persistence = store.persistent_dir();
        let location = self.self_location();
        let fields = [
            ("Rust", rust_line(&rustc)),
            ("C Library", c_library(&host)),
            ("Operation System", operation_system()),
            ("Architecture", arch.clone()),
            ("Current User", user),
            ("Shell", std::env::var("SHELL").unwrap_or_default()),
            ("Rust Executable", executable_base(&rustc)),
            ("rustc location", rustc),
            ("cargo location", cargo),
            (
                "Inside docker container",
                if Path::new("/.dockerenv").exists() {
                    "True".to_string()
                } else {
                    "False".to_string()
                },
            ),
            ("Binary Type", binary),
            ("Location", location),
            ("PID", pid),
            ("Cache folder used", used),
            ("Cache folder (preferred)", preferred),
            ("Cache folder (1st fallback)", first),
            ("Cache folder (2nd fallback)", second),
            ("Persistence storage", persistence),
            (
                "TTY / Interactive",
                if self.tty { "yes".to_string() } else { "no".to_string() },
            ),
        ];
        let mut lines = vec![
            format!(
                "{} {}(v{})  [CHECK SYSTEM]:",
                self.stamp(),
                self.app_name,
                self.version
            ),
            "  Now checking your operation system!".to_string(),
        ];
        for (label, value) in fields {
            lines.push(format!("    {label}: {value}"));
        }
        lines
    }
}

fn rust_line(rustc: &str) -> String {
    if rustc.is_empty() {
        return String::new();
    }
    command_first_line("rustc", &["--version"])
}

fn c_library(host: &str) -> String {
    if host.is_empty() {
        return String::new();
    }
    host.to_string()
}

fn executable_base(path: &str) -> String {
    if path.is_empty() {
        return String::new();
    }
    Path::new(path)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn toolchain() -> (String, String, String) {
    let rustc = which("rustc");
    let cargo = which("cargo");
    let host = if rustc.is_empty() {
        String::new()
    } else {
        command_host()
    };
    (rustc, cargo, host)
}

fn command_host() -> String {
    let text = command_output("rustc", &["-vV"]);
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("host: ") {
            return rest.trim().to_string();
        }
    }
    String::new()
}

fn command_first_line(program: &str, args: &[&str]) -> String {
    command_output(program, args)
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .to_string()
}

fn command_output(program: &str, args: &[&str]) -> String {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .map(|out| String::from_utf8_lossy(&out.stdout).into_owned())
        .unwrap_or_default()
}

fn which(name: &str) -> String {
    let path = std::env::var("PATH").unwrap_or_default();
    for dir in path.split(':') {
        if dir.is_empty() {
            continue;
        }
        let candidate = PathBuf::from(dir).join(name);
        if candidate.is_file() {
            return candidate.to_string_lossy().into_owned();
        }
    }
    String::new()
}

pub fn arch_label() -> String {
    match std::env::consts::ARCH {
        "x86_64" | "amd64" => "amd64".to_string(),
        "aarch64" | "arm64" => "arm64".to_string(),
        "x86" | "i386" | "i686" => "x86".to_string(),
        other => other.to_string(),
    }
}

pub fn libc_token(host: &str) -> String {
    let lower = host.to_lowercase();
    if lower.contains("msvc") || lower.contains("windows") {
        return "msc".to_string();
    }
    if lower.contains("musl") {
        return "musl".to_string();
    }
    if lower.contains("gnu") {
        return "glibc".to_string();
    }
    if host.is_empty() && std::env::var("SHELL").ok().as_deref() == Some("/bin/ash") {
        return "muslc".to_string();
    }
    if host.is_empty() {
        return "unknown".to_string();
    }
    host.rsplit('-').next().unwrap_or("unknown").to_string()
}

pub fn binary_type(arch: &str, libc: &str) -> String {
    if arch.is_empty() {
        return String::new();
    }
    if libc.is_empty() {
        return format!("{arch}-");
    }
    format!("{arch}-{libc}")
}

pub fn current_user() -> String {
    if let Ok(user) = std::env::var("USER") {
        if !user.is_empty() {
            return user;
        }
    }
    if let Ok(user) = std::env::var("USERNAME") {
        if !user.is_empty() {
            return user;
        }
    }
    String::new()
}

fn operation_system() -> String {
    if let Ok(text) = fs::read_to_string("/etc/os-release") {
        for line in text.lines() {
            if let Some(rest) = line.strip_prefix("PRETTY_NAME=") {
                let name = rest.trim_matches('"').trim();
                if !name.is_empty() {
                    return name.to_string();
                }
            }
        }
    }
    format!("{} {}", std::env::consts::OS, std::env::consts::ARCH)
}

fn home_dir() -> String {
    match std::env::var("HOME") {
        Ok(home) if !home.is_empty() && Path::new(&home).is_absolute() => home,
        _ => String::new(),
    }
}

pub fn install_kind(path: &str, console_name: &str, app_name: &str) -> String {
    let path = Path::new(path);
    let text = path.to_string_lossy().to_string();
    let marker = format!("{}src{}", std::path::MAIN_SEPARATOR, std::path::MAIN_SEPARATOR);
    let packaged = text.contains("site-packages") || text.contains("dist-packages");
    if !packaged && text.contains(&marker) && source_checkout(path) {
        return "uninstalled".to_string();
    }
    let home = home_dir();
    let in_home = !home.is_empty()
        && (text == home || text.starts_with(&(home.clone() + std::path::MAIN_SEPARATOR_STR)));
    let base = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    if in_home && (packaged || base == console_name || base == app_name || base == "rustagent") {
        return "local".to_string();
    }
    if text.starts_with("/usr/") || text.starts_with("/opt/") || text.starts_with("/bin/") {
        return "global".to_string();
    }
    if packaged && !in_home {
        return "global".to_string();
    }
    if text.contains("/.cargo/bin/") && in_home {
        return "local".to_string();
    }
    "uninstalled".to_string()
}

fn source_checkout(path: &Path) -> bool {
    let mut cursor = path.to_path_buf();
    for _ in 0..8 {
        if cursor.join("Cargo.toml").is_file() {
            return true;
        }
        if !cursor.pop() {
            return false;
        }
    }
    false
}

fn local_stamp_micros() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs() as i64;
    let micros = now.subsec_micros();
    let (year, month, day, hour, minute, second) = civil_local(secs);
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}.{micros:06}")
}

fn civil_local(secs: i64) -> (i32, u32, u32, u32, u32, u32) {
    let offset = local_offset(secs);
    let local = secs + offset;
    let days = local.div_euclid(86_400);
    let tod = local.rem_euclid(86_400) as u32;
    let (year, month, day) = civil_from_days(days);
    (year, month, day, tod / 3600, (tod % 3600) / 60, tod % 60)
}

fn local_offset(secs: i64) -> i64 {
    // GNU date is not required. The offset is the difference between localtime and UTC
    // reported by the C library when the symbol is present. A missing symbol stays at UTC.
    unsafe { local_offset_libc(secs) }
}

unsafe fn local_offset_libc(secs: i64) -> i64 {
    #[repr(C)]
    struct Tm {
        tm_sec: i32,
        tm_min: i32,
        tm_hour: i32,
        tm_mday: i32,
        tm_mon: i32,
        tm_year: i32,
        tm_wday: i32,
        tm_yday: i32,
        tm_isdst: i32,
        tm_gmtoff: i64,
        tm_zone: *const i8,
    }
    extern "C" {
        fn localtime_r(timep: *const i64, result: *mut Tm) -> *mut Tm;
        fn gmtime_r(timep: *const i64, result: *mut Tm) -> *mut Tm;
    }
    let mut local = std::mem::zeroed::<Tm>();
    let mut utc = std::mem::zeroed::<Tm>();
    if localtime_r(&secs, &mut local).is_null() || gmtime_r(&secs, &mut utc).is_null() {
        return 0;
    }
    local.tm_gmtoff
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

pub fn app_name() -> &'static str {
    APP_NAME
}
