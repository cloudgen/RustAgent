# RustAgent - Install and run xAI grok, and back up its auth files

[![Version](https://img.shields.io/badge/Version-1.2.0-blue?style=flat-square)](https://github.com/cloudgen/RustAgent)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](LICENSE.md)
[![CIAO](https://img.shields.io/badge/CIAO-Defensive%20Programming-orange?style=flat-square)](https://github.com/cloudgen/ciao)
[![Stars](https://img.shields.io/github/stars/cloudgen/RustAgent?style=flat-square)](https://github.com/cloudgen/RustAgent)
[![Rust](https://img.shields.io/badge/Rust-2021-orange?style=flat-square)](https://www.rust-lang.org/)

RustAgent installs and runs the peer binary `grok`, and it backs up `~/.grok` auth files. Typed verbs do that work in the terminal. The text menu uses the same tokens. `hello` prints one welcome line and does not draw the menu. **Exit** is menu-only.

## Features

- Console binary `rust-agent`. There is no `python -m` entry.
- On a terminal with no arguments, the text menu opens. The first row names the current directory on the left and a local clock (`HH:MM:SS`) on the right when the row has room.
- Multi-user front rows are **1 grok-auth**, **3 system-log**, **5 sudoers**, **6 self-management**, **7 language**, and **9 Exit**. A this-login-only host shows **2 run** and hides **5**. **4** and **8** are not rows.
- **grok-auth** covers session, backup, sync, crontab, setup, and reinstall. **sudoers** covers the backup grant only. **system-log** covers view, clear, and the log folder.
- Thirteen menu languages, codes **80**–**92**. **70**–**79** and **93**–**98** stay unprinted so a later language can be added. The saved leaf is `~/.local/RustAgent/language`. `RUSTAGENT_LANG` overrides one process and does not write the file. `language` is not a typed verb.
- **about** stays in English. It prints the domain line, the runtime tools, a host check, the cache tiers, and the persistence folder.
- Cargo lifecycle: `version-check`, `self-update`, `self-install`, and `self-uninstall`. Those verbs do not use sudo. `self-uninstall` on the command line needs `--force`.
- Empty arguments on a terminal do not run cargo and do not download `grok`. With no terminal and no verb, the program prints help and returns 0.

## Advantages

1. **Two ways to do the work.** A menu number and the same typed verb share one action. `rust-agent hello` prints `Welcome to RustAgent 1.2.0.` and does not draw the menu. With no arguments on a terminal, the text menu opens and waits. With no terminal and no verb, the program prints help and returns 0.
2. **Built-in languages.** Row **7** lists thirteen languages. The numbers are **80** through **92**. **70** through **79** and **93** through **98** stay unprinted so a later language can be added. The choice is saved for the next run. Leaf shorts stay English.
3. **Cache and persistence.** Linux prefers `/dev/shm/cache`, then `/tmp/cache`, then a leaf under the home cache directory. A missed tier is silent. Persistence is `~/.local/RustAgent`. The about page names the tier this process selected.
4. **A narrow root path.** The only elevated action is `rust-agent backup`, and only after an administrator installs a sudoers fragment whose argument is `backup`. Other verbs stay this login. Peer `grok` is fetched by `setup` from the vendor channel, then the public fallback. This package does not run a vendor shell installer.

| Feature | Typed verb | Text menu |
|---------|------------|-----------|
| Daily start on a terminal | `rust-agent menu` | empty argv |
| Welcome line | `rust-agent hello` | not a front row |
| Peer binary | `rust-agent setup` | grok-auth row **15** |
| Auth deposit | `rust-agent backup` | grok-auth row **11** when signed in |
| Menu language | not a verb | row **7**, codes **80**–**92** |
| System log | not a verb | front row **3**, then **31**–**33** |
| This computer | `rust-agent about` | self-management row **63** |
| Cargo lifecycle | `version-check`, `self-update`, `self-install`, `self-uninstall` | rows **64**, **65**, **66**, and **67**. **62** is version. **68** is `where-is-me` |

## Quick Installation

**Rust:** edition 2021. `setup` needs `curl` when it downloads peer `grok`.

```bash
cargo install rustagent
```

This installs the console entry **`rust-agent`**. The crate name is `rustagent`. This tree is **1.2.0**. Nothing in this tree uploads a release by itself.

From a checkout of the public repository [github.com/cloudgen/RustAgent](https://github.com/cloudgen/RustAgent):

```bash
git clone git@github.com:cloudgen/RustAgent.git
cd RustAgent
cargo install --path .
```

HTTPS:

```bash
git clone https://github.com/cloudgen/RustAgent.git
```

After install, on a terminal, `rust-agent` with no arguments prints this screen. The verb is the short token. The note after the colon is the explanation. `9` leaves. `0` on a submenu goes back. A number that is not on the list prints an error and lets you choose again. The path row is the current directory and a local clock (`HH:MM:SS`).

```text
$ rust-agent
Path: <current directory>                                          HH:MM:SS
1. grok-auth      : backup, sync, crontab, setup, and reinstall
3. system-log     : view, clear, and the log folder
5. sudoers        : grant and drafts
6. self-management: version, about, and cargo lifecycle
7. language       : display language for this menu
9. Exit           : leave
╭──────────────╮
│ > █          │
╰──────────────╯
  RustAgent 1.2.0  │  main menu  │  Up/Down  •  Enter
```

## Usage

```bash
rust-agent
```

**hello** prints the welcome line in the terminal and does not draw the menu:

```bash
rust-agent hello
```

```bash
rust-agent help
rust-agent version
rust-agent about
rust-agent hello
rust-agent setup
rust-agent check-session
rust-agent backup
rust-agent where-is-me
rust-agent version-check
rust-agent self-update
rust-agent self-install
rust-agent self-uninstall --force
```

`version` prints `RustAgent 1.2.0` and does not call cargo. `about` shows one English page: the product identity, a host check of this computer, the cache tiers, the persistence folder, and a star box. Basic Usage on that page is `rust-agent`. It does not call cargo. `hello` prints `Welcome to RustAgent 1.2.0.` and does not draw the menu. `help` prints usage.

`setup` downloads peer `grok` for this platform and links `~/.grok/bin/grok`. `backup` copies `auth.*` into `/var/rust-agent` when this login may run that one command as root. `sync-auth` copies the deposit back and skips that copy when grok is already signed in. `add-crontab` stays closed until the backup grant is present.

`version-check` runs `cargo search rustagent --limit 1`. `self-update` runs `cargo install rustagent --force`. `self-install` runs `cargo install rustagent`. `self-uninstall` runs `cargo uninstall rustagent` and needs `--force` on the command line. Those cargo verbs do not use sudo. Empty arguments do not install or update.

`rust-agent install` writes this login's shell startup files so `~/.local/bin` is on PATH. A zsh login gets that line on `.zshenv` and again on `.zprofile`. A bash-only login does not invent those zsh files. `.zshrc` is not a PATH file. `rust-agent rc-test` is test-purpose: it writes only under `--root` and leaves this login's real notes still. `rust-agent uninstall --force` may remove this product's installer stickers and does not delete the files.

## Screenshots

The fenced picture under Quick Installation is the live menu: front row **3** is system-log, and the status line is `RustAgent 1.2.0`. The PNG files below are earlier AgentCli captures. They still show `AgentCli` on the status line. They do not show RustAgent 1.2.0. On those captures the path is `/tmp/agent-cli-shots` and the clock is `12:06:45`. Leaf shorts stay English. **1 grok-auth** is highlighted on each main-menu capture.

### `language-menu.png`

Row **7** has opened the language list. **80 English** is highlighted, with the note "use English for this menu." The other rows are **81 简体中文**, **82 繁體中文**, **83 Español**, **84 العربية**, **85 Français**, **86 Português**, **87 Русский**, **88 Deutsch**, **89 日本語**, **90 한국어**, **91 Nederlands**, and **92 Ελληνικά**. Each note says to use that language for this menu. **0 Back** says "return to the main menu." The path label is `Path`. The status line says `language`.

![Language list, 80 English highlighted](https://raw.githubusercontent.com/cloudgen/agent-cli/main/screenshots/language-menu.png)

### `main-menu-en.png`

English main menu. There is no saved-language line above the box. The path label is `Path`. **1 grok-auth** is "backup, sync, crontab, setup, and reinstall." Then **5 sudoers** "grant and drafts," **6 self-management** "check, update, or remove this package," **7 language** "display language for this menu," and **9 Exit** "leave." The status line says `main menu`.

![English main menu](https://raw.githubusercontent.com/cloudgen/agent-cli/main/screenshots/main-menu-en.png)

### `main-menu-zh-hans.png`

Simplified Chinese main menu. The line above the box says `菜单语言是简体中文`. The path label is `路径`. **1 grok-auth** stays the English token. **5** stays `sudoers`. **6** is `自我管理`, **7** is `语言`, and **9** is `离开`. The status line says `主菜单`.

![Simplified Chinese main menu](https://raw.githubusercontent.com/cloudgen/agent-cli/main/screenshots/main-menu-zh-hans.png)

### `main-menu-zh-hant.png`

Traditional Chinese main menu. The line above the box says `選單語言是繁體中文`. The path label is `路徑`. **5** stays `sudoers`. **6** is `自我管理`, **7** is `語言`, and **9** is `離開`. The status line says `主選單`.

![Traditional Chinese main menu](https://raw.githubusercontent.com/cloudgen/agent-cli/main/screenshots/main-menu-zh-hant.png)

### `main-menu-es.png`

Spanish main menu. The line above the box says `El idioma del menú es español`. The path label is `Ruta`. **6** is `autogestión`, **7** is `idioma`, and **9** is `Salir`. The status line says `menú principal`.

![Spanish main menu](https://raw.githubusercontent.com/cloudgen/agent-cli/main/screenshots/main-menu-es.png)

### `main-menu-ar.png`

Arabic main menu. The line above the box says `لغة القائمة هي العربية`. The path label is `المسار`. The numbers stay on the left. **6** is `إدارة ذاتية`, **7** is `لغة`, and **9** is `خروج`. The status line says `القائمة الرئيسية`.

![Arabic main menu](https://raw.githubusercontent.com/cloudgen/agent-cli/main/screenshots/main-menu-ar.png)

### `main-menu-fr.png`

French main menu. The line above the box says `La langue du menu est le français`. The path label is `Chemin`. **6** is `autogestion`, **7** is `langue`, and **9** is `Quitter`. The status line says `menu principal`.

![French main menu](https://raw.githubusercontent.com/cloudgen/agent-cli/main/screenshots/main-menu-fr.png)

### `main-menu-pt.png`

Portuguese main menu. The line above the box says `O idioma do menu é português`. The path label is `Caminho`. **6** is `autogestão`, **7** is `idioma`, and **9** is `Sair`. The status line says `menu principal`.

![Portuguese main menu](https://raw.githubusercontent.com/cloudgen/agent-cli/main/screenshots/main-menu-pt.png)

### `main-menu-ru.png`

Russian main menu. The line above the box says `Язык меню — русский`. The path label is `Путь`. **6** is `самоуправление`, **7** is `язык`, and **9** is `Выход`. The status line says `главное меню`.

![Russian main menu](https://raw.githubusercontent.com/cloudgen/agent-cli/main/screenshots/main-menu-ru.png)

### `main-menu-de.png`

German main menu. The line above the box says `Die Menüsprache ist Deutsch`. The path label is `Pfad`. **6** is `Selbstverwaltung`, **7** is `Sprache`, and **9** is `Beenden`. The status line says `Hauptmenü`.

![German main menu](https://raw.githubusercontent.com/cloudgen/agent-cli/main/screenshots/main-menu-de.png)

### `main-menu-ja.png`

Japanese main menu. The line above the box says `メニューの言語は日本語`. The path label is `パス`. **6** is `自己管理`, **7** is `言語`, and **9** is `終了`. The status line says `メインメニュー`.

![Japanese main menu](https://raw.githubusercontent.com/cloudgen/agent-cli/main/screenshots/main-menu-ja.png)

### `main-menu-ko.png`

Korean main menu. The line above the box says `메뉴 언어는 한국어`. The path label is `경로`. **6** is `자기관리`, **7** is `언어`, and **9** is `종료`. The status line says `주 메뉴`.

![Korean main menu](https://raw.githubusercontent.com/cloudgen/agent-cli/main/screenshots/main-menu-ko.png)

### `main-menu-nl.png`

Dutch main menu. The line above the box says `De menutaal is Nederlands`. The path label is `Pad`. **6** is `zelfbeheer`, **7** is `taal`, and **9** is `Afsluiten`. The status line says `hoofdmenu`.

![Dutch main menu](https://raw.githubusercontent.com/cloudgen/agent-cli/main/screenshots/main-menu-nl.png)

### `main-menu-el.png`

Greek main menu. The line above the box says `Η γλώσσα του μενού είναι ελληνικά`. The path label is `Διαδρομή`. **6** is `αυτοδιαχείριση`, **7** is `γλώσσα`, and **9** is `Έξοδος`. The status line says `κύριο μενού`.

![Greek main menu](https://raw.githubusercontent.com/cloudgen/agent-cli/main/screenshots/main-menu-el.png)

### `system-log.png`

This AgentCli capture opened system-log from a typed token. **31 view-log** is highlighted: "list a log file and show it." Then **32 clear-log**, **33 log-folder**, and **0 Back**. The live RustAgent front board puts that same board on row **3**. The path label is `Path`. The status line on the capture says `system-log`.

![System log, 31 view-log](https://raw.githubusercontent.com/cloudgen/agent-cli/main/screenshots/system-log.png)

### `self-management.png`

This AgentCli capture opened self-management on pip rows **64**, **65**, **66**, and **68**. The live RustAgent board is cargo rows **62** version, **63** about, **64** version-check, **65** self-update, **66** self-uninstall, **67** self-install, **68** where-is-me, and **0** Back. The path label is `Path`. The status line on the capture says `self-management`.

![Self-management, 64 version-check](https://raw.githubusercontent.com/cloudgen/agent-cli/main/screenshots/self-management.png)

### `grok-auth.png`

**1 grok-auth** on a multi-user host while the session word is missing. The board shows **12 sync-auth-from-remote**, **13 sync-auth**, **14 add-crontab**, **15 setup**, **16 reinstall**, and **0 Back**. A signed-in session shows **11 backup** instead of the sync rows. The status line says `grok-auth`.

![Logged-out grok-auth board](https://raw.githubusercontent.com/cloudgen/agent-cli/main/screenshots/grok-auth.png)

### `sudoers.png`

Row **5** has opened sudoers. The rows are **51 generate-sudoer-request**, **52 submit-sudoer-request**, **53 print-sudoers**, **54 print-sudoers-install-script**, **55 remove-project-sudoers**, and **0 Back**. The grant argument is `backup` only. The status line says `sudoers`.

![Sudoers board, backup grant only](https://raw.githubusercontent.com/cloudgen/agent-cli/main/screenshots/sudoers.png)

### `tui-about.png`

This AgentCli capture is the about result page. It prints `AgentCli 1.2.0` and `Entry points: agent-cli, python -m AgentCli`. The live page prints `RustAgent 1.2.0`, `Domain: Install and run xAI grok, and back up its auth files.`, `Runtime tools: curl, grok`, and entry `rust-agent`. The host check is headed `[CHECK SYSTEM]:` and includes the cache lines. The footer says `Up/Down scrolls this page.` and `Press a key to return to the main menu.` There is no input box and no clock. The page stays in English.

![About page with cache lines](https://raw.githubusercontent.com/cloudgen/agent-cli/main/screenshots/tui-about.png)

## Examples

```bash
rust-agent
rust-agent version
rust-agent hello
rust-agent about
rust-agent setup
```

`version` prints `RustAgent 1.2.0` and does not call cargo. A run that is not the text menu can also print status lines above that. The text menu keeps those lines off the screen.

```bash
rust-agent hello
```

That prints `Welcome to RustAgent 1.2.0.` and does not draw the menu.

## Platform Compatibility

Linux is the primary host. macOS and Windows work when Rust and cargo are present. `setup` selects `linux`, `macos`, or `windows`, and `x86_64` or `aarch64`. The text menu needs a terminal. Git Bash and Windows cmd are this-login-only: they show **run** and hide **sudoers**. Termux is detected from `TERMUX_VERSION` or a set `PREFIX`, unless `RUSTAGENT_HOST_OVERRIDE` is `multiuser`, `gitbash`, or `windows-cmd`.

## Related Projects

- [RustAgent on GitHub](https://github.com/cloudgen/RustAgent) — this program’s source
- [grok-cli](https://github.com/cloudgen/grok-cli) — POSIX reference this crate specializes beside
- [CIAO](https://github.com/cloudgen/ciao) — Caution, Intentional, Anti-fragile, Over-engineered
- [CIAO-Lite](https://github.com/cloudgen/ciao-lite) — short agent contract
- [safe-rm](https://github.com/cloudgen/safe-rm) — guarded `rm`

## Contributing

1. Keep product law in `docs/requirements/` aligned with this document when behavior changes.
2. Do not add `join`, `list-videos`, or an encoder. Those verbs stay unknown.
3. Version dual SSOT: bump **`Cargo.toml`** `version` and **`src/identity.rs`** `VERSION` together.
4. Do not widen the root path past `rust-agent backup`.
5. Empty argv on a terminal opens the menu. Empty argv off a terminal prints help and returns 0.

## License

MIT. See [LICENSE.md](LICENSE.md).

Copyright (c) 2026 cloudgen.

## Last Update

2026-10-22 — **1.2.0**. The ship unit is the Rust crate `rustagent`, binary `rust-agent`. Install is cargo. Front rows are **1 grok-auth**, **3 system-log**, **5 sudoers**, **6 self-management** (**62**–**68**), **7 language**, and **9 Exit**. Language codes are **80**–**92**. The leaf is `~/.local/RustAgent/language`. `RUSTAGENT_LANG` overrides one process. The text picture above is the live menu. PNG files under Screenshots are earlier AgentCli captures. `submit-sudoer-request` queues a backup-only grant and does not write `/etc`.
