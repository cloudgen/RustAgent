//! README and packaging contract. requirement-rust-readme, requirement-rust-packaging.

use std::fs;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

const HEADINGS: &[&str] = &[
    "Features",
    "Advantages",
    "Quick Installation",
    "Usage",
    "Screenshots",
    "Examples",
    "Platform Compatibility",
    "Related Projects",
    "Contributing",
    "License",
    "Last Update",
];

#[test]
fn readme_matches_crate_version_and_menu() {
    let root = root();
    let readme = fs::read_to_string(root.join("README.md")).expect("README");
    let cargo = fs::read_to_string(root.join("Cargo.toml")).expect("Cargo.toml");
    assert!(cargo.contains("name = \"rustagent\""));
    assert!(cargo.contains("version = \"1.2.0\""));
    assert_eq!(rustagent::identity::VERSION, "1.2.0");
    assert!(readme.contains("Version-1.2.0-blue"));
    assert!(readme.contains("HH:MM:SS"));
    assert!(readme.contains("3. system-log"));
    assert!(readme.contains("~/.local/RustAgent"));
    for banned in ["setup.sh", "sudo pip", "curl | sh", "video.png", "join-first.png"] {
        assert!(!readme.contains(banned), "{banned}");
    }
    let headings: Vec<&str> = readme
        .lines()
        .filter_map(|line| line.strip_prefix("## "))
        .filter(|title| !title.starts_with('#'))
        .collect();
    assert_eq!(headings, HEADINGS);
    let shots = fs::read_dir(root.join("screenshots")).expect("screenshots");
    for entry in shots {
        let name = entry.expect("shot").file_name();
        let name = name.to_string_lossy();
        if !name.ends_with(".png") {
            continue;
        }
        let url = format!(
            "https://raw.githubusercontent.com/cloudgen/agent-cli/main/screenshots/{name}"
        );
        assert!(readme.contains(&url), "{name}");
    }
    assert!(!readme.contains("](screenshots/"));
    assert!(!readme.contains("](./screenshots/"));
    let related = readme
        .split("## Related Projects")
        .nth(1)
        .expect("related")
        .split("## Contributing")
        .next()
        .expect("contributing");
    for url in [
        "https://github.com/cloudgen/RustAgent",
        "https://github.com/cloudgen/grok-cli",
        "https://github.com/cloudgen/ciao",
        "https://github.com/cloudgen/ciao-lite",
        "https://github.com/cloudgen/safe-rm",
    ] {
        assert!(related.contains(url), "{url}");
    }
    assert!(!related.contains("pypi.org"));
    assert!(!related.contains("ChronicleLogger"));
    assert!(!root.join("pyproject.toml").exists());
    assert!(!root.join("src/AgentCli").exists());
}
