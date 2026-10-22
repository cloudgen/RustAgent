//! Contracts carried over from the text menu: hello, language, path, columns.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use rustagent::cli::{self, MemConsole};
use rustagent::identity::{APP_NAME, CONSOLE_NAME, PRODUCT_VERBS, VERSION};
use rustagent::language::MenuLanguage;
use rustagent::model::{Action, Key, MenuModel};
use rustagent::painter::{MenuPainter, PATH_LABEL_EN, PATH_LABEL_ZH_HANT};
use rustagent::self_manage::SelfManage;
use rustagent::session::MenuSession;
use rustagent::width::display_width;

fn temp_dir(prefix: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("{prefix}{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn hello_prints_welcome_and_skips_the_menu() {
    assert!(PRODUCT_VERBS.contains(&"hello"));
    assert!(!PRODUCT_VERBS.contains(&"join"));
    assert!(!PRODUCT_VERBS.contains(&"list-videos"));
    assert!(!PRODUCT_VERBS.contains(&"language"));
    assert!(!cli::opens_text_screen(&["hello".into()], true));
    assert!(cli::opens_text_screen(&[], true));
    assert!(cli::opens_text_screen(&["about".into()], true));
    assert!(!cli::opens_text_screen(&["join".into()], true));
    assert!(!cli::opens_text_screen(&[], false));

    let mut console = MemConsole { tty: true, ..MemConsole::default() };
    let code = cli::run(&["hello".into()], &mut console, Some("/tmp"), Some("/tmp"));
    assert_eq!(code, 0);
    assert_eq!(console.out, format!("Welcome to {APP_NAME} {VERSION}.\n"));
}

#[test]
fn removed_video_verbs_are_unknown() {
    for verb in ["join", "list-videos"] {
        let mut console = MemConsole::default();
        let code = cli::run(&[verb.into()], &mut console, Some("/tmp"), Some("/tmp"));
        assert_eq!(code, 1, "{verb}");
        assert!(console.err.contains(&format!("Unknown verb '{verb}'")));
    }
    let mut console = MemConsole::default();
    assert_eq!(cli::run(&["help".into()], &mut console, Some("/tmp"), Some("/tmp")), 0);
    assert!(!console.out.contains("list-videos"));
    assert!(!console.out.to_lowercase().contains("ffmpeg"));
    assert!(!console.out.contains("concatenate"));
    assert!(console.out.contains(CONSOLE_NAME));
}

#[test]
fn menu_row_1_is_grok_auth_and_hello_stays_a_verb() {
    let home = temp_dir("rustagent-hello-");
    std::env::remove_var("RUSTAGENT_LANG");
    let painter = std::rc::Rc::new(std::cell::RefCell::new(MenuPainter::new(Some(
        home.to_string_lossy().into_owned(),
    ))));
    assert_eq!(painter.borrow().language.code(), "en");
    let rows = painter.borrow().display_rows("front");
    assert_eq!(rows[0].number, 1);
    assert_eq!(rows[0].short, "grok-auth");
    assert_eq!(rows[0].explain, "backup, sync, crontab, setup, and reinstall");
    assert_eq!(rows[0].kind, "grok-auth");
    assert!(PRODUCT_VERBS.contains(&"hello"));
    let mut session = MenuSession {
        app_name: APP_NAME.to_string(),
        version: VERSION.to_string(),
        model: MenuModel::new(std::rc::Rc::clone(&painter)),
        on_version: Box::new(|| String::new()),
        on_about: Box::new(|| String::new()),
        on_kind: Some(Box::new(|kind, _screen, _model| {
            if kind == "hello" {
                Some(format!("Welcome to {APP_NAME} {VERSION}."))
            } else {
                None
            }
        })),
    };
    let mut screen = NullScreen;
    assert_eq!(session.take_action(Action::Kind("hello".into()), &mut screen), "stay");
    assert!(session.model.phase_is_result());
    assert_eq!(session.model.result_text, format!("Welcome to {APP_NAME} {VERSION}."));
}

#[test]
fn path_label_ignores_lang() {
    assert_eq!(PATH_LABEL_EN, "Path");
    assert_eq!(PATH_LABEL_ZH_HANT, "路徑");
    let home = temp_dir("rustagent-path-");
    let folder = temp_dir("rustagent-cwd-");
    let previous = std::env::current_dir().unwrap();
    std::env::set_current_dir(&folder).unwrap();
    std::env::set_var("LANG", "zh_TW.UTF-8");
    std::env::set_var("LC_ALL", "zh_TW.UTF-8");
    std::env::remove_var("RUSTAGENT_LANG");
    let mut painter = MenuPainter::new(Some(home.to_string_lossy().into_owned()));
    painter.set_clock(Some("14:05:09".into()));
    assert_eq!(painter.path_label(), "Path");
    let current = folder.canonicalize().unwrap().to_string_lossy().into_owned();
    let logical = format!("Path: {current}  14:05:09");
    assert_eq!(painter.path_line(None), logical);
    assert!(!logical.contains("路徑"));
    let narrow = "Path: ".len() + 4;
    let shortened = painter.path_line(Some(narrow));
    assert_eq!(shortened, format!("Path: {}", &current[current.len() - 4..]));
    assert!(!shortened.contains("14:05:09"));
    std::env::set_current_dir(previous).unwrap();
}

#[test]
fn wide_columns_count_east_asian_width() {
    assert_eq!(MenuPainter::display_width("╭─╮"), 3);
    assert_eq!(MenuPainter::display_width("█"), 1);
    assert_eq!(MenuPainter::display_width("路径"), MenuPainter::display_width("Path"));
    assert_eq!(MenuPainter::display_width("路徑"), 4);
    assert_eq!(MenuPainter::display_width("パス"), 4);
    assert_eq!(MenuPainter::display_width("경로"), 4);
    assert_eq!(display_width("A"), 1);
}

#[test]
fn language_leaf_round_trip() {
    let home = temp_dir("rustagent-lang-");
    std::env::set_var("LANG", "zh_TW.UTF-8");
    std::env::remove_var("RUSTAGENT_LANG");
    let painter = std::rc::Rc::new(std::cell::RefCell::new(MenuPainter::new(Some(
        home.to_string_lossy().into_owned(),
    ))));
    let mut model = MenuModel::new(std::rc::Rc::clone(&painter));
    let front = painter.borrow().display_rows("front");
    let numbers: Vec<i32> = front.iter().map(|row| row.number).collect();
    assert_eq!(numbers, vec![1, 3, 5, 6, 7, 9]);
    assert_eq!(front[1].kind, "system-log");
    assert_eq!(front[2].kind, "sudoers");
    assert_eq!(front[3].kind, "self-management");
    assert_eq!(front[4].short, "language");
    let log_rows = painter.borrow().display_rows("log");
    assert_eq!(
        log_rows.iter().map(|row| row.number).collect::<Vec<_>>(),
        vec![31, 32, 33, 0]
    );
    assert_eq!(log_rows[0].short, "view-log");
    let self_rows = painter.borrow().display_rows("self");
    assert_eq!(
        self_rows.iter().map(|row| row.number).collect::<Vec<_>>(),
        vec![62, 63, 64, 65, 66, 67, 68, 0]
    );
    assert_eq!(self_rows[6].kind, "where-is-me");
    let lang_rows = painter.borrow().display_rows("lang");
    let printed: Vec<i32> = lang_rows.iter().map(|row| row.number).collect();
    assert_eq!(&printed[..13], &(80..=92).collect::<Vec<_>>());
    assert_eq!(*printed.last().unwrap(), 0);
    assert!(!printed.contains(&79));
    assert!(!printed.contains(&93));
    assert_eq!(lang_rows[0].short, "English");
    assert_eq!(lang_rows[2].short, "繁體中文");
    assert_eq!(lang_rows[lang_rows.len() - 2].short, "Ελληνικά");

    model.buffer = "7".into();
    model.cursor = 1;
    model.focus = rustagent::model::Focus::Input;
    assert_eq!(model.apply_key(Key::Enter, 24), Action::Stay);
    assert_eq!(model.layer, "lang");
    model.buffer = "82".into();
    model.cursor = 2;
    let action = model.apply_key(Key::Enter, 24);
    assert_eq!(action, Action::Kind("set-zh-Hant".into()));
    let mut session = MenuSession {
        app_name: APP_NAME.into(),
        version: VERSION.into(),
        model,
        on_version: Box::new(|| String::new()),
        on_about: Box::new(|| String::new()),
        on_kind: None,
    };
    let mut screen = NullScreen;
    assert_eq!(session.take_action(action, &mut screen), "stay");
    assert_eq!(session.model.layer, "front");
    assert_eq!(session.model.notice, "選單語言是繁體中文");
    assert_eq!(painter.borrow().path_label(), "路徑");
    let leaf = painter.borrow().language.leaf_path();
    assert_eq!(fs::read_to_string(&leaf).unwrap(), "zh-Hant\n");
    let mode = fs::metadata(&leaf).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
    let folder_mode = fs::metadata(PathBuf::from(&leaf).parent().unwrap())
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(folder_mode, 0o700);
    let front = painter.borrow().display_rows("front");
    assert_eq!(front[4].short, "語言");
    assert_eq!(front[0].short, "grok-auth");
    assert_eq!(front[3].explain, "版本、關於，以及 cargo 生命週期");
}

#[test]
fn invalid_leaf_stays_english_and_env_wins() {
    let home = temp_dir("rustagent-leaf-");
    let leaf = home.join(".local").join("RustAgent").join("language");
    fs::create_dir_all(leaf.parent().unwrap()).unwrap();
    fs::write(&leaf, "nope\nzh-Hant\n").unwrap();
    let loaded = MenuLanguage::new(Some(home.to_string_lossy().into_owned()));
    assert_eq!(loaded.code(), "en");
    assert_eq!(fs::read_to_string(&leaf).unwrap(), "nope\nzh-Hant\n");
    std::env::set_var("RUSTAGENT_LANG", "zh-Hant");
    let forced = MenuLanguage::new(Some(home.to_string_lossy().into_owned()));
    assert_eq!(forced.code(), "zh-Hant");
    assert_eq!(forced.path_label(), PATH_LABEL_ZH_HANT);
    assert_eq!(fs::read_to_string(&leaf).unwrap(), "nope\nzh-Hant\n");
    std::env::remove_var("RUSTAGENT_LANG");
    fs::write(&leaf, "es\r\n").unwrap();
    let mut again = MenuLanguage::new(Some(home.to_string_lossy().into_owned()));
    again.load();
    assert_eq!(again.code(), "es");
    assert_eq!(again.path_label(), "Ruta");
}

#[test]
fn failed_write_keeps_the_code() {
    let mut empty = MenuLanguage::new(Some(String::new()));
    assert_eq!(empty.save("es"), "Could not save the menu language");
    assert_eq!(empty.code(), "en");
}

#[test]
fn cargo_lifecycle_argv_has_no_sudo() {
    let manage = SelfManage::new(APP_NAME, VERSION);
    let check = manage.argv_for("version-check").unwrap();
    assert_eq!(check, vec!["cargo", "search", "rustagent", "--limit", "1"]);
    assert!(!check.iter().any(|part| part == "sudo" || part == "pip"));
    let update = manage.argv_for("self-update").unwrap();
    assert_eq!(update[0], "cargo");
    assert!(update.iter().any(|part| part == "--force"));
    let uninstall = manage.argv_for("self-uninstall").unwrap();
    assert_eq!(uninstall, vec!["cargo", "uninstall", "rustagent"]);
    let mut console = MemConsole::default();
    let code = cli::run(&["self-uninstall".into()], &mut console, Some("/tmp"), Some("/tmp"));
    assert_eq!(code, 1);
    assert!(console.err.contains("--force"));
}

#[test]
fn about_page_identity_stays_english() {
    let mut console = MemConsole { tty: false, ..MemConsole::default() };
    let code = cli::run(&["about".into()], &mut console, Some("/tmp"), Some("/tmp"));
    assert_eq!(code, 0);
    let text = &console.out;
    assert!(text.starts_with(&format!("{APP_NAME} {VERSION}\n")));
    assert!(text.contains("Domain: Install and run xAI grok, and back up its auth files."));
    assert!(text.contains("Runtime tools: curl, grok"));
    assert!(text.contains(&format!("Entry points: {CONSOLE_NAME}")));
    assert!(text.contains("[CHECK SYSTEM]:"));
    assert!(text.contains("Now checking your operation system!"));
    assert!(text.contains("You are using"));
    assert!(!text.contains("curl -fsSL"));
    assert!(!text.contains("py-tui"));
    assert!(!text.to_lowercase().contains("ffmpeg"));
    assert!(text.contains("Basic Usage:"));
    assert!(text.contains("rust-agent"));
}

#[test]
fn version_matches_cargo_manifest() {
    let manifest = fs::read_to_string("Cargo.toml").unwrap();
    assert!(manifest.contains("version = \"1.2.0\""));
    assert_eq!(VERSION, "1.2.0");
    let mut console = MemConsole::default();
    assert_eq!(cli::run(&["version".into()], &mut console, Some("/tmp"), Some("/tmp")), 0);
    assert_eq!(console.out, format!("{APP_NAME} {VERSION}\n"));
}

#[test]
fn logger_resolves_kebab_name() {
    let dir = temp_dir("rustagent-log-");
    let base = dir.join("base");
    let logs = dir.join("logs");
    let logger = rustagent::logger::StatusLog::new(
        APP_NAME,
        true,
        base.to_str().unwrap(),
        logs.to_str().unwrap(),
    );
    assert_eq!(logger.log_name(), "rust-agent");
    assert!(logger.is_quiet());
    assert!(logger.daily_path().to_string_lossy().contains("rust-agent-"));
}

struct NullScreen;

impl rustagent::painter::Screen for NullScreen {
    fn size(&self) -> (u16, u16) { (24, 80) }
    fn erase(&mut self) {}
    fn put(&mut self, _y: u16, _x: u16, _text: &str, _style: rustagent::painter::Style) {}
    fn refresh(&mut self) {}
    fn move_to(&mut self, _y: u16, _x: u16) {}
    fn read_key(&mut self) -> Key { Key::Esc }
    fn poll_key(&mut self, _timeout_ms: u64) -> Option<Key> { None }
}
