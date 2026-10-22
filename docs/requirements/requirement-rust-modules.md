**file**: docs/requirements/requirement-rust-modules.md
**Status**: Active (Version 1.2.0)
**Area**: rust
**Key**: `requirement-rust-modules`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

This file names the Rust modules that implement RustAgent and which requirement owns each module’s behavior. It replaces `requirement-python-oop`. There is no Python package and no class named `Join`.

### 1.1 Human-facing

**In one sentence:** Menu drawing, grok work, and the about page live in separate modules under `src/`, and `rust-agent` starts at `src/main.rs`.

## 2. Core Rules

1. The library crate **MUST** be `rustagent`. `src/lib.rs` **MUST** declare the modules in the table and no Python module.
2. `src/main.rs` **MUST** call `rustagent::cli::main` and exit with that code.
3. A module **MUST NOT** take over a behavior owned by another row.

| Module | Owns | Law |
|--------|------|-----|
| `src/identity.rs` | Name, version, console, crate, author, homepage, verb list, deposit | `requirement-rust-packaging` |
| `src/cli.rs` | Parse, dispatch, screen gate, operator errors | `requirement-rust-cli-interface`, `requirement-rust-error-handling` |
| `src/tui.rs` | Text-screen session and menu kind outcomes | `requirement-rust-tui` |
| `src/painter.rs` | Frame, path line, clock, row tables | `requirement-rust-tui` |
| `src/model.rs` | Focus, buffer, key actions | `requirement-rust-tui` |
| `src/session.rs` | Board loop and the one-second clock wait | `requirement-rust-tui` |
| `src/language.rs` | Leaf load and save, `RUSTAGENT_LANG` | `requirement-rust-cli-language` |
| `src/lang_catalog.rs` | Thirteen language strings | `requirement-rust-cli-language` |
| `src/logger.rs` | `StatusLog` | `requirement-rust-cli-logging` |
| `src/system_log.rs` | List, read, and clear log files | `requirement-rust-tui` |
| `src/self_manage.rs` | Cargo argv for rows 64–67 | `requirement-rust-cli-interface` |
| `src/about.rs` | About page text | `requirement-rust-about` |
| `src/check_system.rs` | Host-check reads and install kind | `requirement-rust-about` |
| `src/width.rs` | Display columns | `requirement-rust-tui` |
| `src/domain.rs` | Setup, session, backup, sync, crontab, sudoer JSON | `requirement-domain-grok-cli` |
| `src/host.rs` | This-login-only detect | `requirement-rust-tui` |
| `src/output.rs` | Human lines and JSON | `requirement-rust-error-handling` |
| `src/storage.rs` | Cache tiers and scratch files | `requirement-shell-cli-storage` |
| `src/shell_startup.rs` | rc files | `requirement-shell-startup-files` |

4. **MUST NOT** add a module whose job is joining, listing, or encoding media.
5. `StatusLog` is constructed in `cli` before dispatch. Other modules **MUST NOT** construct it at startup.

## 3. Acceptance

| AC | Statement |
|----|-----------|
| AC-1 | `src/lib.rs` declares every module in the table |
| AC-2 | `src/main.rs` exits with `rustagent::cli::main` |
| AC-3 | No `src/AgentCli/` package remains |
| AC-4 | There is no `Join` type |
