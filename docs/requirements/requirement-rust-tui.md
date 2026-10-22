**file**: docs/requirements/requirement-rust-tui.md
**Status**: Active (Version 1.2.0)
**Area**: rust
**Key**: `requirement-rust-tui`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

This file is the picture of RustAgent’s text menu. The frame is the HolaTui text menu: path on the left, local clock on the right, a three-row rounded input box, and a status line that names the product and the version. The numbered rows are the RustAgent domain menu plus the HolaTui system-log row and the HolaTui cargo self-management rows.

Typed verbs stay on `requirement-rust-cli-interface`. Domain procedures stay on `requirement-domain-grok-cli`. Language codes stay on `requirement-rust-cli-language`. The POSIX menu in `src/grok-cli` is not this picture.

### 1.1 Human-facing

**In one sentence:** On a terminal, `rust-agent` with no arguments shows the grok menu inside the HolaTui frame, and choosing a listed row runs that verb.

| You | Another role | Not this |
|-----|--------------|----------|
| Pick **1** grok-auth, **3** system-log, or **9** Exit | The same verb typed on the command line | A `Choice:` line, a media join, or renumbering to close a gap |

## 2. Core Rules

1. The session **MUST** be `Tui` in `src/tui.rs`. The frame **MUST** be `MenuPainter` in `src/painter.rs`. `src/cli.rs` **MUST NOT** own the frame glyphs. The product **MUST NOT** import a second menu crate and **MUST NOT** draw a `Choice:` line.
2. Multi-user front rows **MUST** be **1** grok-auth (`backup, sync, crontab, setup, and reinstall`), **3** system-log, **5** sudoers (`grant and drafts`), **6** self-management, **7** language, and **9** Exit. There is no Back row on the front board. Rows **4** and **8** are not rows. **MUST NOT** renumber to close those gaps. `hello` is a typed verb and an any-board token. It is not numbered row 1.
3. This-login-only (Termux, Git Bash, Windows cmd) **MUST** also show **2** run (`start grok without auto-update`) and **MUST** hide **5** sudoers. `TERMUX_VERSION` counts as Termux. A set `PREFIX` counts as Termux unless `RUSTAGENT_HOST_OVERRIDE` is `multiuser`, `gitbash`, or `windows-cmd`. Override `multiuser` wins over Termux and `PREFIX`. Android from `/proc/sys/kernel/osrelease` counts as this-login-only.
4. grok-auth rows **MUST** keep fixed numbers: **11** backup when signed in and not this-login-only; **12** sync-auth-from-remote when logged out; **13** sync-auth when logged out and not this-login-only; **14** add-crontab; **15** setup; **16** reinstall; **0** Back. `sync-auth-to-remote` is typed and is not a row. A probe timeout counts as signed in.
5. self-management rows **MUST** be **62** version, **63** about, **64** version-check, **65** self-update, **66** self-uninstall, **67** self-install, **68** where-is-me, and **0** Back. Row **61** is not a row. The long text names cargo, not pip. Traditional Chinese self long is `版本、關於，以及 cargo 生命週期`.
6. sudoers rows **MUST** be **51** generate-sudoer-request, **52** submit-sudoer-request, **53** print-sudoers, **54** print-sudoers-install-script, **55** remove-project-sudoers, and **0** Back.
7. system-log rows **MUST** be **31** view-log, **32** clear-log, **33** log-folder, and **0** Back. Choosing a row runs that verb and shows a result page. Esc on a nested board returns to the front board and is not row **9** Exit.
8. Cargo rows **MUST** run `cargo search rustagent --limit 1`, `cargo install rustagent --force`, `cargo uninstall rustagent`, and `cargo install rustagent`. They **MUST NOT** use sudo or pip. Choosing **66** is the menu confirmation. Command-line `self-uninstall` still needs `--force` (`requirement-rust-cli-interface`).
9. The first row **MUST** be `<label>: <absolute cwd>` with the clock `HH:MM:SS` on the right when the row has room. English label is `Path`. Traditional Chinese label is `路徑`. The path and the clock are not translated. The one-second clock wait applies on front, self, log, lang, auth, and sudo. It **MUST NOT** be read as Esc. Result pages, the view-log list, the clear confirm, the remote SPEC prompt, and the setup keep/reinstall prompt use a blocking read.
10. The bottom **MUST** be a three-row box using `╭` `╮` `╰` `╯` `─` `│` and caret `█`, then the unavailable lines, then the status line `  RustAgent <version>  │  <title>  │  Up/Down  •  Enter`. ASCII `+` `-` `|` and sharp corners **MUST NOT** stand in for the arcs.
11. Unavailable lines **MUST** be: host `backup, sync-auth and sudoers features are not available in {label}.`; logged-out `backup features are not available for logged-out environment.`; logged-in `sync-auth and sync-auth-from-remote features are not available for logged-in environment.`
12. Domain leaf shorts stay English. Chrome shorts (language, system-log, self-management, Exit, Back) may follow the selected language.
13. A window that cannot hold the path line, one numbered row, the three-row box, the unavailable lines, and the status line **MUST** fail closed with `The text screen is too small for the menu and the input box.` and **MUST NOT** fall back to a `Choice:` line. Control-C restores the terminal and returns 130.
14. Default `MenuPainter::new` is multi-user and logged out, with no unavailable lines, so an empty notice list still fits the HolaTui chrome until the host and session are bound.

## 3. Acceptance

| AC | Statement |
|----|-----------|
| AC-1 | Front numbers on the default painter are 1, 3, 5, 6, 7, 9 and row 1 is grok-auth |
| AC-2 | Self numbers are 62, 63, 64, 65, 66, 67, 68, 0 |
| AC-3 | Language row 7 uses codes 80–92 |
| AC-4 | The input box uses the arc glyphs and the status line names RustAgent |
