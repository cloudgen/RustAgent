**file**: docs/requirements/requirement-python-cli-interface.md
**Status**: Superseded (Version 1.2.0) by `requirement-rust-cli-interface`
**Area**: python
**Key**: `requirement-python-cli-interface`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

Define the command-line entry, empty-argv behavior, and typed verbs for the AgentCli Python package.

The text-menu picture is `requirement-python-tui`. Domain verbs are `requirement-domain-grok-cli`. The about page is `requirement-python-about`. The logger construct is `requirement-python-cli-logging`.

### 1.1 Human-facing

**In one sentence:** `agent-cli` with no words opens the text menu on a terminal, and with no terminal it prints help and returns 0.

| You | Another role | Not this |
|-----|--------------|----------|
| Type `agent-cli`, `agent-cli hello`, or a domain verb | The menu draws the same verbs | `join`, `list-videos`, `restore`, or a vendor `install.sh` |

**Includes:** console script, module entry, typed verbs, empty-argv menu or help.  
**Excludes:** a second shell installer as the package install story.

| You do… | What it means | What you type |
|---------|---------------|---------------|
| Start on a terminal | The front board opens. Pip does not run. grok is not downloaded | `agent-cli` |
| Start with no terminal | Help prints and the process returns 0 | `agent-cli` |
| See the welcome line | One line, no menu | `agent-cli hello` |
| Do the domain work | The same verb the menu runs | `agent-cli setup` |

## 2. Core Rules

1. Console script **MUST** be `agent-cli`. Module entry **MUST** be `python -m AgentCli`. Both call `cli.main`.
2. `def main` **MUST** construct `ChronicleLogger(...)` before `Cli`.
3. On a terminal, empty argv **MUST** open the front board and **MUST NOT** run pip and **MUST NOT** download grok.
4. With no terminal and no verb, the program **MUST** print help and return 0.
5. `--json` with no verb **MUST** print JSON help and **MUST NOT** open the menu.
6. Typed verbs **MUST** be: `help`, `version`, `about`, `hello`, `menu`, `main`, `ensure`, `install`, `uninstall`, `rc-test`, `where-is-me`, `self-install`, `version-check`, `self-update`, `self-uninstall`, `setup`, `update-grok`, `reinstall`, `run`, `check-session`, `backup`, `sync-auth`, `sync-auth-from-remote`, `sync-auth-to-remote`, `add-crontab`, `print-sudoers`, `print-sudoers-install-script`, `remove-project-sudoers`, `generate-sudoer-request`, `submit-sudoer-request`.
7. `language` and `Exit` are menu-only. **MUST NOT** add `join`, `list-videos`, or `restore`.
8. `hello` **MUST** print `Welcome to AgentCli <version>.` and **MUST NOT** draw the menu.
9. `self-uninstall` on the command line **MUST** require `--force`.
10. `run` **MUST** refuse `--json`. It **MUST** pass `--no-auto-update` unless the operator already passed that flag.
11. `update-grok` **MUST** fail closed when grok is not installed, with next step `agent-cli setup`.
12. Unknown verbs **MUST** exit non-zero and name the unknown token.
13. Domain behavior for the grok verbs stays on `requirement-domain-grok-cli`, `requirement-grok-setup`, `requirement-grok-auth-backup`, and `requirement-grok-crontab`. This file names the entry. It does not restate those procedures.

## 3. Acceptance

| AC | Statement |
|----|-----------|
| AC-1 | Empty argv on a terminal opens the menu and does not run pip |
| AC-2 | Empty argv with no terminal prints help and returns 0 |
| AC-3 | `hello` prints the welcome line and does not draw the menu |
| AC-4 | `join`, `list-videos`, and `restore` are unknown |
| AC-5 | `run` inserts `--no-auto-update` |
