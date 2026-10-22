//! Human lines and JSON objects.
//! requirement-shell-output-requirements — success on stdout, errors on stderr.
//! requirement-rust-error-handling

use serde_json::{json, Value};

pub struct Output {
    pub json_mode: bool,
    pub quiet: bool,
    pub capture: bool,
    pub out: String,
    pub err: String,
}

impl Output {
    pub fn new() -> Self {
        Self {
            json_mode: false,
            quiet: false,
            capture: true,
            out: String::new(),
            err: String::new(),
        }
    }

    fn push_out(&mut self, text: &str) {
        self.out.push_str(text);
        if !text.ends_with('\n') {
            self.out.push('\n');
        }
        if !self.capture {
            println!("{text}");
        }
    }

    fn push_err(&mut self, text: &str) {
        self.err.push_str(text);
        if !text.ends_with('\n') {
            self.err.push('\n');
        }
        if !self.capture {
            eprintln!("{text}");
        }
    }

    pub fn info(&mut self, message: &str) {
        if self.quiet || self.json_mode {
            return;
        }
        self.push_out(&format!("[INFO] {message}"));
    }

    pub fn raw(&mut self, text: &str) {
        if self.json_mode {
            return;
        }
        self.push_out(text.trim_end_matches('\n'));
    }

    pub fn error(&mut self, message: &str, next: &str) -> i32 {
        if self.json_mode {
            let body = json!({
                "type": "out_error",
                "message": message,
                "code": "error",
                "next": next,
            });
            self.push_err(&body.to_string());
        } else {
            self.push_err(&format!("ERROR: {message}"));
            self.push_err(&format!("   Next: {next}"));
        }
        1
    }

    pub fn ok(&mut self, kind: &str, message: &str, fields: Option<Value>) -> i32 {
        if self.json_mode {
            let mut body = json!({"type": kind, "message": message});
            if let Some(Value::Object(map)) = fields {
                if let Some(object) = body.as_object_mut() {
                    for (key, value) in map {
                        object.insert(key, value);
                    }
                }
            }
            self.push_out(&body.to_string());
        } else if !self.quiet {
            self.push_out(&format!("[INFO] {message}"));
        }
        0
    }
}

impl Default for Output {
    fn default() -> Self {
        Self::new()
    }
}
