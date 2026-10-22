**file**: docs/requirements/requirement-bootstrap-chain.md  
**Status**: Active (Version 4.0.0)  
**Area**: architecture  
**Key**: `requirement-bootstrap-chain`  
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

Declare the **bootstrap chain** for this product: ordered lineage, direction, architecture inheritance, and the **domain retarget** from folder-archive backup onto grok auth backup/sync.

Text-menu architecture for the Rust ship unit comes from sibling **HolaTui**. Direction is HolaTui → RustAgent only. Do not edit HolaTui. The domain parent remains this tree’s grok-cli law. The ship unit is the crate `rustagent`, not the Python package AgentCli.

**Direction is sacred:** ancestor → descendant only. Never reverse-copy this product onto the bootstrap parent.

---

### 1.1 Human-facing

This file says grok-cli grew from cli-template through sibling folder-backup. Do not copy grok-cli back onto those parents.

| You | Another role | Not this |
|-----|--------------|----------|
| Read the hop table so you know where architecture came from | Parent products stay their own workspaces | Reverse-copy; wiping grok-cli law to look like genesis |

**Includes:** lineage, keep/extend matrix. **Excludes:** grok auth verb catalog.

| You do… | What it means | What you type |
|---------|---------------|---------------|
| Check origin | Confirm dest is grok-cli specialized from folder-backup | Open `docs/requirements/requirement-bootstrap-chain.md` |


## 2. Core Rules (Mandatory)

### 2.1 Direction

1. Every edge **MUST** be **ancestor → descendant** only.  
2. Plans **MUST NOT** copy this product’s ship unit onto the bootstrap parent to “share fixes.”  
3. Detected reverse-copy **MUST** be treated as critical pollution (restore parent; rebuild this product).

### 2.2 Chain declaration (this product)

| Field | Value |
|-------|--------|
| **Root / hop 0** | `cli-template` — Type 0 local-only template (sibling workspace; **do not edit from this product**) |
| **Hop 1** | `folder-backup` — architecture parent (sibling workspace; **do not edit from this product**) |
| **Leaf / hop 2 (B)** | `grok-cli` — this workspace product |
| **Immediate origin of leaf** | `folder-backup` (domain) **plus** `selfmanaged` (online package) |
| **Specialize mode** | **Domain retarget** (grok auth backup/sync + sudoers-elevated `backup`) **and** online-package specialize from selfmanaged (`curl\|sh`, Type O off-TTY). |
| **A ship unit (immediate)** | Sibling folder-backup ship unit (do not reverse-copy) |
| **B ship unit** | `src/grok-cli` |
| **A channel ownership (architecture hop)** | **None** — folder-backup is local-only |
| **B channel ownership** | `SCRIPT_URL` default github raw `src/grok-cli` (from selfmanaged) |
| **A domain** | folder tar.gz backup (sibling; not this product) |
| **B domain** | grok session gate + `/var/grok-cli` deposit + `sync-auth` (see `requirement-domain-grok-cli`) |
| **Online-package origin** | sibling `selfmanaged` — specialize channel onto B; **do not reverse-copy** |

### 2.3 Architecture inheritance (B from A)

B **MUST** inherit A’s structural contracts:

| Layer | Inherit / extend |
|-------|------------------|
| Runtime | POSIX `/bin/sh`, `set -u`, explicit errors |
| Output SSOT | `out_*` family |
| Modular prefixes | `out_`, `inst_`, `util_`, `app_`, `path_`, `prompt_`; domain uses dedicated `gc_` prefix |
| Entry / dispatch | Single `app_main`; always call `app_main "$@"` at end |
| Global flags | `--quiet` / `--json` / `--debug` / `--force` / `--global` |
| Integrity companion | **Extend** from selfmanaged (`src/grok-cli.sha256`) |
| Online lifecycle | **Extend** from selfmanaged (`version-check`, `self-update`, `self-uninstall`, Type O off-TTY, `SCRIPT_URL`) |
| Local lifecycle | **Keep** checkout `install` / `uninstall` / `where-is-me` |
| Empty argv | **Extend**: TTY numbered menu; off-TTY Type O ensure |
| Domain | **Add** on B only |

### 2.4 Keep / extend matrix (normative for this product)

| Surface | Decision | Notes for folder-backup |
|---------|----------|-------------------------|
| `out_*` output SSOT | **Keep** | Surgical only |
| Modular single-file design | **Keep** | Ship unit under `src/` |
| Global flags + `app_main` | **Keep** | Same contracts; domain flags added on B |
| Storage resolve | **Keep / adapt** | Staging for tar.gz |
| Idempotency / interactive modes | **Keep / retarget** | Domain confirm paths stay fail-closed |
| Online channel (`SCRIPT_URL`, `REPO_*` as channel) | **Extend** (from selfmanaged) | Default github raw `src/grok-cli` |
| Type O empty argv | **Extend** (off-TTY only) | TTY stays menu |
| Remote `version-check` / `self-update` / `self-uninstall` | **Extend** (from selfmanaged) | Routed verbs |
| Companion `.sha256` product law | **Extend** | `requirement-shell-automatic-checksum` |
| Local `install` / `uninstall` / `where-is-me` | **Keep** | Local self-managed package |
| Domain grok auth + sudoers fragment | **Retarget** | Domain SSOT `requirement-domain-grok-cli` |
| Domain / out Protection Zones | **Keep spirit** | Do not “simplify away” defensive layers for style |

