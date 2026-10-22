//! About page: identity, host check, and the star box.
//! requirement-rust-about

use crate::check_system::{self, CheckSystem};
use crate::identity::{
    AUTHOR_NAME, BASIC_USAGE, CONSOLE_NAME, DOWNLOAD_URL, HOMEPAGE, LAST_UPDATE,
};

pub struct AboutPage {
    pub check: CheckSystem,
    pub app_name: String,
    pub version: String,
    pub author: String,
    pub updated: String,
    pub homepage: String,
    pub download_url: String,
    pub usage: String,
    pub console_name: String,
}

impl AboutPage {
    pub fn new(check: CheckSystem, app_name: &str, version: &str) -> Self {
        Self {
            check,
            app_name: app_name.to_string(),
            version: version.to_string(),
            author: AUTHOR_NAME.to_string(),
            updated: LAST_UPDATE.to_string(),
            homepage: HOMEPAGE.to_string(),
            download_url: DOWNLOAD_URL.to_string(),
            usage: BASIC_USAGE.to_string(),
            console_name: CONSOLE_NAME.to_string(),
        }
    }

    pub fn install_sentence(kind: &str) -> String {
        match kind {
            "global" => "You are using the GLOBAL INSTALLED version, location:".to_string(),
            "local" => "You are using the LOCAL INSTALLED version, location:".to_string(),
            _ => "You are using an UNINSTALLED version, location:".to_string(),
        }
    }

    pub fn about_box_lines(&self) -> Vec<String> {
        let location = self.check.self_location();
        let kind = check_system::install_kind(&location, &self.console_name, &self.app_name);
        let app = Self::install_sentence(&kind);
        let app_again = app.clone();
        let msg1 = format!(
            "{} ({}) by {} on {}",
            self.app_name, self.version, self.author, self.updated
        );
        let mut msg = vec![
            msg1,
            String::new(),
            app,
            format!("    {location}"),
            String::new(),
            "Basic Usage:".to_string(),
            format!("    {}", self.usage),
            String::new(),
            "Please visit our homepage: ".to_string(),
            format!("    \"{}\"", self.homepage),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
        ];
        if !self.download_url.is_empty() {
            msg = vec![
                format!(
                    "{} ({}) by {} on {}",
                    self.app_name, self.version, self.author, self.updated
                ),
                String::new(),
                app_again,
                format!("    {location}"),
                String::new(),
                "Basic Usage:".to_string(),
                format!("    {}", self.usage),
                String::new(),
                "Please visit our homepage: ".to_string(),
                format!("    \"{}\"", self.homepage),
                String::new(),
                "Installation command:".to_string(),
                format!("    curl -fsSL {} | sh", self.download_url),
                String::new(),
            ];
        }
        let max_line = if self.download_url.is_empty() {
            if self.homepage.is_empty() {
                msg.len().saturating_sub(3)
            } else {
                msg.len().saturating_sub(2)
            }
        } else {
            msg.len().saturating_sub(1)
        };
        let max_line = if max_line < 1 { msg.len() } else { max_line };
        let max_len = msg.iter().take(max_line).map(|line| line.len()).max().unwrap_or(0);
        let star = "*".repeat(max_len + 4);
        let blank = " ".repeat(max_len);
        let mut lines = vec![star.clone(), format!("* {blank} *")];
        for row in msg.iter().take(max_line) {
            lines.push(format!("* {}{} *", row, " ".repeat(max_len - row.len())));
        }
        lines.push(format!("* {blank} *"));
        lines.push(star);
        lines
    }

    pub fn framework_about(&self) -> String {
        let mut lines = vec![
            format!("{} {}", self.app_name, self.version),
            "Domain: Install and run xAI grok, and back up its auth files.".to_string(),
            "Runtime tools: curl, grok".to_string(),
            format!("Entry points: {}", self.console_name),
            String::new(),
        ];
        lines.extend(self.check.check_system_lines());
        lines.push(String::new());
        lines.extend(self.about_box_lines());
        lines.join("\n")
    }
}
