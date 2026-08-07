# 01 — Product & Scope

Defines what the system is, who it is for, the words it is allowed to use, where its boundary sits, and every quality target it must meet. Everything in [02](02-business-rules.md)–[06](06-decision-log-and-open-items.md) is written in the vocabulary this file establishes.

---

## 1. Purpose

A single-administrator, fully offline desktop application that manages a referral-based distribution network of **500–5,000 members** (architected to 25,000). Each month the administrator records a Business Volume figure against individual members. The system rolls those figures up the hierarchy and computes every member's Rewards from the percentage-slab differential between them and their direct children, plus a royalty bonus, and produces permanent monthly records plus spreadsheet extracts.

Today this is done entirely by hand: a multiplication of percentages across a structure of several thousand people, repeated monthly, where each figure depends on figures one level below it. One error near the base silently corrupts everything above it.

**No money and no currency figure appears anywhere in the software.** Figures are unitless. The administrator converts Rewards to rupees by hand, outside the application.

### 1.1 Business objectives

| ID | Objective | Why it matters |
|---|---|---|
| **BO-1** | Produce an accurate, defensible reward figure for every member, every month, without manual working | The core value. An indefensible figure damages the client's relationship with the member it concerns. |
| **BO-2** | Hold one authoritative record of who introduced whom, that cannot drift or be quietly rewritten | Every reward figure derives from the structure. Uncertain structure means uncertain figures. |
| **BO-3** | Close each month deliberately, with a permanent record — **correctable afterward if wrong, not frozen** | After close, live figures are cleared; the record is the only evidence the month happened. *Revised 4 Aug 2026 — "unalterable" was the architect's framing, not the client's requirement (RQ-7).* |
| **BO-4** | Give visibility of performance across a year, at member and network level | Identifies who is contributing, who has stalled, where the structure needs attention. |
| **BO-5** | Keep every scheme parameter adjustable by the client, without a developer | The scheme will evolve. Waiting on a developer makes the business rigid. |
| **BO-6** | Present the entire system in discreet, non-commercial language | Stated and non-negotiable client requirement. |
| **BO-7** | Keep the system operable by one non-technical person with minimal effort | There is exactly one user; all operational burden falls on them. |

### 1.2 The ideal outcome, twelve months after go-live

The client should be able to say:

1. "I record the month's figures as they come in, and the numbers are right the moment I press save."
2. "When a member questions their figure, I can show them exactly which people below them contributed what."
3. "I have not done a percentage calculation by hand since we launched."
4. "I can open any month from the last three years and it shows me exactly what it showed me at the time."
5. "When I changed the scheme in March, I changed it myself, in two minutes, without calling anyone."
6. "Nobody outside this office has ever seen this system, and nothing in it reads like a trading document."

---

## 2. Glossary — the four quantities, kept apart

**This is the single most important section in the specification.** The client's original notes used one phrase for several different numbers. Every calculation depends on keeping them apart, and a terminology rename on 3 August 2026 changed what one of these words means.

| Term | Definition | How it changes |
|---|---|---|
| **Business Volume** | The figure the administrator records directly against **one** member. | Manual entry only. Zeroed at monthly close. |
| **Total Business Volume** | A member's own Business Volume **plus the already-computed Total Business Volume of each direct child** — one level of addition only. Full-depth coverage follows transitively because each child's figure is itself complete. | Derived. Recomputed whenever any Business Volume beneath the member changes. |
| **Slab %** | The percentage band a member falls into, looked up from their **Total Business Volume** — never from their own Business Volume. | Derived from Total Business Volume. |
| **Rewards** | A member's score for the period = Differential + Royalty. | Derived. **A separate ledger** — never added to any volume figure (Rule-13). |

### 2.1 The rename of 3 August 2026 — read this before any older document

| Was called | Is now called | What it actually is |
|---|---|---|
| Individual Credit Points | **Business Volume** | What the administrator types in against one member |
| Business Volume | **Total Business Volume** | That member's own figure **plus** their whole team below |
| Earned Points | **Rewards** | The score = differential + royalty |

