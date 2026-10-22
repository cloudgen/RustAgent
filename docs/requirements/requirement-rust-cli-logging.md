**file**: docs/requirements/requirement-rust-cli-logging.md
**Status**: Active (Version 1.2.0)
**Area**: rust
**Key**: `requirement-rust-cli-logging`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

`main` constructs one `StatusLog` before it dispatches a verb and before it builds the text menu. When the run will open the text screen, the construct is quiet so those lines do not paint over the frame.

This product does not depend on ChronicleLogger.

### 1.1 Human-facing

**In one sentence:** `rust-agent version` can show status lines, and the text menu keeps those lines off the screen.

| You | Another role | Not this |
|-----|--------------|----------|
| Run `rust-agent` | The logger name is the kebab `rust-agent` | A pip install of ChronicleLogger |

## 2. Core Rules

1. `main` **MUST** call `StatusLog::new` once, with the product name `RustAgent`, before verb dispatch and before `Tui` is built. The stored name **MUST** be `rust-agent`.
2. When the run opens the text screen, quiet **MUST** be true at construct, before the log directory is created. A quiet run **MUST NOT** print `Created directory:`.
3. Other modules **MUST NOT** construct a second logger.
4. `--debug` is a product flag. When debug is on and the run is not the text screen, the operator may see debug lines. The text screen stays quiet.
5. view-log and clear-log **MUST** log `read log path=<path>` and `clear log path=<path>` with component `menu` before the read or the empty.
6. A missing logger crate **MUST NOT** be reported as a pip install line. There is no ChronicleLogger dependency.
7. system-log views, clears, and shows that log folder. It does not invent a second log library.

## 3. Acceptance

| AC | Statement |
|----|-----------|
| AC-1 | `StatusLog` resolved name for `RustAgent` is `rust-agent` |
| AC-2 | `Cargo.toml` does not name ChronicleLogger |
| AC-3 | A text-screen run does not print `Created directory:` |
