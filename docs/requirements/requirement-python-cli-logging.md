**file**: docs/requirements/requirement-python-cli-logging.md
**Status**: Superseded (Version 1.2.0) by `requirement-rust-cli-logging`
**Area**: python
**Key**: `requirement-python-cli-logging`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

One ChronicleLogger instance, constructed in `def main`.

### 1.1 Human-facing

**In one sentence:** The program writes its log through ChronicleLogger, and the text menu does not print that log on the console.

| You | Another role | Not this |
|-----|--------------|----------|
| Run `agent-cli` | `main` passes `is_quiet=True` when the screen will open | A second logger constructed at import time |

## 2. Core Rules

1. `def main` **MUST** contain the text `ChronicleLogger(`.
2. When the run opens the text screen, `is_quiet` **MUST** be true so directory-created lines stay off the console.
3. Other modules **MUST NOT** construct ChronicleLogger at import.
4. A missing ChronicleLogger **MUST** exit 1 and name `python -m pip install "ChronicleLogger>=1.3.1"`.
5. system-log views, clears, and shows that log folder. It does not invent a second log library.

## 3. Acceptance

| AC | Statement |
|----|-----------|
| AC-1 | `cli.py` `main` writes `ChronicleLogger(` |
| AC-2 | Import of `cli` does not require the constructor to have run |