⚠️ **"Business Volume" changed meaning.** It used to mean the rolled-up team figure; it now means a member's own directly-entered figure. Anyone holding a pre-3-August copy of any document will read every formula backwards.

Consequently, and bindingly:

- **The abbreviations `ICP` and `BV` are retired entirely.** They must never appear in code, comments, variable names, commit messages, or UI strings. Reusing `BV` for its new meaning is the easiest possible way to introduce a silent error. All three quantities are always spelled out in full.
- **"Credit points" is gone, and so is "points" as a unit.** Figures are stated bare: *Person A's Business Volume is 300.*
- `documents/draft/requirement-draft.md` deliberately still uses the old vocabulary. It is the historical source and is never updated.

### 2.2 Other terms

| Term | Meaning |
|---|---|
| **Member** | A person in the referral hierarchy. |
| **Royalty** | The reward earned when enough direct children reach the top slab. |
| **Top slab** | Always the row in the slab table holding the **highest percentage**, whatever its threshold. Never hardcoded to 14% or 10,000. |
| **Period** | A calendar month, from open through close. |
| **Chain** | The path from a given member up to the root, used for incremental recalculation. |
| **Snapshot** | A permanent, versioned record of a member's figures for a closed period. |
| **Leg** | One direct child. "Number of legs" = count of direct children. |
| **Introducer / sponsor / Reference ID** | The member who introduced this member. Fixed permanently at creation. |
| **Console** | The whole installed application and its single database file. |

---

## 3. The vocabulary constraint — UN-27, BC-4, AC-36

**Binding on every user-visible string without exception.**

| | |
|---|---|
| **Permitted vocabulary** | *member, Business Volume, Rewards, royalty, volume, slab, level, leg* |
| **Excluded, absolutely** | *sale, purchase, order, cash, payment, commission, invoice* — and any equivalent |

The constraint applies to: screen labels, buttons, column headings, table headers, tooltips, placeholder text, empty-state copy, error messages, toast text, modal titles, **extract filenames**, and mock/test data if it could ever appear in a screenshot or demo.

Extract filenames matter disproportionately: they leave the system and travel. A filename must identify the period, never a member (also an NFR-4 requirement — no personal data in filenames).

This is not a presentational preference. It is a condition of the system being acceptable at all. A pre-release automated grep of every UI string against the excluded-word list is a mandatory check (see [05](05-quality-and-acceptance.md) §3).

---

## 4. Users

### 4.1 P-1 — Business Owner / Administrator 🟢 *Primary, and the only user*

| Attribute | Detail |
|---|---|
| **Who** | Siddharth Patel — owner of the network, sole account holder. Holds the entire scheme in their head today. Runs the network personally, not through a team. |
| **Goals** | Record activity quickly and without ceremony. Trust the figures without re-checking them. Close each month cleanly. See who is performing. Keep total control of scheme parameters. |
| **Responsibilities** | Onboarding every member. Recording all activity. Triggering and completing the monthly close. Taking and safeguarding backups. Adjusting settings. Communicating rewards to members, outside the system. |
| **Technical skill** | **Low to moderate.** Comfortable with a browser and spreadsheets. Not a technical user. **Will not read documentation** — every recovery path must be self-evident from the screen. |
| **Environment** | One desktop or laptop, office or home. Extracts open in a spreadsheet application. |
| **Frequency** | Daily–weekly for recording and member questions; monthly for close, backup and reporting; occasional for settings. |
| **What failure looks like** | A figure they cannot explain to a member. A month that cannot be closed. A number that changed since they last looked at it. |

### 4.2 P-2 — Network Member / Beneficiary 🟢 *Secondary — zero system access*

500–5,000 people, each introduced by an existing member. **Never logs in, never sees a screen, may not know the system exists** — but their name, contact number and address are held in it, and their reward is determined by it.

