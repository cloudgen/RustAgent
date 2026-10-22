//! Keystroke state for the text menu.
//! requirement-rust-tui

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::painter::{self, MenuPainter, Row};

pub type SharedPainter = Rc<RefCell<MenuPainter>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    Enter,
    Backspace,
    Esc,
    Eof,
    Interrupt,
    Char(char),
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    List,
    Input,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Board,
    Result,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Stay,
    Exit,
    Kind(String),
}

pub struct MenuModel {
    pub painter: SharedPainter,
    pub layer: String,
    pub index: usize,
    pub buffer: String,
    pub cursor: usize,
    pub focus: Focus,
    pub error: String,
    pub notice: String,
    pub phase: Phase,
    pub result_text: String,
    pub result_offset: usize,
    pub boards: Option<HashMap<String, Vec<Row>>>,
}

impl MenuModel {
    pub fn new(painter: SharedPainter) -> Self {
        Self {
            painter,
            layer: "front".to_string(),
            index: 0,
            buffer: String::new(),
            cursor: 0,
            focus: Focus::List,
            error: String::new(),
            notice: String::new(),
            phase: Phase::Board,
            result_text: String::new(),
            result_offset: 0,
            boards: None,
        }
    }

    pub fn phase_is_result(&self) -> bool {
        self.phase == Phase::Result
    }

    pub fn focus_is_list(&self) -> bool {
        self.focus == Focus::List
    }

    pub fn banner(&self) -> String {
        if !self.notice.is_empty() {
            return self.notice.clone();
        }
        self.error.clone()
    }

    pub fn rows(&self) -> Vec<Row> {
        if let Some(boards) = &self.boards {
            if let Some(found) = boards.get(&self.layer) {
                return found.clone();
            }
        }
        self.painter.borrow().display_rows(&self.layer)
    }

    pub fn show_front(&mut self) {
        self.phase = Phase::Board;
        self.layer = "front".to_string();
        self.index = 0;
        self.buffer.clear();
        self.cursor = 0;
        self.focus = Focus::List;
        self.error.clear();
        self.result_offset = 0;
    }

    pub fn show_result(&mut self, text: String) {
        self.result_text = text;
        self.result_offset = 0;
        self.phase = Phase::Result;
        self.layer = "front".to_string();
        self.index = 0;
        self.buffer.clear();
        self.cursor = 0;
        self.focus = Focus::List;
        self.error.clear();
    }

    pub fn apply_key(&mut self, key: Key, screen_height: u16) -> Action {
        self.notice.clear();
        match key {
            Key::Esc => {
                if self.phase == Phase::Result || self.layer != "front" {
                    self.show_front();
                }
                return Action::Stay;
            }
            Key::Eof => {
                if self.phase == Phase::Result || self.layer != "front" {
                    self.show_front();
                    return Action::Stay;
                }
                return Action::Exit;
            }
            _ => {}
        }
        if self.phase == Phase::Result {
            let lines: Vec<&str> = if self.result_text.is_empty() {
                vec![""]
            } else {
                self.result_text.lines().collect()
            };
            let count = if lines.is_empty() { 1 } else { lines.len() };
            let overflow = count > screen_height.saturating_sub(3) as usize;
            if overflow && matches!(key, Key::Up | Key::Char('k')) {
                self.scroll_result(-1, screen_height);
                return Action::Stay;
            }
            if overflow && matches!(key, Key::Down | Key::Char('j')) {
                self.scroll_result(1, screen_height);
                return Action::Stay;
            }
            self.show_front();
            return Action::Stay;
        }
        if matches!(key, Key::Up | Key::Down)
            || (self.focus == Focus::List && matches!(key, Key::Char('k') | Key::Char('j')))
        {
            self.arrow(key);
            return Action::Stay;
        }
        if matches!(key, Key::Left | Key::Right | Key::Home | Key::End) {
            self.slide(key);
            return Action::Stay;
        }
        if key == Key::Backspace {
            self.backspace();
            return Action::Stay;
        }
        if key == Key::Enter {
            return self.commit();
        }
        if let Key::Char(ch) = key {
            if ch >= ' ' && ch != '\u{7f}' {
                self.insert(ch);
                return Action::Stay;
            }
        }
        self.error = self.painter.borrow().language.text("invalid");
        Action::Stay
    }

    fn scroll_result(&mut self, delta: i32, screen_height: u16) {
        let count = if self.result_text.is_empty() {
            1
        } else {
            self.result_text.lines().count().max(1)
        };
        let room = MenuPainter::result_room(screen_height, count);
        let max_offset = count.saturating_sub(room);
        let next = self.result_offset as i32 + delta;
        self.result_offset = next.clamp(0, max_offset as i32) as usize;
    }

