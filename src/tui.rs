//! Text menu session for RustAgent.
//! requirement-rust-tui — menu region and the bottom input box.

use std::cell::RefCell;
use std::io::{self, Write};
use std::rc::Rc;
use std::time::Duration;

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::style::{Attribute, Print, SetAttribute};
use crossterm::terminal::{
    self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;

use crate::about::AboutPage;
use crate::domain::{self, Domain};
use crate::identity::{APP_NAME, PRODUCT_VERBS, VERSION};
use crate::model::{Action, Focus, Key, MenuModel};
use crate::painter::{Style, MenuPainter, Screen};
use crate::self_manage::{self, SelfManage};
use crate::session::MenuSession;
use crate::system_log::SystemLog;

pub struct Tui {
    pub app_name: String,
    pub version: String,
    pub painter: Rc<RefCell<MenuPainter>>,
    pub about: AboutPage,
    pub manage: SelfManage,
    pub logs: SystemLog,
    pub domain: Domain,
}

impl Tui {
    pub fn new(about: AboutPage, manage: SelfManage, logs: SystemLog, painter: Rc<RefCell<MenuPainter>>) -> Self {
        Self::with_domain(about, manage, logs, painter, Domain::from_process())
    }

    pub fn with_domain(
        about: AboutPage,
        manage: SelfManage,
        logs: SystemLog,
        painter: Rc<RefCell<MenuPainter>>,
        domain: Domain,
    ) -> Self {
        Self {
            app_name: APP_NAME.to_string(),
            version: VERSION.to_string(),
            painter,
            about,
            manage,
            logs,
            domain,
        }
    }

    pub fn hello_text(&self) -> String {
        self_manage::welcome(&self.version)
    }

    pub fn help_text(&self) -> String {
        format!(
            "help: usage for {name}.\n\
Verbs: {verbs}.\n\
version shows the installed version and does not call cargo.\n\
about shows the about page.\n\
hello prints a welcome line on the terminal and does not draw this menu.\n\
Menu row 1 is grok-auth. language is menu row 7 and is not a verb.\n\
Row 3 is system-log. This-login-only hosts show row 2 run and hide row 5 sudoers.\n\
This program does not join media or list files in this folder.\n\
version-check runs: cargo search {crate_name} --limit 1\n\
self-update runs: cargo install {crate_name} --force\n\
self-install runs: cargo install {crate_name}\n\
self-uninstall runs: cargo uninstall {crate_name}\n\
On the command line, self-uninstall also needs --force.\n\
Empty arguments on a terminal open this menu and do not run cargo.",
            name = self.app_name,
            verbs = PRODUCT_VERBS.join(", "),
            crate_name = crate::identity::CRATE_NAME,
        )
    }

    pub fn open_text_menu(&mut self) -> Result<(), String> {
        let mut screen = TermScreen::enter().map_err(|_| "open".to_string())?;
        let result = self.run_board(&mut screen);
        screen.leave();
        result
    }

    pub fn open_about(&mut self) -> Result<(), String> {
        let mut screen = TermScreen::enter().map_err(|_| "open".to_string())?;
        let text = self.about.framework_about();
        self.notice(&mut screen, &text, "about");
        let result = self.run_board(&mut screen);
        screen.leave();
        result
    }

    fn run_board(&mut self, screen: &mut dyn Screen) -> Result<(), String> {
        let app_name = self.app_name.clone();
        let version = self.version.clone();
        let painter = Rc::clone(&self.painter);
        let mut session = MenuSession {
            app_name: app_name.clone(),
            version: version.clone(),
            model: MenuModel::new(Rc::clone(&painter)),
            on_version: Box::new({
                let version = version.clone();
                let name = app_name.clone();
                move || format!("{name} {version}")
            }),
            on_about: Box::new({
                let page = self.about.framework_about();
                move || page.clone()
            }),
            on_kind: None,
        };
        // The kind handler needs Tui fields. Run the loop here so view-log can borrow self.
        self.bind_board();
        let (height, width) = screen.size();
        if !self.painter.borrow().screen_can_hold_here(height, width) {
            return Err("too-small".to_string());
        }
        loop {
            let rows = session.model.rows();
            painter.borrow().paint(
                screen,
                &mut session.model,
                &rows,
                &session.app_name,
                &session.version,
            );
            let (height, _) = screen.size();
            let show_clock = !session.model.phase_is_result()
                && matches!(session.model.layer.as_str(), "front" | "self" | "log" | "lang" | "auth" | "sudo");
            let key = if show_clock {
                screen.poll_key(1000)
            } else {
                Some(screen.read_key())
            };
            let Some(key) = key else {
                continue;
            };
            if key == Key::Interrupt {
                return Err("interrupt".to_string());
            }
            let action = session.model.apply_key(key, height);
            if self.dispatch(&mut session, action, screen)? == "exit" {
                return Ok(());
            }
        }
    }

    fn dispatch(
        &mut self,
        session: &mut MenuSession,
        action: Action,
        screen: &mut dyn Screen,
    ) -> Result<&'static str, String> {
        match action {
            Action::Kind(ref kind)
                if !kind.starts_with("set-") && !matches!(kind.as_str(), "version" | "about") =>
            {
                let outcome = self.kind_outcome(kind, screen, &mut session.model);
                if let Some(text) = outcome {
                    if !text.is_empty() {
                        session.model.show_result(text);
                    }
                }
                Ok("stay")
            }
            other => Ok(session.take_action(other, screen)),
        }
    }

    fn kind_outcome(
        &mut self,
        kind: &str,
        screen: &mut dyn Screen,
        model: &mut MenuModel,
    ) -> Option<String> {
        match kind {
            "help" => Some(self.help_text()),
            "hello" => Some(self.hello_text()),
            "version-check" | "self-update" | "self-install" | "self-uninstall" => {
                Some(self.manage.run_cargo(kind).1)
            }
            "view-log" => self.view_log(screen, model),
            "clear-log" => self.clear_log(screen, model),
            "log-folder" => Some(self.logs.folder_text()),
            "sync-auth-from-remote" | "sync-auth-to-remote" => {
                let spec = self.read_line(
                    screen,
                    model,
                    &["Remote (user@host, IPv4, domain, or user@domain)".to_string()],
                    kind,
                )?;
                let spec = spec.trim().to_string();
                if spec.is_empty() {
                    return Some("Cancelled.".to_string());
                }
                Some(self.domain.captured(kind, &[spec], false))
            }
            "setup" => {
                let peer = self.domain.peer_path();
                if self.domain.peer_ready() {
                    let choice = self.read_line(
                        screen,
                        model,
                        &[
                            format!("grok is already installed at {peer}."),
                            "1. keep".to_string(),
                            "2. reinstall".to_string(),
                            "9. Exit".to_string(),
                        ],
                        "setup",
                    )?;
                    let token = choice.trim().to_lowercase();
                    if token == "2" || token == "reinstall" {
                        return Some(self.domain.captured("reinstall", &[], true));
                    }
                    if token == "9" || token == "exit" {
                        return None;
                    }
                    return Some(format!("Left the existing grok at {peer}."));
                }
                Some(self.domain.captured("setup", &[], false))
            }
            other if domain::is_menu_kind(other) => Some(self.domain.captured(other, &[], false)),
            _ => None,
        }
    }

    fn bind_board(&mut self) {
        let logged_in = self.domain.logged_in();
        let this_login_only = self.domain.host.normal_user_only();
        let lines = domain::unavailable_lines(&self.domain.host, logged_in);
        let mut painter = self.painter.borrow_mut();
        painter.logged_in = logged_in;
        painter.this_login_only = this_login_only;
        painter.unavailable = lines;
    }

    fn view_log(&mut self, screen: &mut dyn Screen, model: &mut MenuModel) -> Option<String> {
        let folder = self.logs.log_dir();
        if folder.is_empty() {
            return Some("No log folder.".to_string());
        }
        let files = self.logs.log_files();
        if files.is_empty() {
            return Some(format!("No log file in {folder}."));
        }
        let mut lines = vec!["Log files:".to_string()];
        for (index, path) in files.iter().enumerate() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            lines.push(format!("{}. {name}", index + 1));
        }
        lines.push(String::new());
        lines.push("Choose a log file:".to_string());
        let picked = self.read_index(screen, model, &lines, files.len(), "system-log")?;
        let path = &files[picked];
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
        let Some(body) = self.logs.read_log(path) else {
            return Some(format!("ERROR: {name} is not a log file in the log folder."));
        };
        let body = if body.is_empty() { "(empty)".to_string() } else { body };
        Some(format!("{name}\n\n{body}"))
    }

    fn clear_log(&mut self, screen: &mut dyn Screen, model: &mut MenuModel) -> Option<String> {
        let folder = self.logs.log_dir();
        if folder.is_empty() {
            return Some("No log folder.".to_string());
        }
        let files = self.logs.log_files();
        if files.is_empty() {
            return Some(format!("No log file in {folder}."));
        }
        let mut lines = vec!["Log files:".to_string()];
        for (index, path) in files.iter().enumerate() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            lines.push(format!("{}. {name}", index + 1));
        }
        lines.push(String::new());
        lines.push("Choose a log file to clear:".to_string());
        let picked = self.read_index(screen, model, &lines, files.len(), "system-log")?;
        let path = files[picked].clone();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
        let answer = self.read_yes_no(
            screen,
            model,
            &[format!("Clear {name}? (y/n)")],
            "system-log",
        )?;
        if !answer {
            return None;
        }
        if !self.logs.clear_log(&path) {
            return Some(format!("ERROR: {name} is not a log file in the log folder."));
        }
        Some(format!("Cleared {name}."))
    }

    fn read_index(
        &mut self,
        screen: &mut dyn Screen,
        model: &mut MenuModel,
        lines: &[String],
        count: usize,
        title: &str,
    ) -> Option<usize> {
        loop {
            let raw = self.read_line(screen, model, lines, title)?;
            let raw = raw.trim();
            match raw.parse::<usize>() {
                Ok(idx) if (1..=count).contains(&idx) => return Some(idx - 1),
                Ok(_) => model.error = format!("Enter 1–{count}"),
                Err(_) => model.error = "Please type a number".to_string(),
            }
        }
    }

    fn read_yes_no(
        &mut self,
        screen: &mut dyn Screen,
        model: &mut MenuModel,
        lines: &[String],
        title: &str,
    ) -> Option<bool> {
        loop {
            let raw = self.read_line(screen, model, lines, title)?;
            let raw = raw.trim().to_lowercase();
            if raw.is_empty() || raw == "n" || raw == "no" {
                return Some(false);
            }
            if raw == "y" || raw == "yes" {
                return Some(true);
            }
            model.error = "Please enter y or n".to_string();
        }
    }

    fn read_line(
        &mut self,
        screen: &mut dyn Screen,
        model: &mut MenuModel,
        lines: &[String],
        title: &str,
    ) -> Option<String> {
        model.buffer.clear();
        model.cursor = 0;
        model.focus = Focus::Input;
        let painter = Rc::clone(&self.painter);
        loop {
            painter.borrow().paint_prompt(
                screen,
                model,
                title,
                lines,
                true,
                &self.app_name,
                &self.version,
            );
            let key = screen.read_key();
            if matches!(key, Key::Eof | Key::Esc | Key::Interrupt) {
                model.error.clear();
                model.buffer.clear();
                model.cursor = 0;
                model.focus = Focus::List;
                return None;
            }
            if key == Key::Enter {
                let text = model.buffer.clone();
                model.buffer.clear();
                model.cursor = 0;
                model.error.clear();
                return Some(text);
            }
            if key == Key::Backspace {
                model.backspace();
                model.error.clear();
                continue;
            }
            if matches!(key, Key::Left | Key::Right | Key::Home | Key::End) {
                // slide only works while focus is input. apply a local slide.
                let len = model.buffer.chars().count();
                match key {
                    Key::Left => model.cursor = model.cursor.saturating_sub(1),
                    Key::Right => model.cursor = (model.cursor + 1).min(len),
                    Key::Home => model.cursor = 0,
                    Key::End => model.cursor = len,
                    _ => {}
                }
                continue;
            }
            if let Key::Char(ch) = key {
                if ch >= ' ' {
                    model.insert(ch);
                    model.error.clear();
                }
            }
        }
    }

    fn notice(&self, screen: &mut dyn Screen, text: &str, title: &str) {
        let painter = Rc::clone(&self.painter);
        let mut model = MenuModel::new(Rc::clone(&painter));
        let mut lines: Vec<String> = text.lines().map(|line| line.to_string()).collect();
        if lines.is_empty() {
            lines.push(String::new());
        }
        lines.push(String::new());
        lines.push("Press a key to return to the main menu.".to_string());
        painter.borrow().paint_prompt(
            screen,
            &model,
            title,
            &lines,
            true,
            &self.app_name,
            &self.version,
        );
        let _ = screen.read_key();
        model.show_front();
    }
}

