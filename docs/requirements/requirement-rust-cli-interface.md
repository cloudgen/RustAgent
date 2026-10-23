**file**: docs/requirements/requirement-rust-cli-interface.md
**Status**: Active (Version 1.2.1)
**Area**: rust
**Key**: `requirement-rust-cli-interface`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

This file owns the typed verbs and the decision to open the text menu. The menu picture is `requirement-rust-tui`. Domain procedures are `requirement-domain-grok-cli`. The about page is `requirement-rust-about`. The logger construct is `requirement-rust-cli-logging`.

### 1.1 Human-facing

**In one sentence:** `rust-agent` with no words opens the text menu on a terminal, and with no terminal it prints help and returns 0.

| You | Another role | Not this |
|-----|--------------|----------|
| Type `rust-agent`, `rust-agent hello`, or a domain verb | The menu runs the same verb | `join`, `list-videos`, `restore`, `python -m`, or a vendor `install.sh` |

## 2. Core Rules

1. The program name **MUST** be `rust-agent`. There is no `python -m` entry.
2. Product verbs **MUST** be `help`, `version`, `about`, `hello`, `menu`, `main`, `ensure`, `install`, `uninstall`, `rc-test`, `where-is-me`, `self-install`, `version-check`, `self-update`, `self-uninstall`, `setup`, `update-grok`, `reinstall`, `run`, `check-session`, `backup`, `sync-auth`, `sync-auth-from-remote`, `sync-auth-to-remote`, `add-crontab`, `print-sudoers`, `print-sudoers-install-script`, `remove-project-sudoers`, `generate-sudoer-request`, and `submit-sudoer-request`. `language`, `system-log`, and `Exit` are menu-only. **MUST NOT** add `join`, `list-videos`, or `restore`.
3. On a terminal, empty argv **MUST** open the front board, return 0 when the person chooses Exit, and **MUST NOT** run cargo and **MUST NOT** download grok. With no terminal and no verb, the program **MUST** print help and return 0.
4. `--json` with no verb, and `help` or `--help` when `--json` is set, **MUST** print JSON help (`type` `help`, `message` `rust-agent help`, `verbs` the product list) and **MUST NOT** open the menu.
5. `hello` **MUST** print `Welcome to RustAgent 1.2.1.` and **MUST NOT** draw the menu.
6. `version` and `--version` **MUST** print `RustAgent 1.2.1` and **MUST NOT** call cargo and **MUST NOT** draw the menu.
7. On a terminal, `about`, `menu`, and `main` **MUST** open the text screen. Off a terminal, `about` **MUST** print the page and return 0. JSON about **MUST** use `type` `about`, the domain sentence as `message`, `name` `RustAgent`, and `version` `1.2.1`.
8. `version-check` runs `cargo search rustagent --limit 1`. `self-update` runs `cargo install rustagent --force`. `self-install` runs `cargo install rustagent`. `self-uninstall` runs `cargo uninstall rustagent` and on the command line **MUST** require `--force`. These verbs **MUST NOT** use sudo or pip.
9. `--force` is accepted on `self-uninstall`, `uninstall`, `setup`, `reinstall`, and `remove-project-sudoers`. On any other verb the program **MUST** return non-zero with `--force is only for self-uninstall.` and **MUST NOT** run the verb.
10. `run` **MUST** refuse `--json`. It **MUST** pass `--no-auto-update` unless the operator already passed that flag.
11. `update-grok` **MUST** fail closed when grok is not installed, and the message **MUST** contain `not installed` with next step `rust-agent setup`.
12. An unknown verb **MUST** return non-zero and name the token. `rc-test` accepts `--root`, `--file`, `--case`, and `--feature`. Other extra operands stay on the domain verb.
13. The screen opens only when stdout is a terminal and argv is empty or the verb is `about`, `menu`, or `main`. Help, version, `--json`, and a parse error do not open it.
14. Domain behavior for the grok verbs stays on `requirement-domain-grok-cli`, `requirement-grok-setup`, `requirement-grok-auth-backup`, and `requirement-grok-crontab`.

## 3. Acceptance

| AC | Statement |
|----|-----------|
| AC-1 | Empty argv on a terminal opens the menu and does not run cargo |
| AC-2 | Empty argv with no terminal prints help and returns 0 |
| AC-3 | `hello` prints the welcome line and does not draw the menu |
| AC-4 | `join`, `list-videos`, and `restore` are unknown |
| AC-5 | `--json` with no verb prints JSON help and does not open the menu |