    fn arrow(&mut self, key: Key) {
        let last = self.rows().len().saturating_sub(1);
        let down = matches!(key, Key::Down | Key::Char('j'));
        if self.focus == Focus::Input {
            self.focus = Focus::List;
            self.index = if down { 0 } else { last };
            return;
        }
        if down {
            if self.index < last {
                self.index += 1;
            } else {
                self.focus = Focus::Input;
                self.cursor = self.buffer.chars().count();
            }
            return;
        }
        if self.index > 0 {
            self.index -= 1;
            return;
        }
        self.focus = Focus::Input;
        self.cursor = self.buffer.chars().count();
    }

    fn slide(&mut self, key: Key) {
        if self.focus != Focus::Input {
            return;
        }
        let len = self.buffer.chars().count();
        match key {
            Key::Left => self.cursor = self.cursor.saturating_sub(1),
            Key::Right => self.cursor = (self.cursor + 1).min(len),
            Key::Home => self.cursor = 0,
            Key::End => self.cursor = len,
            _ => {}
        }
    }

    pub fn insert(&mut self, ch: char) {
        if self.focus != Focus::Input {
            self.focus = Focus::Input;
            self.cursor = self.buffer.chars().count();
        }
        let len = self.buffer.chars().count();
        self.cursor = self.cursor.min(len);
        let mut out = String::new();
        let mut placed = false;
        for (index, item) in self.buffer.chars().enumerate() {
            if index == self.cursor {
                out.push(ch);
                placed = true;
            }
            out.push(item);
        }
        if !placed {
            out.push(ch);
        }
        self.buffer = out;
        self.cursor += 1;
    }

    pub fn backspace(&mut self) {
        if self.focus != Focus::Input {
            self.focus = Focus::Input;
            self.cursor = self.buffer.chars().count();
        }
        if self.cursor == 0 || self.buffer.is_empty() {
            self.cursor = 0;
            return;
        }
        let len = self.buffer.chars().count();
        self.cursor = self.cursor.min(len);
        let mut out = String::new();
        for (index, item) in self.buffer.chars().enumerate() {
            if index + 1 == self.cursor {
                continue;
            }
            out.push(item);
        }
        self.buffer = out;
        self.cursor -= 1;
    }

    fn reject(&mut self) {
        self.error = self.painter.borrow().language.text("invalid");
        self.focus = Focus::Input;
        self.cursor = self.buffer.chars().count();
    }

    fn commit(&mut self) -> Action {
        let token = self.buffer.trim().to_string();
        self.buffer.clear();
        self.cursor = 0;
        let rows = self.rows();
        if token.is_empty() {
            if let Some(row) = rows.get(self.index) {
                return self.activate(row.number, &row.kind);
            }
            self.reject();
            return Action::Stay;
        }
        if token.chars().all(|ch| ch.is_ascii_digit()) {
            if let Ok(number) = token.parse::<i32>() {
                return self.activate_number(number);
            }
        }
        for row in &rows {
            if token == row.short {
                return self.activate(row.number, &row.kind);
            }
        }
        let layer = self.layer.clone();
        let mapped = self.painter.borrow().language.token_kind(&token, &layer);
        if let Some(mapped) = mapped {
            return self.activate(0, &mapped);
        }
        if let Some(kind) = painter::any_board(&token) {
            return self.activate(0, kind);
        }
        self.reject();
        Action::Stay
    }

    fn activate_number(&mut self, number: i32) -> Action {
        let rows = self.rows();
        for row in &rows {
            if row.number == number {
                return self.activate(number, &row.kind);
            }
        }
        self.reject();
        Action::Stay
    }

    fn activate(&mut self, _number: i32, kind: &str) -> Action {
        self.error.clear();
        self.focus = Focus::List;
        self.cursor = 0;
        match kind {
            "exit" => Action::Exit,
            "back" => {
                self.layer = "front".to_string();
                self.index = 0;
                Action::Stay
            }
            "self-management" => {
                self.open_layer("self");
                Action::Stay
            }
            "system-log" => {
                self.open_layer("log");
                Action::Stay
            }
            "grok-auth" => {
                self.open_layer("auth");
                Action::Stay
            }
            "sudoers" => {
                self.open_layer("sudo");
                Action::Stay
            }
            "language" => {
                self.open_layer("lang");
                Action::Stay
            }
            other => Action::Kind(other.to_string()),
        }
    }

    fn open_layer(&mut self, layer: &str) {
        self.layer = layer.to_string();
        self.index = 0;
        self.focus = Focus::List;
        self.buffer.clear();
        self.cursor = 0;
    }
}
