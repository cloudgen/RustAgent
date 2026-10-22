**file**: docs/requirements/requirement-rust-readme.md
**Status**: Active (Version 1.2.0)
**Area**: rust
**Key**: `requirement-rust-readme`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

The root README is the user document for RustAgent. The live menu is a text picture. Older PNG captures stay labeled as AgentCli pictures.

### 1.1 Human-facing

**In one sentence:** The README tells you how to install `rust-agent` with cargo and shows the text menu.

| You | Another role | Not this |
|-----|--------------|----------|
| Read Features, cargo install, and the text menu | Behavior law stays in requirements | A relative `screenshots/` image, a PyPI install, or a claim that old PNGs show RustAgent 1.2.0 |

## 2. Core Rules

1. The version badge **MUST** be `1.2.0` and **MUST** match `Cargo.toml` `version` and `identity::VERSION`.
2. Level-2 headings **MUST** be, in order: Features, Advantages, Quick Installation, Usage, Screenshots, Examples, Platform Compatibility, Related Projects, Contributing, License, Last Update.
3. The primary menu picture **MUST** be a fenced text picture of the live front board: rows **1** grok-auth, **3** system-log, **5** sudoers, **6** self-management, **7** language, **9** Exit, clock `HH:MM:SS`, and status `RustAgent 1.2.0`.
4. Every PNG **MUST** use an absolute `https://raw.githubusercontent.com/` URL. **MUST NOT** use a relative `screenshots/` link.
5. Captions **MUST** say the existing PNGs are earlier AgentCli captures when they still show AgentCli. **MUST NOT** claim those PNGs show RustAgent 1.2.0. The system-log caption **MUST** say the live front board puts system-log on row **3**. The self-management caption **MUST** say the live rows are cargo **62**–**68**.
6. **MUST NOT** mention `setup.sh`, `sudo pip`, `curl | sh`, `video.png`, or `join-first.png`.
7. The README **MUST** mention the local clock and `HH:MM:SS`, the leaf `~/.local/RustAgent/language`, and `RUSTAGENT_LANG`.
8. Related Projects **MUST** link GitHub `cloudgen/RustAgent`, grok-cli, CIAO, CIAO-Lite, and safe-rm. It **MUST NOT** list PyPI AgentCli or ChronicleLogger as this product’s installer.
9. Install commands **MUST** be `cargo install --path .` and `cargo install rustagent`. **MUST NOT** document `pip install`.
10. The README **MUST NOT** list skills, molds, or the agent inventory.

## 3. Acceptance

| AC | Statement |
|----|-----------|
| AC-1 | Badge, headings, clock, and absolute picture URLs |
| AC-2 | No `sudo pip`, `curl \| sh`, `setup.sh`, `video.png`, or `join-first.png` |
| AC-3 | The text menu picture includes row 3 system-log |