Two obligations follow from them holding no access: figures must be explainable by the administrator on their behalf, and their personal details must be handled responsibly by a system they cannot see into. Retention is permanent, by explicit client requirement.

### 4.3 P-3 — Solution Architect / Maintainer 🟢 *Secondary*

Keyur Patel. Builds and supports the system on request; no standing operational role. Intensive during build, rare and reactive thereafter.

### 4.4 User goals, by frequency

| ID | Goal | Frequency |
|---|---|---|
| **UG-1** | Record a figure against one member and move on, confident the rest updates itself | Daily |
| **UG-2** | Find a specific member instantly, by name, number or phone | Daily |
| **UG-3** | Answer a member's question about their figure, with the contributing detail to hand | Weekly |
| **UG-4** | See the shape of a branch — who sits under whom | Weekly |
| **UG-5** | Onboard a new member under their introducer, in under a minute | Weekly |
| **UG-6** | Close the month cleanly, with a safe copy taken before anything is cleared | Monthly |
| **UG-7** | Extract the month's figures into a spreadsheet | Monthly |
| **UG-8** | Review who has and has not contributed across the year | Yearly / on demand |
| **UG-9** | Adjust the scheme — a threshold, a percentage, a qualifying count — without help | Occasionally |
| **UG-10** | Be certain nobody else can see any of this | Continuous |

---

## 5. Scope boundary

### 5.1 In scope

| Area | What is included |
|---|---|
| **Structure** | One permanent root member; every other member introduced by an existing one; adjustable depth and level widths, treated as guidance only |
| **Members** | Add, edit, deactivate, reactivate; unique random 6-digit numbers; unique contact numbers; mandatory consent capture |
| **Recording** | Search by name, number or phone; record Business Volume to two decimal places into the current month or into a month that has ended but is not yet closed; correct any entry at any time, in any month |
| **Calculation** | Team totals, slab assignment, differential rewards, royalty, immediate recalculation on every entry |
| **Viewing** | Home search; structure chart, one branch at a time; **full hierarchy view — the whole structure expanded, in a separate read-only window**; member detail with direct team and full reward breakdown; audit log |
| **Monthly close** | Manual close, undismissable alert, mandatory backup gate, permanent versioned record, clearing of all live figures. The current month cannot be recorded into until the outstanding one is closed |
| **Reporting** | Monthly extract with adjustable columns; yearly average extract; low-contribution report; re-download of any past closed month |
| **Settings** | Thresholds, percentages, slab rows, depth, widths, royalty rate and qualifying count, yearly cycle, low-contribution threshold, reference unit value, default extract columns, session timeout, whole-console backup schedule and retention |
| **Access** | One administrator login, PIN and/or password, mandatory failed-attempt lockout, one-time recovery codes |
| **Console backup** | Whole-console backup on a schedule or on demand; restore on any machine including a brand-new install |
| **Language** | Restricted, non-commercial vocabulary everywhere visible |

### 5.2 Out of scope — permanently

| # | Excluded | Basis |
|---|---|---|
| **OS-1** | Any member login, member screen, or member notification | Confirmed — Rule-29 |
| **OS-2** | Any currency figure on any screen, report or extract | Confirmed — Rule-14, Rule-16 |
| **OS-3** | Currency conversion of any kind inside the system | Confirmed — done by hand outside |
| **OS-4** | Any movement or handling of money | Throughout |
| **OS-5** | Any discount capability | Confirmed — "final discounts" meant final Rewards |
| **OS-6** | Additional logins, roles or permission levels | Confirmed — Rule-29 |
| **OS-7** | Changing a member's introducer | Confirmed — Rule-37, no override ever |
| **OS-8** | Permanent deletion of a member or their history | Confirmed — Rule-28, Rule-42 |
| **OS-9** | Automatic monthly close | Confirmed — Rule-17 |
| **OS-10** | Stock, catalogue, or anything describing goods | Never in scope |
| **OS-11** | Integration with any other system — no import/export API, no plugin surface, no scripting | Never discussed; architected as a closed box |
| **OS-12** | Migration of existing member or activity data | Confirmed 3 Aug 2026 — the system starts empty |
| **OS-13** | Phone or tablet use | Confirmed 3 Aug 2026 — single offline desktop only |
| **OS-14** | Any language other than English | Confirmed 4 Aug 2026 |
| **OS-15** | Viewing a past month on screen | Confirmed 3 Aug 2026 — extracts only. *Note: this is unrelated to editing one specific entry within a closed month by search (M2.4), which is in scope.* |