struct TermScreen {
    stdout: io::Stdout,
    active: bool,
}

impl TermScreen {
    fn enter() -> io::Result<Self> {
        let mut stdout = io::stdout();
        enable_raw_mode()?;
        stdout.execute(EnterAlternateScreen)?;
        stdout.execute(Hide)?;
        let _ = stdout.execute(crossterm::terminal::DisableLineWrap);
        Ok(Self { stdout, active: true })
    }

    fn leave(&mut self) {
        if !self.active {
            return;
        }
        self.active = false;
        let _ = self.stdout.execute(crossterm::terminal::EnableLineWrap);
        let _ = self.stdout.execute(Show);
        let _ = self.stdout.execute(LeaveAlternateScreen);
        let _ = disable_raw_mode();
        let _ = self.stdout.flush();
    }
}

impl Drop for TermScreen {
    fn drop(&mut self) {
        self.leave();
    }
}

impl Screen for TermScreen {
    fn size(&self) -> (u16, u16) {
        terminal::size()
            .map(|(w, h)| (h, w))
            .unwrap_or((24, 80))
    }

    fn erase(&mut self) {
        let _ = self.stdout.execute(terminal::Clear(terminal::ClearType::All));
        let _ = self.stdout.execute(MoveTo(0, 0));
    }

