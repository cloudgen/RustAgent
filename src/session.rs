//! Run the text menu until Exit.
//! requirement-rust-tui — one-second clock wait on the four boards.
//! requirement-rust-cli-language — a language pick writes the leaf.

use crate::model::{Action, Key, MenuModel, Phase};
use crate::painter::Screen;

pub type KindFn = Box<dyn FnMut(&str, &mut dyn Screen, &mut MenuModel) -> Option<String>>;

pub struct MenuSession {
    pub app_name: String,
    pub version: String,
    pub model: MenuModel,
    pub on_version: Box<dyn Fn() -> String>,
    pub on_about: Box<dyn Fn() -> String>,
    pub on_kind: Option<KindFn>,
}

impl MenuSession {
    pub fn run(&mut self, screen: &mut dyn Screen) -> Result<(), String> {
        let (height, width) = screen.size();
        if !self.model.painter.borrow().screen_can_hold_here(height, width) {
            return Err("too-small".to_string());
        }
        loop {
            self.paint(screen);
            let (height, _) = screen.size();
            let Some(key) = self.wait_key(screen) else {
                continue;
            };
            if key == Key::Interrupt {
                return Err("interrupt".to_string());
            }
            let action = self.model.apply_key(key, height);
            if self.take_action(action, screen) == "exit" {
                return Ok(());
            }
        }
    }

    fn paint(&mut self, screen: &mut dyn Screen) {
        let rows = self.model.rows();
        let painter = std::rc::Rc::clone(&self.model.painter);
        painter.borrow().paint(
            screen,
            &mut self.model,
            &rows,
            &self.app_name,
            &self.version,
        );
    }

    fn wait_key(&mut self, screen: &mut dyn Screen) -> Option<Key> {
        let show_clock = self.model.phase != Phase::Result
            && matches!(self.model.layer.as_str(), "front" | "self" | "log" | "lang" | "auth" | "sudo");
        if show_clock {
            screen.poll_key(1000)
        } else {
            Some(screen.read_key())
        }
    }

    pub fn take_action(&mut self, action: Action, screen: &mut dyn Screen) -> &'static str {
        match action {
            Action::Exit => "exit",
            Action::Stay => "stay",
            Action::Kind(kind) if kind.starts_with("set-") => {
                let code = kind[4..].to_string();
                let message = self.model.painter.borrow_mut().language.save(&code);
                self.model.show_front();
                self.model.notice = message;
                "stay"
            }
            Action::Kind(kind) if kind == "version" => {
                let text = (self.on_version)();
                self.model.show_result(text);
                "stay"
            }
            Action::Kind(kind) if kind == "about" => {
                let text = (self.on_about)();
                self.model.show_result(text);
                "stay"
            }
            Action::Kind(kind) => {
                let mut callback = self.on_kind.take();
                let outcome = if let Some(ref mut function) = callback {
                    function(&kind, screen, &mut self.model)
                } else {
                    None
                };
                self.on_kind = callback;
                if outcome.as_deref() == Some("leave") {
                    return "exit";
                }
                if let Some(text) = outcome {
                    if !text.is_empty() {
                        self.model.show_result(text);
                    }
                }
                "stay"
            }
        }
    }
}