### 5.3 Deferred — not this build, but the design does not preclude them

| Item | Why deferred |
|---|---|
| Currency conversion inside the system | Client explicitly does not want it now; may be added later if asked |
| Viewing a past month on screen | Only extracts specified today (RQ-14). Snapshots hold everything needed to add it later without rework |
| Member self-service view | Not required; a separate piece of work |
| Additional logins or roles | Not required today |
| Moving an entry between months | RQ-21: date edits stay within their own month. A cross-month move would be an explicit separate action, not a silent date-field behaviour |
| Phone or tablet use | Never discussed |
| A width-stable (indented outline) full hierarchy layout | The client chose the top-down chart on 7 Aug 2026 knowing its width behaviour (TR-7). The outline remains the fallback if the chart proves unusable at scale — the data path would not change |

---

## 6. Positioning

Not a general-purpose network-marketing or MLM platform. Off-the-shelf network software was **explicitly rejected** for using commercial language the client will not use.

It is a private, single-user calculation and record-keeping tool built entirely in the client's own restricted vocabulary. It moves no money, holds no currency figure anywhere, gives members no access of any kind, and every scheme parameter is client-editable without a developer.

**Branding.** No company name or commercial branding appears anywhere in the UI — it is a private tool nobody outside the client's office ever sees. A plain visual app icon or mark (window icon, sidebar, sign-in and setup screens) is acceptable; it identifies the application, not a business.

---

## 7. Non-functional requirements

All sixteen, as confirmed by the client (3–7 August 2026), with how the architecture satisfies each.

| ID | Quality | Requirement | Satisfied by |
|---|---|---|---|
| **NFR-1** | **Performance** | Any screen **< 2 s**; recalculation **< 2 s**; extracts **< 30 s**. Targets are fixed regardless of volume. Sizing: ~**1,000** Business Volume entries per month, explicitly approximate and variable (confirmed 4 Aug 2026). **Exception, agreed 7 Aug 2026:** the full hierarchy view (FR-10) is **outside** the two-second screen budget — it draws the entire network at once, in a separate window, behind an explicit confirmation naming the member count. The budget that binds it instead is that the **main console stays responsive** while it draws (AC-45) | ADR-005 chain-upward recalculation — cost is *O*(depth × average width), independent of total member count. The full view's isolation in a separate window is what keeps its cost off every other screen (TR-7) |
| **NFR-2** | **Scalability** | Design ceiling **25,000 members / 200,000 entries per year**. Client's actual scale 500–5,000 | ADR-005; SQLite handles this row volume comfortably |
| **NFR-3** | **Availability** | ~100% — meaning the *software* is available whenever the machine is on. **Not** protection against the client's device failing (that is NFR-13) | ADR-001 — no server or network dependency to fail |
| **NFR-4** | **Security** | Encryption at rest; session/inactivity lock; mandatory failed-attempt lockout; **no member data in extract filenames**. "Encryption in transit" is **ruled inapplicable** — nothing ever transits | ADR-003, ADR-008; see [04](04-technical-architecture.md) §8 |
| **NFR-5** | **Auditability** | A recording log: date and time, member affected, value before, value after, what caused it | Module M9, `audit_log` table |
| **NFR-6** | **Maintainability** | Every scheme parameter client-editable without a developer | ADR-010 — settings-driven, no hardcoded business constants |
| **NFR-7** | **Compliance** | India's DPDP Act 2023. Consent captured at onboarding. Retention **permanent and complete** — members never removed, all data persists including in exports | `consent_given`/`consent_date` on `members`; Rule-28, Rule-42 |
| **NFR-8** | **Accessibility** | "Standard good practice" — readable text sizes, sufficient contrast, full keyboard operation. **No formal conformance commitment** | shadcn/ui baseline; status is never colour-only (labelled pills) |
| **NFR-9** | **Localisation** | English only, Indian date format, no currency anywhere | UI-layer formatting; no currency field exists in the schema |
| **NFR-10** | **Reporting** | Three extracts — monthly, yearly average, low-contribution — in spreadsheet format | Module M6, ADR-007 |
| **NFR-11** | **Logging** | Technical logging exists, distinct from the audit log, and is **never visible to the client** | Module M9 — separate rotating file, no UI surface |
| **NFR-12** | **Monitoring** | 🔶 **Declined by the client, 3 Aug 2026.** Nothing detects a close silently failing to write its record or backup | **Deliberately absent.** Recorded so it is not mistaken for an oversight. Do not build or test it |
| **NFR-13** | **Backup & recovery** | On-demand backup of the in-progress month; two physically independent copies; a new backup **version** per correction; **whole-console** scheduled/on-demand backup restorable on any machine | [04](04-technical-architecture.md) §9; ADR-006, ADR-012 |
| **NFR-14** | **Hosting & deployment** | Standalone offline desktop application. **No network, no server, no internet dependency of any kind.** Windows + macOS. No auto-update | ADR-001, ADR-011 |
| **NFR-15** | **Browser & device support** | None. No browser is used; no phone or tablet support | Tauri native desktop app — no web deployment target exists |
| **NFR-16** | **Data migration** | None. The system starts empty | No import tooling built |

