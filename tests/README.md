# Tests — RustAgent

## Product proof

```sh
cargo test --offline -- --test-threads=1
```

Exit **0** when the Rust suite passes. Ship unit: crate **rustagent** **1.2.0**, console `rust-agent`.

| File | Focus | Law |
|------|--------|-----|
| `contract.rs` | Front rows, language 80–92, cargo self rows, about identity | `requirement-rust-tui`, `requirement-rust-cli-language`, `requirement-rust-about` |
| `domain.rs` | Session, backup, sync, setup, crontab, sudoer handoff, `run` | `requirement-domain-grok-cli`, `requirement-rust-error-handling` |
| `startup.rs` | `.zprofile` / `.zshenv`, rc-test | `requirement-shell-startup-files` |
| `docs.rs` | README badge, headings, clock, absolute screenshot URLs | `requirement-rust-readme` |

## POSIX reference

```sh
./tests/run.sh
```

`src/grok-cli` is the POSIX behavior reference. `tests/run.sh` is not the product proof. Exit **0** when that reference suite passes; **1** on failure; **2** if `src/grok-cli` is missing.

The Python package `src/AgentCli` is removed. Python unit tests that imported it are removed with it.
