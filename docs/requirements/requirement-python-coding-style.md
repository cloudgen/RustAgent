**file**: docs/requirements/requirement-python-coding-style.md
**Status**: Superseded (Version 1.2.0) by `requirement-rust-coding-style`
**Area**: python
**Key**: `requirement-python-coding-style`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

Python style for AgentCli. Fail closed. Do not hide an error.

### 1.1 Human-facing

**In one sentence:** When a step cannot finish, the program says what happened and the next command.

| You | Another role | Not this |
|-----|--------------|----------|
| Read an ERROR line and a Next line | Domain procedures stay in their requirement files | A media publish path |

## 2. Core Rules

1. Expected failures **MUST** return a non-zero status and an operator-readable message with a next step.
2. **MUST NOT** claim success after a failed copy, download, or crontab install.
3. Scratch files **MUST** go through `Storage.mktemp`. A `$$` template **MUST** be refused.
4. A missed cache tier **MUST** be silent. The next tier is tried.
5. **MUST NOT** pipe a downloaded script into a shell. `setup` **MUST NOT** execute a vendor `install.sh`.
6. Source **MUST NOT** contain unfinished markers (`TODO`, `FIXME`, `NotImplementedError` as delivery).
7. Product paths come from `Storage`, `Host`, and `GrokDomain` attributes. Do not scatter a second deposit path.

## 3. Acceptance

| AC | Statement |
|----|-----------|
| AC-1 | `grok_domain.py` does not contain `install.sh` |
| AC-2 | `mktemp` refuses a `$$` template |