---

## 8. Constraints

### 8.1 Business constraints

| ID | Constraint |
|---|---|
| **BC-1** | Exactly one member at the top level, permanently. It can never grow. |
| **BC-2** | A member's introducer is fixed at the moment they are added and can never change. |
| **BC-3** | A member is never permanently removed. Deactivation is the only removal. |
| **BC-4** | No commercial vocabulary anywhere the client or anyone else can see it. |
| **BC-5** | No currency figure on any screen, report or extract. Conversion is done by the client, by hand, outside the system. |
| **BC-6** | The monthly close is manual. The system prompts and blocks, but never closes a month on the client's behalf. |
| **BC-7** | The system moves no money and holds no financial instrument. It produces figures only. |
| **BC-8** | Royalty stacks at every qualifying level, with no cap. Confirmed with the cost consequence understood. |

### 8.2 Technical constraints

| ID | Constraint |
|---|---|
| **TC-1** | Every figure correct immediately on save; **no recalculation control anywhere**. |
| **TC-2** | Two decimal places throughout; rounding only at the point of display. |
| **TC-3** | All extracts in spreadsheet format. |
| **TC-4** | Expected network size 500–5,000 members; design ceiling 25,000. |
| **TC-5** | One account, no roles, no member access. |
| **TC-6** | Member numbers are random and never reissued, in the range **100001–999999**. |
| **TC-7** | A standalone desktop application, fully offline. Not browser-based. |

### 8.3 Operational constraints

| ID | Constraint |
|---|---|
| **OC-1** | One user operates the entire system. No cover, no second pair of hands. |
| **OC-2** | From the moment a month ends until it is closed, recording continues **into that month** but not into the current one. Amended 7 Aug 2026 (CR-2); previously *"recording is fully locked from the moment a month ends until that month is closed."* |
| **OC-3** | Where several months are open, only the oldest can be closed; the rest wait behind it. |
| **OC-4** | The close cannot proceed without a confirmed backup. |
| **OC-5** | Backups and monthly records are retained permanently. Nothing is deleted automatically. |
| **OC-6** | 🟡 The client's availability at each month end determines when the **new** month can start being recorded. Materially eased 7 Aug 2026 (CR-2): activity in the month that has ended can still be recorded throughout, so nothing is lost while the client is away — only the new month waits. Previously a hard 🔴 dependency of the business continuing to record at all (RQ-11's original answer). |

