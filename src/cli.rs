//! RustAgent CLI — text menu and typed verbs.
//! requirement-rust-cli-interface, requirement-rust-cli-logging, requirement-rust-tui

use std::io::{self, IsTerminal, Write};

use crate::about::AboutPage;
use crate::check_system::CheckSystem;
use crate::domain::Domain;
use crate::identity::{self, APP_NAME, CONSOLE_NAME, PRODUCT_VERBS, VERSION};
use crate::logger::StatusLog;
use crate::painter::MenuPainter;
use crate::self_manage::SelfManage;
use crate::system_log::SystemLog;
use crate::tui::Tui;

const SCREEN_VERBS: &[&str] = &["about", "menu", "main"];
const FORCE_VERBS: &[&str] = &[
    "self-uninstall",
    "uninstall",
    "setup",
    "reinstall",
    "remove-project-sudoers",
];

pub struct Args {
    pub verb: Option<String>,
    pub force: bool,
}

pub trait Console {
    fn print_line(&mut self, text: &str);
    fn print_raw(&mut self, text: &str);
    fn eprint_line(&mut self, text: &str);
    fn is_tty(&self) -> bool;
}

pub struct StdConsole;

impl Console for StdConsole {
    fn print_line(&mut self, text: &str) {
        println!("{text}");
    }
    fn print_raw(&mut self, text: &str) {
        let mut out = io::stdout();
        let _ = write!(out, "{text}");
        if !text.ends_with('\n') {
            let _ = writeln!(out);
        }
    }
    fn eprint_line(&mut self, text: &str) {
        eprintln!("{text}");
    }
    fn is_tty(&self) -> bool {
        io::stdout().is_terminal()
    }
}

#[derive(Default)]
pub struct MemConsole {
    pub out: String,
    pub err: String,
    pub tty: bool,
}

impl Console for MemConsole {
    fn print_line(&mut self, text: &str) {
        self.out.push_str(text);
        if !text.ends_with('\n') {
            self.out.push('\n');
        }
    }
    fn print_raw(&mut self, text: &str) {
        self.print_line(text);
    }
    fn eprint_line(&mut self, text: &str) {
        self.err.push_str(text);
        if !text.ends_with('\n') {
            self.err.push('\n');
        }
    }
    fn is_tty(&self) -> bool {
        self.tty
    }
}

pub fn main() -> i32 {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut console = StdConsole;
    run(&argv, &mut console, None, None)
}

pub fn run(
    argv: &[String],
    console: &mut dyn Console,
    log_base: Option<&str>,
    log_dir: Option<&str>,
) -> i32 {
    match parse(argv) {
        Err(message) => report(console, &message, &format!("{CONSOLE_NAME} help")),
        Ok(args) if args.verb.as_deref() == Some("help") || args.help => {
            if args.json {
                console.print_line(&json_help());
            } else {
                console.print_raw(&help_text());
            }
            0
        }
        Ok(args) if args.version_flag => {
            console.print_line(&format!("{APP_NAME} {VERSION}"));
            0
        }
        Ok(args) => dispatch(argv, args, console, log_base, log_dir),
    }
}

pub struct Parsed {
    pub verb: Option<String>,
    pub force: bool,
    pub help: bool,
    pub version_flag: bool,
    pub json: bool,
    pub extra: Vec<String>,
    pub root: String,
    pub file: String,
    pub case_name: String,
    pub feature: String,
}

pub fn parse(argv: &[String]) -> Result<Parsed, String> {
    let mut verb = None;
    let mut force = false;
    let mut help = false;
    let mut version_flag = false;
    let mut json = false;
    let mut extra = Vec::new();
    let mut root = String::new();
    let mut file = String::new();
    let mut case_name = String::new();
    let mut feature = String::new();
    let mut index = 0;
    while index < argv.len() {
        let token = &argv[index];
        if matches!(token.as_str(), "--root" | "--file" | "--case" | "--feature") {
            if index + 1 >= argv.len() || argv[index + 1].starts_with('-') {
                return Err(format!("Unknown flag '{token}'."));
            }
            let value = argv[index + 1].clone();
            match token.as_str() {
                "--root" => root = value,
                "--file" => file = value,
                "--case" => case_name = value,
                _ => feature = value,
            }
            index += 2;
            continue;
        }
        if token == "--help" || token == "-h" {
            help = true;
        } else if token == "--version" {
            version_flag = true;
        } else if token == "--force" {
            force = true;
        } else if token == "--json" {
            json = true;
        } else if token == "--debug" || token == "--quiet" {
        } else if token.starts_with('-') {
            return Err(format!("Unknown flag '{token}'."));
        } else if verb.is_none() {
            verb = Some(token.clone());
        } else {
            extra.push(token.clone());
        }
        index += 1;
    }
    Ok(Parsed {
        verb,
        force,
        help,
        version_flag,
        json,
        extra,
        root,
        file,
        case_name,
        feature,
    })
}

