**file**: docs/requirements/requirement-python-project-structure.md
**Status**: Superseded (Version 1.2.0) by `requirement-rust-project-structure`
**Area**: python
**Key**: `requirement-python-project-structure`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

The Python package lives under `src/AgentCli/`. One class in each module.

### 1.1 Human-facing

**In one sentence:** The program you run is the `AgentCli` package, not a second copy of HelloTui.

| You | Another role | Not this |
|-----|--------------|----------|
| Import `AgentCli` | `src/grok-cli` is the shell reference | A `join.py` module |

## 2. Core Rules

1. The package **MUST** contain `__init__.py`, `__main__.py`, `cli.py`, `tui.py`, `menu_painter.py`, `menu_model.py`, `menu_session.py`, `menu_language.py`, `system_log.py`, `self_management.py`, `about_page.py`, `check_system.py`, `grok_domain.py`, `host.py`, `output.py`, `storage.py`, and `shell_startup.py`.
2. **MUST NOT** add `join.py` or an encoder module.
3. `__main__.py` **MUST** call `cli.main`.
4. `src/grok-cli` may remain as the POSIX behavior reference. It is not the pip ship unit.

## 3. Acceptance

| AC | Statement |
|----|-----------|
| AC-1 | `python -m AgentCli` reaches `cli.main` |
| AC-2 | No `join.py` is in the package |