### 8.4 Compliance constraints

| ID | Constraint | Status |
|---|---|---|
| **CC-1** | The system holds personal data — name, contact number, address — for several thousand people with no access to it. India's DPDP Act 2023 applies | 🟢 **Resolved.** Retention permanent (RQ-8, 3 Aug). Consent is asked of every member at onboarding and captured in the system as a mandatory checkbox and date (RQ-22, 4 Aug) |
| **CC-2** | No retention limit for member personal data | 🟢 **Resolved 6 Aug 2026.** Permanent, complete retention is the explicit client requirement — see Rule-42. *The user-needs document still marks this 🔴 "needs a client decision"; that marker is stale — see [06](06-decision-log-and-open-items.md) C8* |
| **CC-3** | No route exists for a member to have their details erased | 🟢 **Resolved 6 Aug 2026.** Correction is supported (`edit_member`, fully audited). Erasure is **out of scope by explicit client requirement** — no erasure path exists and none is to be built. *Same stale 🔴 marker — see [06](06-decision-log-and-open-items.md) C8* |
| **CC-4** | The client bears no external audit, tax or regulatory reporting obligation arising from these figures | 🟢 Confirmed 4 Aug 2026 |

> These entries record what was decided, not legal advice. The client has taken their own advice on their obligations.

---

## 9. Business assumptions — all resolved

Every assumption originally inferred by the architect has since been confirmed or corrected by the client. None remains open.

| ID | Assumption | Outcome |
|---|---|---|
| **BA-1** | Nobody but the client consumes the extracts — no accountant, partner or auditor | 🟢 Confirmed 4 Aug 2026 |
| **BA-2** | No existing member or activity data to bring in; the system starts empty | 🟢 Confirmed 3 Aug 2026 — no migration |
| **BA-3** | The client works from one computer at a time; two sessions never run at once | 🟢 Confirmed 4 Aug 2026 |
| **BA-4** | English only | 🟢 Confirmed 4 Aug 2026 |
| **BA-5** | A single offline desktop application; no browser, phone or tablet | 🟢 Confirmed 3 Aug 2026 |
| **BA-6** | The threshold table will always rise — a higher threshold always carries a higher percentage | 🟢 A **client undertaking**, 3 Aug 2026, with the software validation explicitly declined. Nothing in the system checks this — see Rule-41 |
| **BA-7** | The client tells members about their rewards themselves, outside the system | 🟢 Confirmed 4 Aug 2026 |
| **BA-8** | The reference unit value (1 = 500 Rs) is applied to final **Rewards** | 🟢 Confirmed 3 Aug 2026 — the setting is labelled accordingly |
| **BA-9** | A mis-recorded figure only needs correcting within its own open month | 🔷 **Wrong, and reversed 4 Aug 2026.** The client wants correction even after close — see Rule-39 |
| **BA-10** | Activity is recorded steadily through the month, not batched at month end | 🟢 Confirmed 4 Aug 2026 |
| **BA-11** | No external audit, tax or regulatory obligation arises from these figures | 🟢 Confirmed 4 Aug 2026 |

---

## 10. Business risks

Fourteen risks were identified. Eleven are closed, one is mitigated (R-4, by CR-2 on 7 Aug 2026); two remain live and are accepted rather than mitigated.