pub fn opens_text_screen(argv: &[String], tty: bool) -> bool {
    if !tty {
        return false;
    }
    let Ok(parsed) = parse(argv) else {
        return false;
    };
    if parsed.help || parsed.version_flag || parsed.json {
        return false;
    }
    match parsed.verb.as_deref() {
        None => true,
        Some(verb) => SCREEN_VERBS.contains(&verb),
    }
}

fn dispatch(
    argv: &[String],
    args: Parsed,
    console: &mut dyn Console,
    log_base: Option<&str>,
    log_dir: Option<&str>,
) -> i32 {
    let screen = opens_text_screen(argv, console.is_tty());
    let mut logger = StatusLog::new(
        APP_NAME,
        screen,
        log_base.unwrap_or(""),
        log_dir.unwrap_or(""),
    );
    let _ = logger.log_name();
    let base = logger.base_dir().to_string();
    let _ = logger.log_dir();
    if logger.is_debug() {
        logger.log_message(
            &format!("{} v{} (src/cli.rs)", logger.log_name(), VERSION),
            "INFO",
            "main",
        );
        logger.log_message(
            &format!("Using {}", StatusLog::class_version()),
            "INFO",
            "main",
        );
        logger.log_message(&format!("Base {base}"), "DEBUG", "main");
        logger.log_message("debug mode", "INFO", "main");
    }
    let verb = args.verb.as_deref();
    if args.force && !verb.is_some_and(|name| FORCE_VERBS.contains(&name)) {
        return report(
            console,
            "--force is only for self-uninstall.",
            &format!("{CONSOLE_NAME} self-uninstall --force"),
        );
    }
    if verb != Some("rc-test") && (!args.root.is_empty() || !args.file.is_empty() || !args.case_name.is_empty() || !args.feature.is_empty()) {
        return report(
            console,
            "--root, --file, --case, and --feature belong to rc-test.",
            &format!("{CONSOLE_NAME} rc-test --root DIR --file zprofile --case create"),
        );
    }
    let mut domain = Domain::from_process();
    domain.output.json_mode = args.json;
    domain.output.capture = true;
    match verb {
        Some("hello") => {
            console.print_line(&crate::self_manage::welcome(VERSION));
            0
        }
        Some("version") => {
            console.print_line(&format!("{APP_NAME} {VERSION}"));
            0
        }
        Some("about") => {
            if args.json {
                console.print_line(&json_about());
                return 0;
            }
            let page = make_about(console.is_tty()).framework_about();
            if console.is_tty() {
                let mut tui = make_tui(console.is_tty(), logger);
                match tui.open_about() {
                    Ok(()) => 0,
                    Err(kind) => screen_error(console, &kind),
                }
            } else {
                console.print_raw(&page);
                0
            }
        }
        Some("self-uninstall") => {
            if !args.force {
                return report(
                    console,
                    "self-uninstall removes this package with cargo.",
                    &format!("{CONSOLE_NAME} self-uninstall --force"),
                );
            }
            let _ = domain.uninstall_startup(true);
            let mut manage = SelfManage::new(APP_NAME, VERSION);
            let (code, text) = manage.run_cargo("self-uninstall");
            console.print_raw(&text);
            code
        }
        Some(verb @ ("version-check" | "self-update" | "self-install")) => {
            let mut manage = SelfManage::new(APP_NAME, VERSION);
            let (code, text) = manage.run_cargo(verb);
            console.print_raw(&text);
            code
        }
        Some("rc-test") => emit_domain(
            console,
            domain.rc_test(&args.root, &args.file, &args.case_name, &args.feature),
            &domain,
        ),
        Some("menu") | Some("main") | None => {
            if args.json && verb.is_none() {
                console.print_line(&json_help());
                return 0;
            }
            if !console.is_tty() {
                if args.json {
                    console.print_line(&json_help());
                } else {
                    console.print_raw(&help_text());
                }
                return 0;
            }
            let mut tui = make_tui(true, logger);
            match tui.open_text_menu() {
                Ok(()) => 0,
                Err(kind) => screen_error(console, &kind),
            }
        }
        Some(token) if PRODUCT_VERBS.contains(&token) => {
            let code = domain.dispatch(token, &args.extra, args.force);
            emit_domain(console, code, &domain)
        }
        Some(token) => report(
            console,
            &format!("Unknown verb '{token}'."),
            &format!("{CONSOLE_NAME} help — verbs: {}", PRODUCT_VERBS.join(", ")),
        ),
    }
}

