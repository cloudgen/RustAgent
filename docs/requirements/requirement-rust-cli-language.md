**file**: docs/requirements/requirement-rust-cli-language.md
**Status**: Active (Version 1.2.0)
**Area**: rust
**Key**: `requirement-rust-cli-language`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

Menu language for RustAgent. Thirteen languages. Front row **7**. Codes **80**–**92**. HolaTui’s codes **71**–**83** are not this product’s numbers.

### 1.1 Human-facing

**In one sentence:** Row **7** picks the menu language and saves it for the next run.

| You | Another role | Not this |
|-----|--------------|----------|
| Pick **80** English through **92** Greek | Leaf tokens such as `grok-auth` stay English | An argv verb named `language`, or reading `LANG` |

## 2. Core Rules

1. Codes **MUST** be `en` 80, `zh-Hans` 81, `zh-Hant` 82, `es` 83, `ar` 84, `fr` 85, `pt` 86, `ru` 87, `de` 88, `ja` 89, `ko` 90, `nl` 91, `el` 92.
2. **70**–**79** and **93**–**98** **MUST NOT** be printed. Typing a reserved number **MUST NOT** write the leaf.
3. The saved leaf **MUST** be `${HOME}/.local/RustAgent/language`. The directory **MUST** be mode `0700`. The file **MUST** be mode `0600`. The program reads the first line only.
4. `RUSTAGENT_LANG` **MUST** override one process when it is one of the thirteen codes and **MUST NOT** write the file. `LANG`, `LANGUAGE`, `LC_ALL`, and `LC_MESSAGES` **MUST NOT** select a code.
5. **0** Back and Esc **MUST NOT** write. A failed write **MUST** keep the previous code and **MUST** still return to the front board.
6. The default code **MUST** be `en` when the leaf is missing or the first line is not a known code. An invalid first line **MUST NOT** be rewritten.
7. Chrome may follow the code: path label, board titles, category shorts, Back, Exit, and the saved-language line. Domain leaf shorts stay English, including `grok-auth`, `sudoers`, `run`, and the auth and sudoer tokens.
8. The welcome line, cargo output, help, and the about page stay English. The key hint `Up/Down  •  Enter` stays English.
9. `language` is not an argv verb.

## 3. Acceptance

| AC | Statement |
|----|-----------|
| AC-1 | Row 7 lists 80–92 and not 79 or 93 |
| AC-2 | Saving `82` writes `zh-Hant` at mode 0600 and Back does not write |
| AC-3 | After that save, `grok-auth` stays English and the self-management explain names cargo in Traditional Chinese |
| AC-4 | `RUSTAGENT_LANG` wins and does not write |
