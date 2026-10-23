**file**: docs/requirements/requirement-rust-packaging.md
**Status**: Active (Version 1.2.1)
**Area**: rust
**Key**: `requirement-rust-packaging`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

Packaging SSOT for RustAgent. Version, crate name, and the console binary live in `Cargo.toml` and `src/identity.rs`.

### 1.1 Human-facing

**In one sentence:** `cargo install rustagent` installs the command `rust-agent` at version 1.2.1.

| You | Another role | Not this |
|-----|--------------|----------|
| Install with cargo | The binary name is `rust-agent` | `sudo pip`, `curl \| sh` as this package’s installer, or the HolaTui author |

## 2. Core Rules

1. `Cargo.toml` `name` **MUST** be `rustagent`. `version` **MUST** be `1.2.1`. `edition` **MUST** be `2021`. `license` **MUST** be `MIT`. Authors **MUST** be `cloudgen <wongcf22@gmail.com>`.
2. The only binary target **MUST** be `rust-agent` with path `src/main.rs`.
3. `Cargo.toml` `version` and `src/identity.rs` `VERSION` **MUST** both be `1.2.1`. `rust-agent version` **MUST** print `RustAgent 1.2.1`.
4. `src/identity.rs` **MUST** hold `APP_NAME` `RustAgent`, `CONSOLE_NAME` `rust-agent`, `CRATE_NAME` `rustagent`, `AUTHOR_NAME` `cloudgen`, `AUTHOR_EMAIL` `wongcf22@gmail.com`, `HOMEPAGE` `https://github.com/cloudgen/RustAgent`, `DOWNLOAD_URL` empty, `BASIC_USAGE` `rust-agent`, `LANG_ENV` `RUSTAGENT_LANG`, `APP_DIR` `RustAgent`, `DEPOSIT` `/var/rust-agent`, and `LAST_UPDATE` `2026-10-23`.
5. Direct dependencies **MUST** include `crossterm`, `unicode-width`, and `serde_json`. There **MUST NOT** be a `pyproject.toml` or a pip dependency.
6. From this tree the install command is `cargo install --path .`. On the registry the install command is `cargo install rustagent`. Neither command uses sudo. `self-update` uses `--force`.
7. The homepage **MUST** be `https://github.com/cloudgen/RustAgent`. Do not copy the HolaTui homepage or author.
8. `build.sh` may wrap `cargo build`. It **MUST NOT** `cargo publish` and **MUST NOT** `git push`. Nothing in this tree uploads a release by itself.
9. Global bin default is `/usr/local/bin/rust-agent`. Sudoers service name is `rust-agent`. Config drafts live under `~/.config/rust-agent`.

## 3. Acceptance

| AC | Statement |
|----|-----------|
| AC-1 | `Cargo.toml` version is `1.2.1` and matches `identity::VERSION` |
| AC-2 | README badge uses `1.2.1` |
| AC-3 | README does not say `sudo pip` or `curl \| sh` |
| AC-4 | No `pyproject.toml` remains |