    fn put(&mut self, y: u16, x: u16, text: &str, style: Style) {
        let _ = self.stdout.execute(MoveTo(x, y));
        apply_style(&mut self.stdout, style);
        let _ = self.stdout.execute(Print(text));
        let _ = self.stdout.execute(SetAttribute(Attribute::Reset));
    }

    fn refresh(&mut self) {
        let _ = self.stdout.flush();
    }

    fn move_to(&mut self, y: u16, x: u16) {
        let _ = self.stdout.execute(MoveTo(x, y));
    }

    fn read_key(&mut self) -> Key {
        loop {
            match event::read() {
                Ok(event) => {
                    if let Some(key) = map_event(event) {
                        return key;
                    }
                }
                Err(_) => return Key::Eof,
            }
        }
    }

    fn poll_key(&mut self, timeout_ms: u64) -> Option<Key> {
        match event::poll(Duration::from_millis(timeout_ms)) {
            Ok(true) => match event::read() {
                Ok(event) => map_event(event),
                Err(_) => Some(Key::Eof),
            },
            Ok(false) => None,
            Err(_) => Some(Key::Eof),
        }
    }
}

fn apply_style(stdout: &mut io::Stdout, style: Style) {
    let _ = stdout.execute(SetAttribute(Attribute::Reset));
    if style.bold {
        let _ = stdout.execute(SetAttribute(Attribute::Bold));
    }
    if style.italic {
        let _ = stdout.execute(SetAttribute(Attribute::Italic));
    }
    if style.reverse {
        let _ = stdout.execute(SetAttribute(Attribute::Reverse));
    }
    if style.underline {
        let _ = stdout.execute(SetAttribute(Attribute::Underlined));
    }
}

fn map_event(event: Event) -> Option<Key> {
    match event {
        Event::Resize(_, _) => None,
        Event::Key(key) => Some(map_key(key.code, key.modifiers)),
        _ => None,
    }
}

fn map_key(code: KeyCode, modifiers: KeyModifiers) -> Key {
    if modifiers.contains(KeyModifiers::CONTROL) && matches!(code, KeyCode::Char('c') | KeyCode::Char('C'))
    {
        return Key::Interrupt;
    }
    match code {
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::Enter => Key::Enter,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Esc => Key::Esc,
        KeyCode::Char(ch) => Key::Char(ch),
        _ => Key::Other,
    }
}
