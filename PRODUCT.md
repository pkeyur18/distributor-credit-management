# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Stack

Tauri v2 desktop app: React + TypeScript UI in an OS-native WebView, Rust application core, SQLCipher-encrypted SQLite (see `documents/final/04-technical-architecture.md`, ADR-001–003). shadcn/ui + Tailwind CSS component library. Fully offline — no server, no network code, no auto-update. Cross-platform bundle: Windows + macOS.

`documents/design/ui-prototype-v2.html` is a standalone static HTML/CSS/JS mockup used for design iteration and client sign-off — not the implementation, and not built with the above stack.

## Users

One user, permanently: the network's business owner/administrator (client: Siddharth Patel), who is also the sole account. Low-to-moderate technical skill — comfortable with a browser and spreadsheets, not a technical user, will not read documentation; every recovery path must be self-evident from the screen. Uses the system daily/weekly to record activity and answer member questions, monthly to close the month and report, occasionally to adjust settings.

Network members (500–5,000, referral-introduced) are the subjects of the data but have zero system access, zero visibility, and never log in — their rewards are computed and communicated to them by the admin outside the system entirely.

A maintainer/solution architect (Keyur Patel) builds and supports the system on request but has no standing operational role.

## Product Purpose

Replaces a manual, error-prone, hand-calculated process for a referral-based distribution network. Holds the member hierarchy permanently, accepts one monthly Business Volume figure per member, instantly computes every affected member's Total Business Volume, band, differential reward, and royalty up the chain, and produces a permanent, correctable monthly record plus spreadsheet extracts (monthly, yearly average, low-contribution).

Success: the admin records activity in seconds, trusts every figure without re-checking it by hand, can explain any member's reward from one screen, and can open any past month (even corrected ones) and see exactly what it showed.

## Positioning

Not a general-purpose network-marketing / MLM platform. It is a private, single-user calculation and record-keeping tool built entirely in the client's own restricted, non-commercial vocabulary (*member, Business Volume, Rewards, royalty, volume, slab, level, leg* — nothing else, anywhere visible, including filenames and error messages). It moves no money, holds no currency figure anywhere, gives members no access of any kind, and every scheme parameter (thresholds, percentages, royalty rate, widths, depth) is client-editable without a developer. Off-the-shelf network/MLM software was explicitly rejected for using commercial language the client won't use.

## Operating Context

- Fully offline desktop application, one machine, one session at a time — no browser, no phone/tablet.
- Recursive, chain-upward calculation: a member's Total Business Volume = own Business Volume + Total Business Volume of each direct child; band is set by the team figure, not the personal figure; differential and royalty derive from band gaps against direct children only. A member's own Business Volume also earns a reward at their own band (added 8 Aug 2026, CR-4) — a third, additive term alongside differential and royalty, not a change to either.
- Calendar-month operating cycle, closed manually by the admin. Once a month ends, an undismissable alert stands until it is closed; figures dated **in that month** can still be recorded, while the **current** month is refused until the older one closes (Rule 36, amended 7 Aug 2026). Close is gated on a confirmed backup (fails-closed — nothing clears without it) and writes a permanent, versioned snapshot per member before clearing live figures.
- Corrections are allowed at any time, even in already-closed months — via a new snapshot/backup version, never an overwrite. Original backups are never touched.
- Separately, the whole console — every member, entry, snapshot and setting, not just one month — backs up on a configurable schedule (off/daily/weekly/monthly) or on demand, and restores on any machine, including a brand-new install with nothing set up yet. It's the same encrypted database file, copied and verified, credentials included — no separate export format, no re-setup after a restore. Restoring always names what it will replace, requires deliberate confirmation, and takes one more backup of the current state first.
- Reports/extracts open in a spreadsheet application (three extracts: monthly, yearly average, low-contribution).
- Single administrator login, PIN and/or complex password (either authenticates), mandatory failed-attempt lockout, one-time local recovery codes. No cloud/network auth of any kind.

## Capabilities and Constraints

