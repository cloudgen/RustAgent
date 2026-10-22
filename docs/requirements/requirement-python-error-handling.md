**file**: docs/requirements/requirement-python-error-handling.md
**Status**: Superseded (Version 1.2.0) by `requirement-rust-error-handling`
**Area**: python
**Key**: `requirement-python-error-handling`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

Errors say what happened and the next command. Source auth files are not deleted by a failed sync.

### 1.1 Human-facing

**In one sentence:** A failed backup or sync tells you the next command and leaves the other copy where it was.

| You | Another role | Not this |
|-----|--------------|----------|
| Read `ERROR:` and `Next:` | `Output.error` writes both | A silent `pass` on a failed deposit |

## 2. Core Rules

1. Human errors **MUST** go to stderr as `ERROR:` plus `Next:`.
2. `--json` errors **MUST** be one JSON object on stderr with `type` `out_error`.
3. Missing grok on `run` or `update-grok` **MUST** name `agent-cli setup`.
4. `sync-auth` **MUST** skip when the session is valid or timeout, and **MUST** say `No sync-auth for logged-in environment.`
5. `sync-auth-to-remote` **MUST** still run when the session is valid.
6. A bad remote SPEC **MUST** be refused. Accepted forms are `user@IPv4`, IPv4, domain, and `user@domain`.
7. `reinstall` **MUST NOT** delete `auth.*`.
8. Unknown verbs **MUST** be non-zero. `restore` is unknown.

## 3. Acceptance

| AC | Statement |
|----|-----------|
| AC-1 | Logged-in `sync-auth` skips |
| AC-2 | `restore` is unknown |
| AC-3 | A bad SPEC is refused |