fn emit_domain(console: &mut dyn Console, code: i32, domain: &Domain) -> i32 {
    if !domain.output.out.is_empty() {
        console.print_raw(domain.output.out.trim_end_matches('\n'));
    }
    if !domain.output.err.is_empty() {
        console.eprint_line(domain.output.err.trim_end_matches('\n'));
    }
    code
}

fn json_help() -> String {
    serde_json::json!({
        "type": "help",
        "message": format!("{CONSOLE_NAME} help"),
        "verbs": PRODUCT_VERBS,
    })
    .to_string()
}

fn json_about() -> String {
    serde_json::json!({
        "type": "about",
        "message": "Install and run xAI grok, and back up its auth files.",
        "name": APP_NAME,
        "version": VERSION,
    })
    .to_string()
}

fn make_about(tty: bool) -> AboutPage {
    AboutPage::new(
        CheckSystem::new(APP_NAME, VERSION, CONSOLE_NAME, tty),
        APP_NAME,
        VERSION,
    )
}

fn make_tui(tty: bool, logger: StatusLog) -> Tui {
    let painter = std::rc::Rc::new(std::cell::RefCell::new(MenuPainter::new(None)));
    let about = make_about(tty);
    let manage = SelfManage::new(APP_NAME, VERSION);
    let logs = SystemLog::new(Some(logger));
    Tui::new(about, manage, logs, painter)
}

fn screen_error(console: &mut dyn Console, kind: &str) -> i32 {
    let message = if kind == "too-small" {
        "The text screen is too small for the menu and the input box."
    } else if kind == "interrupt" {
        return 130;
    } else {
        "The text menu could not open on this terminal."
    };
    report(console, message, &format!("{CONSOLE_NAME} help"))
}

pub fn report(console: &mut dyn Console, message: &str, next: &str) -> i32 {
    console.eprint_line(&format!("ERROR: {message}"));
    console.eprint_line(&format!("   Next: {next}"));
    1
}

pub fn help_text() -> String {
    format!(
        "{CONSOLE_NAME} — install and run xAI grok, and back up its auth files.\n\
With no arguments on a terminal, opens the text menu.\n\
With no arguments and no terminal, prints this help and stops.\n\
--json with no verb prints JSON help and does not open the menu.\n\
hello prints a welcome line and does not draw the menu.\n\
Menu row 1 is grok-auth. language is menu row 7 and is not a verb.\n\
This program does not join media or list files in this folder.\n\
version-check runs: cargo search {crate_name} --limit 1\n\
self-update runs: cargo install {crate_name} --force\n\
self-install runs: cargo install {crate_name}\n\
self-uninstall runs: cargo uninstall {crate_name}\n\
and needs --force. Empty arguments do not install or update.\n\
Operational verbs include install, setup, and self-install.\n\
Test-purpose, apart from those: rc-test --root DIR \
--file bashrc|zshenv|zprofile|profile|fish|ash \
--case create|modify|noop.\n\
Product verbs: {verbs}.\n",
        crate_name = identity::CRATE_NAME,
        verbs = PRODUCT_VERBS.join(", "),
    )
}
