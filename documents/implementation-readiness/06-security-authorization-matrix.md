# Security & Authorization Matrix

## 1. Roles

Exactly one role exists in this system: **Administrator**. There is no second role, no read-only role, no member-facing role of any kind (Rule-29, FR-9). Network members have **zero system access** — they never log in, never see a screen, and per `user-needs-document.md`'s P-2 persona, "may not know the system exists." This is a hard architectural boundary, not a configuration default: no login screen, session, or IPC command exists for a member identity anywhere in the design.

## 2. Resource / Operation / Permission / Data-Scope Matrix

| Role | Resource | Operation | Permission | Data Scope |
|---|---|---|---|---|
| Administrator | Members | Create, Read, Update, Deactivate/Reactivate | Full | All members, no restriction |
| Administrator | Members | Hard delete | **Denied — not offered anywhere** | N/A (Rule-28: never hard-deleted) |
| Administrator | Members | Change introducer/sponsor | **Denied — not offered anywhere** | N/A (Rule-37: fixed at creation) |
| Administrator | Business Volume entries | Create, Read, Edit | Full, including closed-month corrections | All entries (Rule-39) |
| Administrator | Business Volume entries | Delete | **Denied — not offered anywhere** | N/A — correction is always an edit that preserves history via versioning, never a delete |
| Administrator | Slab table / Royalty settings / all §7 settings | Read, Update, Add/Remove (slab rows) | Full | System-wide (single tenant) |
| Administrator | Monthly close | Trigger, view status | Full, but constrained to oldest-outstanding-first (Rule-20) | All periods |
| Administrator | Backups | Generate, list, re-download | Full | All backups, all versions |
| Administrator | Exports | Generate (monthly, yearly average, low-contribution, closed-month snapshot) | Full | All members/periods |
| Administrator | Audit log | Read | Full | All entries, filterable by member |
| Administrator | Own credentials | Set (first-run), change, recover | Full, self-service only | Self only — no "admin resets another admin's password," since only one account exists |
| *(none)* | Members (network members, as data subjects) | Any operation on the system itself | **No access of any kind** | N/A |

There is no resource in this system with partial/conditional admin access — the single administrator has unconditional access to every feature and every member's data, by design (single-tenant, single-operator system). This matrix is intentionally flat; a richer matrix would misrepresent the actual authorization model.

## 3. Authentication

**Mechanism (Rule-29, corrected/extended):** A 6-digit PIN and a complex password (≥8 characters, letter + number) may **both** be configured simultaneously; either credential authenticates a login. This overrides `requirement-spec.md`'s framing of PIN-vs-password as a still-pending either/or choice — `client-requirements-validation.md` M8.5 (4 August 2026) resolves it as dual-credential support, and `architecture.md`'s ADR-008 already reflects this.

**Hashing:** Argon2id (memory-hard, GPU-resistant), per ADR-008. No plaintext credential is ever persisted.

**Session model:** Single session, single machine. No concurrent-session concept exists (no server, no second device). Session key material (the SQLCipher decryption key, derived from the credential at login) is held only in Rust process memory for the session's lifetime and is dropped on inactivity lock — genuinely inaccessible after lock, not merely UI-hidden (`architecture.md` §11.1).

**Inactivity lock:** Configurable timeout (`settings.session_timeout_minutes`); locking requires re-authentication to resume, which re-derives the encryption key. Manual lock is also available (`lock_session`).

**Recovery:** One-time recovery codes, generated once at first-run setup, shown once, stored hashed. This is the **sole** recovery path — there is no "forgot password" email flow (no network exists) and no vendor backdoor. Using a recovery code invalidates all prior codes and issues a fresh set.

**Pre-authentication surface.** Six commands run without an authenticated session, and the list is closed: `login`, `setup_first_run`, `use_recovery_code`, and — added 6 August 2026 with the data-recovery screen (LOW-3) — `check_data_readable`, `list_restore_points`, `restore_from_backup`. The last three are unauthenticated of necessity, not convenience: the recovery screen exists precisely because the database cannot be opened, and the credential hashes live inside that database, so there is nothing available to authenticate against.

What that exposes, stated plainly rather than left implicit:
- `check_data_readable` and `list_restore_points` reveal only *that* backups exist and which months they cover — no member data, no figures.
- `restore_from_backup` is the **only destructive unauthenticated command in the system**. Someone with physical access to an unlocked machine could roll the data back to an earlier month. This is accepted: it destroys no backup (every version is retained, Rule-31), reveals nothing, and physical device access is already out of scope in the threat model below. It must still verify the backup's checksum before overwriting, so a corrupt backup cannot be restored over a corrupt database.
- The restored database is still encrypted, and still requires the credential to open. Restoring does not grant access to anything.

