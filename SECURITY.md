# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 1.2.0 (current Rust crate `rustagent`) | Yes |

Older numbered sections in `CHANGELOG.md` record the POSIX `grok-cli` history that this git tree still contains. They are not RustAgent releases. Report defects in `rust-agent` **1.2.0** against this repository.

## Reporting a Vulnerability

Please **do not** open a public issue for security-sensitive reports when a private channel is available.

**Maintainer contact (email):** `wongcf22@gmail.com`

- Source of contact: product **author-email** SSOT in [`LICENSE.md`](./LICENSE.md) (Copyright line).  
- Prefer email (or private GitHub security advisories when enabled) for vulnerability details, reproduction steps, and impact.  
- Do not include exploit weaponization guides in public channels.

## Security Design Principles (CIAO)

This project follows **[CIAO](https://github.com/cloudgen/ciao)** / **[CIAO-Lite](https://github.com/cloudgen/ciao-lite)** defensive design. Security-relevant intent:

| Letter | Principle | Security application |
|--------|-----------|----------------------|
| **C** | **Caution** | Session check uses peer `grok`. `backup` deposits `auth.*` as root into `/var/rust-agent` (files mode `0644`). Never print tokens or grok’s prompt answer. |
| **I** | **Intentional** | Session check and `sync-auth` stay the invoking login. `print-sudoers` and `submit-sudoer-request` do not write `/etc`. The queued grant is backup only. |
| **A** | **Anti-fragile** | `RUSTAGENT_*` overrides keep one process off the default deposit and peer paths. Cargo install, update, and uninstall do not use sudo. |
| **O** | **Over-protect** | The elevated command is `rust-agent backup`. `self-uninstall` on the command line requires `--force`. OS tools are not the sudoers grant. |

Full principles: [CIAO](https://github.com/cloudgen/ciao) · [CIAO-Lite](https://github.com/cloudgen/ciao-lite).

This section is **design posture**, not a third-party certification claim.

## Scope notes

- This crate installs with cargo (`cargo install rustagent` or `cargo install --path .`). The product download URL is empty. There is no `curl | sh` installer for `rust-agent`.
- **`setup`** uses `curl` as the invoking login to fetch xAI’s published version pointer and `grok` binary under `https://x.ai/cli`. It does not download or execute a vendor `install.sh`, and it does not grant extra sudoers. Treat that vendor binary as third-party code.
- Elevation for this product is `sudo -n /usr/local/bin/rust-agent backup`. The command copies `~/.grok/auth.*` into `/var/rust-agent` and sets those files to mode `0644`. Anyone who can read the host can use those grok credentials. That shared store is intentional and is not the same as mode `0600` in a private home.
- `sync-auth` copies the deposit back into this login’s `~/.grok` and sets `auth.json` to mode `0600`. It does not use sudo. It skips the copy when grok is already signed in.
- `sync-auth-to-remote` copies this login’s `~/.grok/auth.*` to the remote SSH login’s `~/.grok` with `ssh` and `scp` in BatchMode, then sets remote `auth.json` to mode `0600`. It does not write `/var/rust-agent`.
- `submit-sudoer-request` queues a backup-only grant. It does not write `/etc`. An administrator still installs the fragment. The installed path shape is `/etc/sudoers.d/rust-agent-<user>`. Drafts live under `~/.config/rust-agent`.
- `self-uninstall` on the command line requires `--force`. Cargo uninstall does not use sudo.
- Related docs: [`README.md`](./README.md), [`LICENSE.md`](./LICENSE.md), `docs/requirements/requirement-three-layer-privilege-model.md`, `docs/requirements/requirement-grok-auth-backup.md`, `docs/requirements/requirement-rust-packaging.md`.
