**file**: docs/requirements/requirement-rust-error-handling.md
**Status**: Active (Version 1.2.0)
**Area**: rust
**Key**: `requirement-rust-error-handling`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

Errors say what happened and the next command. A failed sync does not delete the other copy. Source auth files are not deleted by reinstall.

### 1.1 Human-facing

**In one sentence:** A failed backup or sync tells you the next command and leaves the other copy where it was.

| You | Another role | Not this |
|-----|--------------|----------|
| Read `ERROR:` and `Next:` | `--json` writes one `out_error` object | A silent success after a failed deposit |

## 2. Core Rules

1. Human errors **MUST** go to stderr as `ERROR:` plus `   Next:` (three spaces before `Next:`).
2. `--json` errors **MUST** be one JSON object with `type` `out_error`.
3. Missing grok on `run` or `update-grok` **MUST** name `rust-agent setup`.
4. `sync-auth` **MUST** skip when the session is valid or timeout, and **MUST** say `No sync-auth for logged-in environment.` `sync-auth-from-remote` skips when logged in. `sync-auth-to-remote` **MUST** still run when the session is valid.
5. A bad remote SPEC **MUST** be refused. Accepted forms are `user@IPv4`, IPv4, domain, and `user@domain`. Refused examples include `ada@`, `-host`, and `a b@host`.
6. `reinstall` **MUST NOT** delete `auth.*`.
7. Unknown verbs **MUST** be non-zero. `restore`, `join`, and `list-videos` are unknown.
8. `self-uninstall` without `--force` **MUST** return non-zero and **MUST NOT** run cargo. `--force` on a verb outside `self-uninstall`, `uninstall`, `setup`, `reinstall`, and `remove-project-sudoers` **MUST** say `--force is only for self-uninstall.`
9. A too-small text screen **MUST** say `The text screen is too small for the menu and the input box.` and **MUST** name `rust-agent help`. Control-C on the text screen **MUST** return 130 and **MUST NOT** print `ERROR:`. Menu row 9 **MUST** return 0.
10. `remove-project-sudoers` **MUST** require `--force`, **MUST** delete only the user draft, and **MUST** refuse a path under `/etc`.

## 3. Acceptance

| AC | Statement |
|----|-----------|
| AC-1 | Logged-in `sync-auth` skips |
| AC-2 | `restore` is unknown |
| AC-3 | A bad SPEC is refused |
| AC-4 | Bare `self-uninstall` does not run cargo |
