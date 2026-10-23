**file**: docs/requirements/requirement-rust-about.md
**Status**: Active (Version 1.2.1)
**Area**: rust
**Key**: `requirement-rust-about`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

The about page names this build, this computer, the cache tiers, and the persistence folder. It stays English. The screen that draws it stays on `requirement-rust-tui`. The domain sentence stays on `requirement-domain-grok-cli`.

### 1.1 Human-facing

**In one sentence:** `rust-agent about` shows one English page and does not call cargo or pip.

| You | Another role | Not this |
|-----|--------------|----------|
| Read the host check and the star box | Domain sentence from `requirement-domain-grok-cli` | A live FFmpeg probe, `py-tui`, or a curl installer line |

## 2. Core Rules

1. `AboutPage` composes the page. `CheckSystem` owns the host-check reads.
2. The identity block **MUST** include `RustAgent 1.2.1`, `Domain: Install and run xAI grok, and back up its auth files.`, and `Runtime tools: curl, grok`. The entry **MUST** be `rust-agent`. It **MUST NOT** say `python -m`.
3. The page **MUST** name cache folder used, preferred, 1st fallback, 2nd fallback when present, and persistence storage `${HOME}/.local/RustAgent`. Cache lines come from `Storage` built from the process environment: preferred `/dev/shm/cache/cache-rust-agent-<login>-<pid>` (Git Bash and macOS prefer `/tmp/cache`), fallback `/tmp/cache`, second fallback `~/.cache/cache-rust-agent-<pid>`.
4. The star box **MUST** be one rectangle. Basic Usage **MUST** be `rust-agent`, not `rust-agent hello`.
5. The page **MUST NOT** print `curl -fsSL`, `py-tui`, `ffmpeg is on PATH`, or `python -m pip install`. `DOWNLOAD_URL` stays empty, so the live page **MUST NOT** contain a curl installer.
6. Install kind **MUST** be uninstalled for a source checkout that has `Cargo.toml`, local for a home copy or `~/.cargo/bin`, and global for `/usr`, `/opt`, or `/bin`.
7. JSON about, when `--json` is set, **MUST** use `type` `about`, the domain sentence, `name` `RustAgent`, and `version` `1.2.1`.
8. Building the page **MUST NOT** call sudo, install a package, or open a socket for a version lookup.

## 3. Acceptance

| AC | Statement |
|----|-----------|
| AC-1 | Identity includes the domain sentence and `Runtime tools: curl, grok` |
| AC-2 | Basic Usage is `rust-agent` |
| AC-3 | The page has no curl installer line and no `py-tui` |
