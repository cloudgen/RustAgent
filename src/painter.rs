//! Text-menu frame.
//! requirement-rust-tui — menu region and the bottom input box.
//! requirement-rust-cli-language — row 7 and the path word.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::language::MenuLanguage;
use crate::width::{self, clip_columns, clip_columns_right, display_width};

// =============================================================================
// CIAO-Lite Protection Zone
// Do NOT simplify, refactor, or remove without explicit user instruction.
// The frame glyphs are the bottom input box. requirement-rust-tui.md
// =============================================================================
pub const INPUT_MARK: &str = "> ";
pub const INPUT_HINT: &str = "Up/Down  •  Enter";
pub const PATH_LABEL_EN: &str = "Path";
pub const PATH_LABEL_ZH_HANT: &str = "路徑";
pub const FRAME_TOP_LEFT: &str = "╭";
pub const FRAME_TOP_RIGHT: &str = "╮";
pub const FRAME_BOTTOM_LEFT: &str = "╰";
pub const FRAME_BOTTOM_RIGHT: &str = "╯";
pub const FRAME_HORIZ: &str = "─";
pub const FRAME_VERT: &str = "│";
pub const CARET_BLOCK: &str = "█";
pub const FRAME_ROWS: u16 = 3;
pub const FOOTER_ROWS: u16 = 1;
pub const CHROME_ROWS: u16 = FRAME_ROWS + FOOTER_ROWS;
pub const MIN_MENU_ABOVE: u16 = 3;
pub const MIN_HEIGHT: u16 = MIN_MENU_ABOVE + CHROME_ROWS;
pub const MIN_PLACEABLE: u16 = 6;

const SELF_ROWS: &[(i32, &str, &str, &str)] = &[
    (62, "version", "show the installed version", "version"),
    (63, "about", "version and this computer", "about"),
    (64, "version-check", "compare this install with cargo", "version-check"),
    (65, "self-update", "upgrade this package with cargo", "self-update"),
    (66, "self-uninstall", "remove this package with cargo", "self-uninstall"),
    (67, "self-install", "install this package with cargo", "self-install"),
    (68, "where-is-me", "show running and install paths", "where-is-me"),
    (0, "Back", "return to the main menu", "back"),
];

const SUDO_ROWS: &[(i32, &str, &str, &str)] = &[
    (51, "generate-sudoer-request", "write the backup grant JSON", "generate-sudoer-request"),
    (52, "submit-sudoer-request", "hand the JSON to the submitter", "submit-sudoer-request"),
    (53, "print-sudoers", "print the backup grant", "print-sudoers"),
    (54, "print-sudoers-install-script", "print the install commands", "print-sudoers-install-script"),
    (55, "remove-project-sudoers", "print how to remove the grant", "remove-project-sudoers"),
    (0, "Back", "return to the main menu", "back"),
];

const LOG_ROWS: &[(i32, &str, &str, &str)] = &[
    (31, "view-log", "list a log file and show it", "view-log"),
    (32, "clear-log", "empty one log file", "clear-log"),
    (33, "log-folder", "show the log folder", "log-folder"),
    (0, "Back", "return to the main menu", "back"),
];