This list must stay identical to the one in [04-api-specification.md](04-api-specification.md) §Command surface summary.

**Failed-attempt lockout — mandatory, regardless of credential type.** `requirement-spec.md` itself states this is not optional: a 6-digit PIN is one million combinations and is trivially brute-forced without a limit, and this single account guards every member's personal data and phone number. Implementation: 5 failed attempts → timed lockout with exponential backoff (matches the prototype's 20-second countdown at the first threshold).

## 4. Data Protection

| Concern | Mechanism | Source |
|---|---|---|
| Encryption at rest | SQLCipher-encrypted SQLite database file, key derived via Argon2id at login | ADR-003, ADR-008 |
| Encryption in transit | **Explicitly ruled inapplicable** — no network exists, so there is no "transit" | NFR-4, `client-requirements-validation.md` §11 |
| No member data in filenames | Backup/export filenames must never embed a member name/phone/ID in a way that leaks personal data outside the encrypted store | `client-requirements-validation.md` §11.4 |
| Filesystem isolation | WebView (Presentation container) has zero general filesystem/shell/network capability — only the specific named Tauri commands in [04-api-specification.md](04-api-specification.md) are allowlisted | `architecture.md` §11.3, ADR-002 |
| Vocabulary/no-financial-language leakage | No user-visible string (including error messages, tooltips, export filenames) may use sale/purchase/order/cash/payment/commission/invoice | `requirement-spec.md` §1.2, `user-needs-document.md` UN-27 |

## 5. Threat Model — What Is, and Is Not, Defended

`architecture.md` §11.5 states this explicitly and it should not be silently "improved" by a future developer without a client conversation — these are documented, deliberate scope boundaries, not oversights.

| Threat | Defended? | Notes |
|---|---|---|
| Brute-force credential guessing | **Yes** | Mandatory lockout, Argon2id, both credential types |
| Database file theft/copy | **Yes** | SQLCipher encryption at rest — file is useless without the credential |
| Network-based attack (interception, remote exploit) | **N/A — no network exists** | Structural, not policy — no network capability is even declared in the Tauri config |
| Device stolen or accessed while unlocked mid-session | **No — explicitly out of scope** | "Client's responsibility" per the documented threat table; no additional in-app safeguard beyond the configurable inactivity timeout |
| Rollback via the unauthenticated recovery screen (physical access, database already unreadable) | **No — accepted, bounded** | Destroys no backup and reveals no data; the restored database still requires the credential to open. Falls under the same physical-access boundary as the row above. See §3. |
| Loss of both credential and all recovery codes | **No — permanently unrecoverable by design** | No vendor backdoor exists; this is presented as an intentional trade-off of the offline-only, no-cloud architecture, not a gap to fix |
| Slab-table misconfiguration producing an invalid (e.g. negative) differential | **No — explicitly declined by the client** | Rule-41 / ADR-009. Not a security issue but included here because it is the one place a stated business guarantee (Rule-9) is not defended in code, by explicit client choice |
| SQLCipher build fragility / cross-platform crypto issues | Tracked as a technical risk (TR-1), not a security control gap | `architecture.md` §19 |

## 6. Compliance — India's Digital Personal Data Protection Act, 2023

| Item | Status | Notes |
|---|---|---|
| Consent capture at collection | **Implemented** | Mandatory checkbox + auto-captured date at Add Member (Rule-40 / M1.7) |
| Purpose limitation | Implicit — data is used only for hierarchy/reward calculation, never sold/shared (no network exists to share it over) | Not separately documented as a named control, but structurally true |
| Retention | **Permanent and complete, by explicit client requirement** — members are never removed from the application, and all data persists throughout, including in exports (confirmed 6 Aug 2026). Implemented via Rule-38's snapshot model and Rule-28's no-hard-delete. | The business need — figures that can always be explained, years later — is itself the stated justification. This is the requirement, not a constraint to be worked around. |
| Data-subject correction request | Handled — `edit_member`/`edit_entry` support correction of any field, fully audited | |
| Data-subject removal / erasure | **Out of scope by client requirement** (confirmed 6 Aug 2026) | The client has specifically required that no member is ever removed from the application. There is no erasure path and none is to be built. Do not raise this as a gap. |
| Audit obligation | Covered by `audit_log` (NFR-5/M9) | |

## 7. Summary

The security design is unusually complete for a pre-code phase — every control a single-admin, offline, encrypted-at-rest desktop application needs is either implemented or explicitly, deliberately scoped out with a documented rationale. As of 6 August 2026 there are **no open security or compliance items**.