### 2.5 Identity retarget (B only)

| Concern | B value |
|---------|---------|
| `APP_NAME` | `grok-cli` |
| `VERSION` | product version SSOT in ship unit (do not pin a stale number here) |
| Primary install story | Channel `curl \| sh`; checkout `install` secondary |
| README one-liner | Literal `curl -fsSL https://raw.githubusercontent.com/cloudgen/grok-cli/main/src/grok-cli \| sh` |

### 2.6 Implementation Notes (this project)

| Item | Value |
|------|--------|
| **A (architecture)** | `cli-template` → folder-backup (do not reverse-copy) |
| **A (online package)** | sibling `selfmanaged` (specialize online package onto B; do not reverse-copy) |
| **B (this product)** | grok-cli |
| **Specialize intent** | Type 0 architecture + grok auth domain + selfmanaged channel |
| **Install mode** | **dual-mode** (matrix on `requirement-shell-online-install`) |
| **Domain after specialize** | Active `requirement-domain-grok-cli` |
| **Historical origin** | 2026-08-03 named `selfmanaged` with online trim. Domain origin retarget 2026-08-13. Online package re-specialized onto grok-cli 2026-09-02. |

### 2.7 Why This Requirement Exists (CIAO)

- **Principle 2 – Intentional**: Lineage and domain extend are explicit.  
- **Principle 1 – Caution**: Dual-mode is explicit; no half-live channel without a matrix.  
- **Principle 18 / Over-protect**: Reverse-copy is forbidden pollution.  
- **Principle 21 – Dual policies**: Complete B law; portable cores elsewhere.

---

## 3. Design Principles (CIAO / CIAO-Lite)

- **Caution**: Matrix before delete; verify no half-live install.  
- **Intentional**: Explicit keep/extend; registry names absences.  
- **Anti-fragile**: Keep battle-tested `out_*` / modular patterns from A.  
- **Over-protect**: Never reverse-copy; no silent channel reintro.

---

## 4. Protection Rule (Sacred)

**Future AI assistants, Grok, or maintainers MUST NOT**:

1. Reverse-copy `folder-backup` onto `cli-template` or treat reverse as “cleanup.”  
2. Name `selfmanaged` as this product’s live origin without updating this file.  
3. Claim the online package is still trimmed while `SCRIPT_URL` / Type O off-TTY / self-update are Active.  
4. Leave dual-mode online+local install without an explicit dual-mode matrix.  
5. Drop `out_*` / modular Protection Zones as “part of specialize.”  
6. Invent a second bootstrap origin that contradicts this declaration without updating this file.

**Violating this rule is a critical bootstrap-direction regression.**

---

## 5. Acceptance criteria

| ID | Criterion |
|----|-----------|
| AC-1 | Hop table names A=cli-template, B=folder-backup, direction A→B |
| AC-2 | Keep/extend matrix matches Active registry (online package **present**; domain present) |
| AC-3 | B identity retarget complete (`APP_NAME`, `VERSION`, dual-mode install) |
| AC-4 | Domain SSOT present for backup surface |
| AC-5 | `selfmanaged` is the live **online-package** origin (not the domain origin) |

---

## 6. Related requirements (peer keys only)

| Key | Relationship |
|-----|--------------|
| `requirement-class-software-dev` | Class gate |
| `requirement-shell-local-self-management` | Local lifecycle inherited from A |
| `requirement-shell-cli-zero-arguments` | TTY menu; off-TTY Type O (extended from selfmanaged) |
| `requirement-shell-online-install` | Channel + dual-mode matrix |
| `requirement-domain-folder-backup` | Domain extend |
| `docs/requirements/index.md` | Registry |

---

## Design-time verification

| TP family / ID | Suite | Status | Note |
|----------------|-------|--------|------|
| **TP-CLI-04,10** | `tests/test_cli.sh` | have | online verbs routed |
| **TP-CLI-07** | `tests/test_cli.sh` | have | TTY menu; off-TTY Type O ensure |
| **TP-ONL-01..04** | `tests/test_online_install.sh` | have | channel / checksum / version-check / self-uninstall |
| **TP-FOLDER-BACKUP-*** | `tests/test_domain_folder_backup.sh` | have | domain extend |

**Matrix:** `reviews/requirement-test-matrix.md`  
**Map:** `reviews/test-plan.md`

## 7. Status history

| Date | Status | Note |
|------|--------|------|
| 2026-08-03 | Active 1.0.0 | Declared A=selfmanaged → B=folder-backup (trim online) |
| 2026-08-13 | Active 2.0.0 | Re-specialize: A=cli-template → B=folder-backup (domain extend). selfmanaged retired. |
| 2026-08-22 | Active 3.0.0 | Domain retarget grok-cli |
| 2026-08-23 | Active 3.0.1 | Empty argv Type N: TTY menu; off-TTY help |
| 2026-09-02 | Active 4.0.0 | selfmanaged online package specialized onto grok-cli; off-TTY Type O; TTY menu kept |

---

**Last Updated**: 2026-09-02  
**Owner**: project maintainers  
**Alignment**: Registry `docs/requirements/index.md`; **CIAO** (https://github.com/cloudgen/ciao); CIAO-Lite (https://github.com/cloudgen/ciao-lite).