pub fn any_board(token: &str) -> Option<&'static str> {
    match token {
        "help" => Some("help"),
        "version" => Some("version"),
        "about" => Some("about"),
        "hello" => Some("hello"),
        "self-install" => Some("self-install"),
        "version-check" => Some("version-check"),
        "self-update" => Some("self-update"),
        "self-uninstall" => Some("self-uninstall"),
        "self-management" => Some("self-management"),
        "system-log" => Some("system-log"),
        "view-log" => Some("view-log"),
        "clear-log" => Some("clear-log"),
        "log-folder" => Some("log-folder"),
        "grok-auth" => Some("grok-auth"),
        "sudoers" => Some("sudoers"),
        "run" => Some("run"),
        "backup" => Some("backup"),
        "sync-auth" => Some("sync-auth"),
        "sync-auth-from-remote" => Some("sync-auth-from-remote"),
        "sync-auth-to-remote" => Some("sync-auth-to-remote"),
        "add-crontab" => Some("add-crontab"),
        "setup" => Some("setup"),
        "reinstall" => Some("reinstall"),
        "update-grok" => Some("update-grok"),
        "check-session" => Some("check-session"),
        "where-is-me" => Some("where-is-me"),
        "print-sudoers" => Some("print-sudoers"),
        "print-sudoers-install-script" => Some("print-sudoers-install-script"),
        "remove-project-sudoers" => Some("remove-project-sudoers"),
        "generate-sudoer-request" => Some("generate-sudoer-request"),
        "submit-sudoer-request" => Some("submit-sudoer-request"),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub number: i32,
    pub short: String,
    pub explain: String,
    pub kind: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Style {
    pub bold: bool,
    pub italic: bool,
    pub reverse: bool,
    pub underline: bool,
}

pub trait Screen {
    fn size(&self) -> (u16, u16);
    fn erase(&mut self);
    fn put(&mut self, y: u16, x: u16, text: &str, style: Style);
    fn refresh(&mut self);
    fn move_to(&mut self, y: u16, x: u16);
    fn read_key(&mut self) -> crate::model::Key;
    fn poll_key(&mut self, timeout_ms: u64) -> Option<crate::model::Key>;
}

pub struct MenuPainter {
    pub language: MenuLanguage,
    clock_override: Option<String>,
    pub this_login_only: bool,
    pub logged_in: bool,
    pub unavailable: Vec<String>,
}

impl MenuPainter {
    pub fn new(home: Option<String>) -> Self {
        Self {
            language: MenuLanguage::new(home),
            clock_override: None,
            this_login_only: false,
            logged_in: false,
            unavailable: Vec::new(),
        }
    }

    pub fn chrome_rows(&self) -> u16 {
        FRAME_ROWS + FOOTER_ROWS + self.unavailable.len() as u16
    }

    pub fn screen_can_hold_here(&self, height: u16, width: u16) -> bool {
        height >= MIN_MENU_ABOVE + self.chrome_rows() && width >= MIN_PLACEABLE
    }

    pub fn set_clock(&mut self, clock: Option<String>) {
        self.clock_override = clock;
    }

    pub fn display_width(text: &str) -> usize {
        display_width(text)
    }

    pub fn path_label(&self) -> String {
        self.language.path_label()
    }

    pub fn clock_text(&self) -> String {
        if let Some(clock) = &self.clock_override {
            return clock.clone();
        }
        clock_now()
    }

    pub fn path_line(&self, placeable: Option<usize>) -> String {
        let prefix = format!("{}: ", self.path_label());
        let current = std::env::current_dir()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_else(|_| ".".to_string());
        let left = format!("{prefix}{current}");
        let right = self.clock_text();
        let gap = 2usize;
        let left_width = display_width(&left);
        let right_width = display_width(&right);
        let prefix_width = display_width(&prefix);
        let Some(placeable) = placeable else {
            return format!("{left}{}{right}", " ".repeat(gap));
        };
        if placeable >= left_width + gap + right_width {
            let spaces = placeable - left_width - right_width;
            return format!("{left}{}{right}", " ".repeat(spaces));
        }
        if placeable >= left_width {
            return left;
        }
        if placeable <= prefix_width {
            return clip_columns(&prefix, placeable);
        }
        format!(
            "{prefix}{}",
            clip_columns_right(&current, placeable - prefix_width)
        )
    }

    pub fn front_table(&self) -> Vec<(i32, &'static str, &'static str, &'static str)> {
        let mut rows = vec![(1, "grok-auth", "backup, sync, crontab, setup, and reinstall", "grok-auth")];
        if self.this_login_only {
            rows.push((2, "run", "start grok without auto-update", "run"));
        }
        rows.push((3, "system-log", "view, clear, and the log folder", "system-log"));
        if !self.this_login_only {
            rows.push((5, "sudoers", "grant and drafts", "sudoers"));
        }
        rows.push((6, "self-management", "version, about, and cargo lifecycle", "self-management"));
        rows.push((7, "language", "display language for this menu", "language"));
        rows.push((9, "Exit", "leave", "exit"));
        rows
    }

    pub fn auth_table(&self) -> Vec<(i32, &'static str, &'static str, &'static str)> {
        let mut rows = Vec::new();
        if self.logged_in && !self.this_login_only {
            rows.push((11, "backup", "deposit auth files as root", "backup"));
        }
        if !self.logged_in {
            rows.push((12, "sync-auth-from-remote", "copy auth from another login", "sync-auth-from-remote"));
            if !self.this_login_only {
                rows.push((13, "sync-auth", "restore auth files here", "sync-auth"));
            }
        }
        rows.push((14, "add-crontab", "schedule backup and sync-auth", "add-crontab"));
        rows.push((15, "setup", "install or keep grok", "setup"));
        rows.push((16, "reinstall", "replace grok and keep auth files", "reinstall"));
        rows.push((0, "Back", "return to the main menu", "back"));
        rows
    }

    pub fn rows_for(&self, layer: &str) -> Vec<(i32, &'static str, &'static str, &'static str)> {
        match layer {
            "self" => SELF_ROWS.to_vec(),
            "log" => LOG_ROWS.to_vec(),
            "sudo" => SUDO_ROWS.to_vec(),
            "auth" => self.auth_table(),
            _ => self.front_table(),
        }
    }

    pub fn display_rows(&self, layer: &str) -> Vec<Row> {
        if layer == "lang" {
            let mut rows = Vec::new();
            for item in crate::lang_catalog::CODES {
                rows.push(Row {
                    number: item.number,
                    short: item.endonym.to_string(),
                    explain: self.language.lang_long(item.endonym),
                    kind: format!("set-{}", item.code),
                });
            }
            rows.push(Row {
                number: 0,
                short: self.language.row_short("back", "Back"),
                explain: self.language.row_long("back", "return to the main menu"),
                kind: "back".to_string(),
            });
            return rows;
        }
        self.rows_for(layer)
            .iter()
            .map(|(number, short, explain, kind)| Row {
                number: *number,
                short: self.language.row_short(kind, short),
                explain: self.language.row_long(kind, explain),
                kind: (*kind).to_string(),
            })
            .collect()
    }

    pub fn board_title(&self, phase_result: bool, layer: &str) -> String {
        if phase_result {
            return "result".to_string();
        }
        match layer {
            "self" => self.language.text("title_self"),
            "log" => self.language.text("title_log"),
            "lang" => self.language.text("title_language"),
            "auth" => "grok-auth".to_string(),
            "sudo" => "sudoers".to_string(),
            _ => self.language.text("title_main"),
        }
    }

    pub fn row_parts<'a>(&self, rows: &'a [Row]) -> Vec<(String, &'a str, String, &'a str)> {
        let mut number_width = 1usize;
        let mut verb_width = 0usize;
        for row in rows {
            let text = row.number.to_string();
            number_width = number_width.max(text.len());
            verb_width = verb_width.max(display_width(&row.short));
        }
        rows.iter()
            .map(|row| {
                let text = row.number.to_string();
                let number_field = format!("{text:>number_width$}. ");
                let pad = verb_width.saturating_sub(display_width(&row.short));
                (
                    number_field,
                    row.short.as_str(),
                    " ".repeat(pad),
                    row.explain.as_str(),
                )
            })
            .collect()
    }

    pub fn format_rows(&self, rows: &[Row]) -> Vec<String> {
        self.row_parts(rows)
            .into_iter()
            .map(|(number_field, short, verb_pad, explain)| {
                format!("{number_field}{short}{verb_pad}: {explain}")
            })
            .collect()
    }

    pub fn screen_can_hold(height: u16, width: u16) -> bool {
        height >= MIN_HEIGHT && width >= MIN_PLACEABLE
    }

    pub fn result_overflow(height: u16, line_count: usize) -> bool {
        line_count > (height.saturating_sub(3) as usize).max(1)
            && line_count > height.saturating_sub(3) as usize
    }

    pub fn result_room(height: u16, line_count: usize) -> usize {
        if line_count > height.saturating_sub(3) as usize {
            (height.saturating_sub(4) as usize).max(1)
        } else {
            (height.saturating_sub(3) as usize).max(1)
        }
    }

    pub fn paint(
        &self,
        screen: &mut dyn Screen,
        model: &mut crate::model::MenuModel,
        rows: &[Row],
        app_name: &str,
        version: &str,
    ) {
        screen.erase();
        let italic = Style { italic: true, ..Style::default() };
        let title = self.board_title(model.phase_is_result(), &model.layer);
        let (height, width) = screen.size();
        if model.phase_is_result() {
            self.put(screen, 0, 0, app_name, Style { bold: true, ..Style::default() }, width);
            let mut cursor = display_width(app_name) as u16;
            self.put(screen, 0, cursor, " (", Style::default(), width);
            cursor += 2;
            self.put(screen, 0, cursor, version, italic, width);
            cursor += version.len() as u16;
            self.put(screen, 0, cursor, &format!(") — {title}"), Style::default(), width);
            let lines: Vec<&str> = if model.result_text.is_empty() {
                vec![""]
            } else {
                model.result_text.lines().collect()
            };
            let lines = if lines.is_empty() { vec![""] } else { lines };
            let room = Self::result_room(height, lines.len());
            let max_offset = lines.len().saturating_sub(room);
            if model.result_offset > max_offset {
                model.result_offset = max_offset;
            }
            let window = &lines[model.result_offset..model.result_offset.min(lines.len()).saturating_add(room).min(lines.len())];
            let mut y = 2u16;
            for line in window {
                if y >= height.saturating_sub(1) {
                    break;
                }
                self.put(screen, y, 0, line, Style::default(), width);
                y += 1;
            }
            if lines.len() > height.saturating_sub(3) as usize {
                self.put(screen, height.saturating_sub(2), 0, "Up/Down scrolls this page.", italic, width);
                self.put(
                    screen,
                    height.saturating_sub(1),
                    0,
                    "Press a key to return to the main menu.",
                    italic,
                    width,
                );
            } else {
                let mut hint_y = y + 1;
                if hint_y >= height {
                    hint_y = height.saturating_sub(1);
                }
                self.put(
                    screen,
                    hint_y,
                    0,
                    "Press a key to return to the main menu.",
                    italic,
                    width,
                );
            }
            screen.refresh();
            return;
        }
        let placeable = width.saturating_sub(0) as usize;
        self.put(screen, 0, 0, &self.path_line(Some(placeable)), Style::default(), width);
        let box_top = height.saturating_sub(self.chrome_rows());
        let banner = model.banner();
        let mut stop = box_top;
        if !banner.is_empty() && box_top >= 4 {
            stop = box_top - 1;
        }
        let mut y = 2u16;
        if y < stop {
            for (idx, (number_field, short, verb_pad, explain)) in self.row_parts(rows).into_iter().enumerate() {
                if y >= stop {
                    break;
                }
                let selected = idx == model.index;
                let attr = if selected && model.focus_is_list() {
                    Style { reverse: true, ..Style::default() }
                } else if selected {
                    Style { underline: true, ..Style::default() }
                } else {
                    Style::default()
                };
                self.put(screen, y, 0, &number_field, attr, width);
                let verb_x = display_width(&number_field) as u16;
                let mut verb_style = attr;
                verb_style.bold = true;
                self.put(screen, y, verb_x, short, verb_style, width);
                let colon_x = verb_x + display_width(short) as u16;
                self.put(screen, y, colon_x, &format!("{verb_pad}: "), attr, width);
                let explain_x = colon_x + display_width(&verb_pad) as u16 + 2;
                let mut explain_style = attr;
                explain_style.italic = true;
                self.put(screen, y, explain_x, explain, explain_style, width);
                y += 1;
            }
        }
        if !banner.is_empty() && box_top >= 4 {
            self.put(
                screen,
                box_top - 1,
                0,
                &banner,
                Style { bold: true, ..Style::default() },
                width,
            );
        }
        self.paint_box(screen, model, box_top, app_name, version, &title, width);
        screen.refresh();
    }

    pub fn paint_prompt(
        &self,
        screen: &mut dyn Screen,
        model: &crate::model::MenuModel,
        title: &str,
        lines: &[String],
        pin_last: bool,
        app_name: &str,
        version: &str,
    ) {
        screen.erase();
        let italic = Style { italic: true, ..Style::default() };
        let (height, width) = screen.size();
        self.put(screen, 0, 0, app_name, Style { bold: true, ..Style::default() }, width);
        let mut cursor = display_width(app_name) as u16;
        self.put(screen, 0, cursor, " (", Style::default(), width);
        cursor += 2;
        self.put(screen, 0, cursor, version, italic, width);
        cursor += version.len() as u16;
        self.put(screen, 0, cursor, &format!(") — {title}"), Style::default(), width);
        let box_top = height.saturating_sub(self.chrome_rows());
        let banner = model.banner();
        let mut stop = box_top;
        if !banner.is_empty() && box_top >= 4 {
            stop = box_top - 1;
        }
        let room = stop.saturating_sub(2) as usize;
        let shown = visible_lines(lines, room, pin_last);
        let mut y = 2u16;
        for line in shown {
            if y >= stop {
                break;
            }
            self.put(screen, y, 0, &line, Style::default(), width);
            y += 1;
        }
        if !banner.is_empty() && box_top >= 4 {
            self.put(
                screen,
                box_top - 1,
                0,
                &banner,
                Style { bold: true, ..Style::default() },
                width,
            );
        }
        self.paint_box(screen, model, box_top, app_name, version, title, width);
        screen.refresh();
    }

    fn paint_box(
        &self,
        screen: &mut dyn Screen,
        model: &crate::model::MenuModel,
        top: u16,
        app_name: &str,
        version: &str,
        title: &str,
        width: u16,
    ) {
        let placeable = width as usize;
        if placeable < 2 {
            return;
        }
        let inner = placeable - 2;
        let (field, caret_at) = input_field(model, inner);
        let side = FRAME_VERT;
        let lines = [
            format!("{FRAME_TOP_LEFT}{}{FRAME_TOP_RIGHT}", FRAME_HORIZ.repeat(inner)),
            format!("{side}{field}{side}"),
            format!(
                "{FRAME_BOTTOM_LEFT}{}{FRAME_BOTTOM_RIGHT}",
                FRAME_HORIZ.repeat(inner)
            ),
        ];
        for (offset, line) in lines.iter().enumerate() {
            self.put(screen, top + offset as u16, 0, line, Style::default(), width);
        }
        let mut y = top + FRAME_ROWS;
        for line in &self.unavailable {
            self.put(screen, y, 0, line, Style::default(), width);
            y += 1;
        }
        self.put(
            screen,
            y,
            0,
            &status_line(app_name, version, title),
            Style::default(),
            width,
        );
        if let Some(caret_at) = caret_at {
            screen.move_to(top + 1, 1 + caret_at as u16);
        }
    }

    fn put(&self, screen: &mut dyn Screen, y: u16, x: u16, text: &str, style: Style, width: u16) {
        if x >= width {
            return;
        }
        let room = (width - x) as usize;
        let clipped = clip_columns(text, room);
        if !clipped.is_empty() {
            screen.put(y, x, &clipped, style);
        }
    }
}

