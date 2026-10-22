# Review — RustAgent specialize from HolaTui

**Date:** 2026-10-22  
**Direction:** A is sibling HolaTui (not edited). B is this workspace, RustAgent.  
**Ship unit:** crate `rustagent` 1.2.0, binary `rust-agent`.

## Combine

| Surface | A (read only) | B kept | Combine on B |
|---------|---------------|--------|----------------|
| Frame | Path, clock `HH:MM:SS`, arc box, status line | grok-auth, sudoers, run, language 80–92 | Front row **3** system-log; self rows **62–68** cargo; domain leaf shorts stay English |
| CLI | hello, help, version, about, cargo lifecycle | Domain verbs, `--json`, rc-test, where-is-me | Both sets. `language` and `system-log` stay menu-only |
| Domain | Welcome sentence only | grok setup, auth, crontab, sudoer JSON | One Active domain file: `requirement-domain-grok-cli` |
| Package | cargo crate holatui | Version 1.2.0, author cloudgen | Crate `rustagent`, homepage `https://github.com/cloudgen/RustAgent` |

## Law

Python `requirement-python-*` files are superseded and point at `requirement-rust-*`. Exactly one Active class file and one Active domain file remain. `src/AgentCli` and `pyproject.toml` are removed. `src/grok-cli` stays as the POSIX reference.

## Proof

Product proof is `cargo test --offline -- --test-threads=1` (`tests/contract.rs`, `tests/domain.rs`, `tests/startup.rs`, `tests/docs.rs`). The POSIX suite `tests/run.sh` was not re-run in this pass.

## Gaps

- PNG captures still show AgentCli. The README says so and uses a text picture for the live menu.
- Shell `requirement-shell-cli-default-interaction` still describes the POSIX menu (front 3 reserved). The Rust menu is `requirement-rust-tui`. The domain ship-unit note points at that file.
- Opening the real menu can probe `grok` for up to 14 seconds when no session override is set.
