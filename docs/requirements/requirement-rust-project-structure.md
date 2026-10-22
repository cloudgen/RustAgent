**file**: docs/requirements/requirement-rust-project-structure.md
**Status**: Active (Version 1.2.0)
**Area**: rust
**Key**: `requirement-rust-project-structure`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

Repository layout for RustAgent. Module jobs stay on `requirement-rust-modules`. Package fields stay on `requirement-rust-packaging`.

### 1.1 Human-facing

**In one sentence:** The program you run is the `rustagent` crate under `src/*.rs`, and `src/grok-cli` is only the POSIX reference.

| You | Another role | Not this |
|-----|--------------|----------|
| Build with cargo | `src/grok-cli` stays as behavior reference | A second ship unit under `src/AgentCli/` |

## 2. Core Rules

1. The repository **MUST** contain `Cargo.toml`, `Cargo.lock`, `src/main.rs`, `src/lib.rs`, and the modules named by `requirement-rust-modules`.
2. Product proof **MUST** be `cargo test --offline -- --test-threads=1`, including `tests/contract.rs`, `tests/domain.rs`, `tests/startup.rs`, and `tests/docs.rs`.
3. `src/grok-cli` **MAY** remain. `tests/run.sh` **MAY** remain as the POSIX reference suite. Neither is the product proof.
4. After the Rust tests pass, `src/AgentCli/`, `pyproject.toml`, and Python unit tests that import `AgentCli` **MUST** be absent. Shell reference scripts may remain.
5. `docs/requirements/index.md` **MUST** list every live `requirement-*.md`. Exactly one Active class file and exactly one Active domain file.
6. Root `README.md` **MUST** name RustAgent and **MUST NOT** tell the reader to `pip install` this product.
7. `target/` **MUST** be gitignored.
8. Agent-procedure, mold, glossary, policy, and human-intro folders outside `docs/requirements/` **MUST NOT** be rewritten to freeze RustAgent product facts.

## 3. Acceptance

| AC | Statement |
|----|-----------|
| AC-1 | `src/main.rs` and `src/lib.rs` exist |
| AC-2 | `tests/contract.rs` exists |
| AC-3 | `src/AgentCli/` and `pyproject.toml` are absent |
| AC-4 | `src/grok-cli` is still present |
