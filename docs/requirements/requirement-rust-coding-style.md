**file**: docs/requirements/requirement-rust-coding-style.md
**Status**: Active (Version 1.2.0)
**Area**: rust
**Key**: `requirement-rust-coding-style`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

Rust style for the RustAgent crate. Product behavior stays on the peer requirements. This file says how that code is written.

### 1.1 Human-facing

**In one sentence:** When a step cannot finish, the program says what happened and the next command, and it does not pipe a download into a shell.

| You | Another role | Not this |
|-----|--------------|----------|
| Read `ERROR:` and `Next:` | Domain procedures stay in their requirement files | A vendor `install.sh` or a media publish path |

## 2. Core Rules

1. Source **MUST** compile as Rust edition 2021. The compiler floor is rustc 1.74.
2. Each `src/*.rs` file that implements product law **MUST** name a live `requirement-*.md`. It **MUST NOT** cite a requirement file that is not on disk.
3. Expected failures **MUST** return a non-zero status and an operator-readable message with a next step. **MUST NOT** claim success after a failed copy, download, or crontab install.
4. Scratch files **MUST** go through `Storage`. A missed cache tier **MUST** be silent. The next tier is tried.
5. **MUST NOT** pipe a downloaded script into a shell. Source **MUST NOT** contain the string `install.sh`. The admin script may use the `install` command and `cmd_install`.
6. Root detection **MUST** use the effective user id. It **MUST NOT** spawn `id`. Menu columns **MUST** use `unicode-width` with ambiguous width 1.
7. Deposit path **MUST** come from `identity::DEPOSIT` (`/var/rust-agent`). Do not scatter a second deposit constant.
8. Source **MUST NOT** contain unfinished markers (`TODO`, `FIXME`) as delivery. There is no `Join` module and no FFmpeg helper.
9. Product source comments cite only live `requirement-*.md`.

## 3. Acceptance

| AC | Statement |
|----|-----------|
| AC-1 | `Cargo.toml` edition is 2021 |
| AC-2 | `src/**/*.rs` and the wrapper template do not contain `install.sh` |
| AC-3 | Command-line failures use `ERROR:` and `Next:` |