**In scope:** member/structure management (permanent introducer-based hierarchy, unique 6-digit member numbers, deactivate/reactivate, never hard-delete); Business Volume entry (single-field, up to 2 decimals, search by name/number/phone, recordable into the current month or a month that has ended but is not yet closed); calculation engine (team totals, bands, differential, royalty, immediate recalculation, no manual "recalculate" control anywhere); search & structure chart, one branch at a time, plus a full hierarchy view of the whole network in a separate read-only window (each node shows exactly name, number, own Business Volume — nothing more, in either view); monthly close with mandatory backup gate; three spreadsheet extracts; fully client-editable settings (band thresholds/percentages/rows, depth, level widths as guidance only — never enforced as a hard block, royalty rate/qualifying count, yearly cycle, low-contribution threshold, default extract columns, whole-console backup schedule/retention); one admin login with lockout; whole-console backup (scheduled or on demand) and restore (on a running console or a brand-new install) as a second, orthogonal mechanism alongside the monthly close backup.

**Explicitly excluded, permanently:** any member login or member-facing screen; any currency figure anywhere (conversion is manual, outside the system); any movement of money; multiple logins/roles; changing a member's introducer once set (no override, ever); permanent deletion of a member; automatic monthly close.

**Known accepted risk (client decision, not a build gap):** the settings screen does not validate that slab percentages rise monotonically with thresholds — the client declined that safeguard and accepts the residual risk of a silently negative reward if the table is ever misconfigured (see architecture.md ADR-009).

**Undecided / explicitly deferred (not this build):** currency conversion inside the system; viewing a past month on-screen (only extracts exist today); member self-service view; additional logins/roles; phone/tablet use.

## Brand Commitments

No company name or commercial branding surfaces anywhere in the UI — it is a private tool nobody outside the client's office ever sees. A plain visual app icon/mark (window/tab icon, sidebar, sign-in and setup screens) is fine — it identifies the app, not a business. The only binding voice constraint is the restricted vocabulary: *member, Business Volume, Rewards, royalty, volume, slab, level, leg*. No other commercial/network-marketing terminology may appear in any screen label, button, column heading, extract filename, error message, or tooltip.

## Evidence on Hand

- `documents/business/client-requirements-validation.md` and `documents/business/user-needs-document.md` — fully client-confirmed requirements (all 27 open questions/inconsistencies resolved as of 4 August 2026, plus RQ-23 — whole-console backup and cross-device restore — confirmed 7 August 2026), including numeric worked scenarios the calculation engine must reproduce exactly. `documents/final/` and `documents/implementation-readiness/` carry two further confirmed changes from 8 August 2026 (CR-4 — own-Business-Volume reward; CR-5 — Home Rewards-by-slab chart), bringing the worked-scenario set to six.
- `documents/final/04-technical-architecture.md` — complete system architecture with 12 ADRs, full schema, algorithm trace. (`documents/design/architecture.md` is an earlier draft of the same content, superseded by this file — not used for implementation.)
- `documents/design/ui-prototype-v2.html` — current working design prototype, iterated against client review feedback.
- No existing member or activity data — the system starts empty; no migration.
- No customer testimonials, case studies, or press exist or should be fabricated — there is exactly one user and the product has not shipped.

## Product Principles

1. **The screen is never stale.** Every affected figure is correct the instant an entry is saved — no recalculate control, no state where a number on screen might be out of date.
2. **A month's evidence cannot be destroyed.** The close-and-clear action is irreversible by design, so the backup gate before it is a genuine transactional precondition, never a dismissable prompt.
3. **Nothing is silently lost, and nothing is silently frozen.** Members are deactivated, never deleted; monthly records are corrected via a new version, never overwritten — history stays reconstructable exactly as it stood at any point in time.
4. **The scheme belongs to the client, not the codebase.** Every threshold, percentage, rate, width, and depth is data the client edits themselves; nothing is hardcoded.
5. **One field, one action, no ceremony.** The system's most frequent action (recording a Business Volume figure) is optimized to a single field and a fifteen-second completion target — every additional field or decision on that path is a cost paid on every use.

## Accessibility & Inclusion

No accessibility standard or specific need was stated by the client. Standard desktop UI craft applies as a baseline (readable type, sufficient contrast, not relying on color alone — relevant given the inactive-member color coding already required by M4.5/M6.5) but is not a formal compliance requirement here.
