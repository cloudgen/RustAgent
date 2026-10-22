**file**: docs/requirements/requirement-python-oop.md
**Status**: Superseded (Version 1.2.0) by `requirement-rust-modules`
**Area**: python
**Key**: `requirement-python-oop`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

One class in each module. The class that owns a job is the only place that job is written.

### 1.1 Human-facing

**In one sentence:** Menu drawing, grok work, and the about page are three classes, not one long function.

| You | Another role | Not this |
|-----|--------------|----------|
| Change a menu row | Edit `MenuPainter` | A class named `Join` |

## 2. Core Rules

1. `Cli` owns parse and dispatch. `Tui` owns the screen. `MenuPainter` owns rows and the frame. `MenuModel` owns keystrokes. `MenuSession` owns the loop. `MenuLanguage` owns the saved language.
2. `GrokDomain` owns setup, session, backup, sync, crontab, and sudoer JSON.
3. `Host` owns this-login-only detect. `Storage` owns cache tiers. `Output` owns human lines and JSON. `ShellStartup` owns rc files. `SelfManage` owns pip. `SystemLog` owns log files. `AboutPage` and `CheckSystem` own the about page.
4. **MUST NOT** add class `Join`.
5. `def main` constructs `ChronicleLogger`. Other modules **MUST NOT** construct it at import time.

## 3. Acceptance

| AC | Statement |
|----|-----------|
| AC-1 | `GrokDomain` is the only class in `grok_domain.py` |
| AC-2 | There is no `Join` class |