fn status_line(app_name: &str, version: &str, title: &str) -> String {
    format!("  {app_name} {version}  {FRAME_VERT}  {title}  {FRAME_VERT}  {INPUT_HINT}")
}

fn input_field(model: &crate::model::MenuModel, inner: usize) -> (String, Option<usize>) {
    let prefix = format!(" {INPUT_MARK}");
    let focused = !model.focus_is_list();
    let chars: Vec<char> = model.buffer.chars().collect();
    let cursor = model.cursor.min(chars.len());
    let reserve = prefix.chars().count() + if focused { 1 } else { 0 };
    let text_room = inner.saturating_sub(reserve);
    let mut start = 0usize;
    if text_room > 0 && cursor > text_room {
        start = cursor - text_room;
    }
    if text_room > 0 && start > chars.len().saturating_sub(text_room) {
        start = chars.len().saturating_sub(text_room);
    }
    let shown: String = if text_room == 0 {
        String::new()
    } else {
        chars.iter().skip(start).take(text_room).collect()
    };
    let rel = cursor.saturating_sub(start).min(shown.chars().count());
    let (body, caret_at) = if focused && inner >= reserve {
        let mut body = String::new();
        for (index, ch) in shown.chars().enumerate() {
            if index == rel {
                body.push_str(CARET_BLOCK);
            }
            body.push(ch);
        }
        if rel == shown.chars().count() {
            body.push_str(CARET_BLOCK);
        }
        (body, Some(prefix.chars().count() + rel))
    } else {
        (shown, None)
    };
    let mut field: String = prefix + &body;
    let field_len = field.chars().count();
    if field_len < inner {
        field.push_str(&" ".repeat(inner - field_len));
    } else if field_len > inner {
        field = field.chars().take(inner).collect();
    }
    let caret_at = caret_at.filter(|at| *at < inner);
    (field, caret_at)
}

pub fn visible_lines(lines: &[String], room: usize, pin_last: bool) -> Vec<String> {
    if room < 1 {
        return Vec::new();
    }
    if lines.len() <= room {
        return lines.to_vec();
    }
    if !pin_last {
        return lines[lines.len() - room..].to_vec();
    }
    if room == 1 {
        return vec![lines[lines.len() - 1].clone()];
    }
    let mut out = lines[..room - 2].to_vec();
    out.push("…".to_string());
    out.push(lines[lines.len() - 1].clone());
    out
}

fn clock_now() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let offset = unsafe { clock_offset(secs) };
    let local = secs + offset;
    let tod = local.rem_euclid(86_400) as u32;
    format!("{:02}:{:02}:{:02}", tod / 3600, (tod % 3600) / 60, tod % 60)
}

unsafe fn clock_offset(secs: i64) -> i64 {
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
    }
    let mut local = std::mem::zeroed::<Tm>();
    if localtime_r(&secs, &mut local).is_null() {
        return 0;
    }
    local.tm_gmtoff
}

pub fn char_columns(text: &str) -> usize {
    width::display_width(text)
}