| ID | Risk | L | I | Position |
|---|---|---|---|---|
| **R-1** | A month's record is lost entirely — the close clears everything, so a close on a failed backup leaves no evidence | Low | **Critical** | 🟢 **Closed 3 Aug 2026.** The retained in-system copy is the verifiable gate (RQ-6) |
| **R-2** | An edited threshold table produces negative rewards — Rule-9's guarantee depends on the table always rising | **Medium** | **High** | 🔶 **LIVE — accepted by the client, not mitigated in software.** The recommended validation was explicitly declined. See Rule-41, ADR-009 |
| **R-3** | A wrong figure cannot be traced — one login, instant recalculation, no record of change | Medium | High | 🟢 **Closed 3 Aug 2026.** Audit log built (RQ-9, NFR-5) |
| **R-4** | The business stops being recorded because a month end falls while the client is away | **Low** *(was Medium)* | **Low–Medium** *(was High)* | ✅ **MITIGATED 7 Aug 2026 by CR-2.** Recording no longer stops when a month ends: activity dated in the ended month can be recorded for as long as it stays unclosed, which is precisely the case this risk described. What remains is narrower — the **new** month cannot be recorded into until the old one is closed, so a long absence still defers new-month recording (and those figures must then be entered afterwards, dated correctly). The undismissable alert (Rule-20) remains the pressure to close. Also OC-6 |
| **R-5** | Total loss of access — one login, one credential | Low | Critical | 🟢 **Closed.** Recovery codes at setup (3 Aug); strengthened 4 Aug by dual credentials, either authenticating |
| **R-6** | Personal data exposure — thousands of people's details behind one credential | Low | High | 🟢 **Closed.** Permanent retention agreed, mandatory lockout, consent capture (RQ-8, RQ-22) |
| **R-7** | ~~Deactivation produces wrong figures~~ | — | — | 🟢 **Closed 4 Aug 2026.** Inactive is display-only, zero calculation effect (RQ-2) |
| **R-8** | Royalty cost grows with depth — it stacks at every qualifying level with no cap | **Medium** | **Medium** | 🔶 **LIVE — understood and accepted.** Recommendation: review total royalty as a figure each month once live |
| **R-9** | Confidence is lost in the first month — the client will check early figures by hand | Medium | High | Mitigation: reconcile all five worked scenarios with the client, in front of them, before handover |
| **R-10** | ~~Building from a superseded statement~~ | — | — | 🟢 **Closed 3 Aug 2026** (INC-1–INC-5). **This document set is the standing mitigation** |
| **R-11** | ~~Extract columns drift into new capture requirements~~ | — | — | 🟢 **Closed 3 Aug 2026.** Joining date now captured automatically (RQ-15) |
| **R-12** | ~~Single-machine backup independence~~ | — | — | 🟢 **Closed 4 Aug 2026.** The downloaded copy goes to a physically separate medium (RQ-19) |
| **R-13** | ~~Historical-correction provenance undefined~~ | — | — | 🟢 **Closed 4 Aug 2026.** Original backup untouched; a new dated version per correction (RQ-20, RQ-21) |
| **R-14** | ~~Consent has no evidence trail~~ | — | — | 🟢 **Closed 4 Aug 2026.** Mandatory checkbox and date at Add Member (RQ-22) |

---

## 11. Product principles

Five principles that decide close calls during implementation.

1. **The screen is never stale.** Every affected figure is correct the instant an entry is saved. No recalculate control, no state where a number on screen might be out of date.
2. **A month's evidence cannot be destroyed.** The close-and-clear action is irreversible by design, so the backup gate before it is a genuine transactional precondition — never a dismissable prompt.
3. **Nothing is silently lost, and nothing is silently frozen.** Members are deactivated, never deleted. Monthly records are corrected via a new version, never overwritten. History stays reconstructable exactly as it stood at any point.
4. **The scheme belongs to the client, not the codebase.** Every threshold, percentage, rate, width and depth is data the client edits themselves. Nothing is hardcoded.
5. **One field, one action, no ceremony.** The most frequent action — recording a Business Volume figure — is optimised to a single field and a fifteen-second completion target. Every additional field or decision on that path is a cost paid on every use.

---

## 12. Handover deliverables

What the client receives, as confirmed in the project confirmation summary:

- A private desktop application — no internet needed, nothing browser-based
- Three ready-to-use extracts, opening directly in a spreadsheet
- Permanent, safe backups, taken before anything is ever cleared
- A recovery method, in case a login is ever forgotten
- A change log, so any figure can always be explained
