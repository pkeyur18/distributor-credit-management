# 03 — Functional Specification

FR-1–10, UN-01–31, RQ-1–23 coverage, and a screen-by-screen specification of every view, modal and flow in the approved prototype (`documents/design/ui-prototype-v2.html`), which is the client-signed UI behaviour of record.

---

## 1. Modules — purpose and major functions

Eight functional modules (M1–M8) plus the architecture-introduced M9. Every function below is cited by the business rules it implements — see [02](02-business-rules.md) for the rule text.

### M1 — Member & Structure Management

**Purpose:** Hold the network structure and every member's details, permanently and unambiguously — the foundation of every figure the system produces. **No other module dependency.**

| # | Function | Rules |
|---|---|---|
| M1.1 | Create the single top-level member, once, at initial setup, without an introducer | Rule-1, Rule-30 |
| M1.2 | Add a member: name, phone, email (optional), address, mandatory introducer number | Rule-30, Rule-34 |
| M1.3 | Assign a random, unused 6-digit number automatically | Rule-35 |
| M1.4 | Edit any member's details at any time | Rule-28 |
| M1.5 | Mark a member inactive; recognise and reactivate a returning member by phone number | Rule-28, Rule-34 |
| M1.6 | Refuse any attempt to move a member to a different introducer | Rule-37 |
| M1.7 | Mandatory consent checkbox at Add Member, date auto-captured | Rule-40 |

### M2 — Business Volume Entry

**Purpose:** Let the admin record activity as quickly and unambiguously as possible. **Depends on M1** (member must exist), **M5** (which month may be recorded into — see Rule-36 as amended), triggers **M3** on every save.

| # | Function | Rules |
|---|---|---|
| M2.1 | Search for a member by name, 6-digit number or phone number | Rule-15, Rule-44 |
| M2.2 | Record a Business Volume figure against the selected member | Rule-15, Rule-16, Rule-16a |
| M2.3 | Name the month being recorded into on every entry, and state which month must be closed before the current month can be recorded | Rule-36 |
| M2.4 | Edit or correct a previously recorded entry, at any time, in any month — open **or already closed**, with an explicit warning on a closed-month edit | Rule-39 |
| M2.5 | Every entry carries a date, pre-filled and bounded to the month being recorded into, editable within that month only | Rule-39, RQ-21, Rule-36 |
| M2.6 | Accept entries dated in any month that has ended but is not yet closed, for as long as it stays unclosed | Rule-36 |
| M2.7 | Refuse a current-month entry while any earlier month is outstanding, naming the month that must be closed first | Rule-36 |

### M3 — Calculation Engine

**Purpose:** Turn recorded activity into a reward figure for every member, correctly and consistently, every time. **Depends on M1** (structure), **M2** (figures), **M7** (thresholds, royalty settings). Pure, no UI or IPC surface of its own except the settings-preview command.

| # | Function | Rules |
|---|---|---|
| M3.1 | Work out each member's Total Business Volume from their own figure plus finished figures beneath | Rule-6 |
| M3.2 | Assign each member's slab from their Total Business Volume | Rule-3, Rule-7 |
| M3.3 | Work out the differential reward against each direct child | Rule-8, Rule-9 |
| M3.4 | Assess royalty qualification and work out royalty where earned | Rule-10, Rule-11, Rule-25 |
| M3.5 | Combine the two into Rewards, held in a separate record | Rule-12, Rule-13 |
| M3.6 | Do all of the above immediately on every entry, updating only the affected chain | Rule-26 |

### M4 — Search & Structure Visualisation

**Purpose:** Find any member instantly, see the shape of any branch — and, on demand, the shape of the whole network. **Depends on M1** (structure), **M3** (figures shown).

| # | Function | Rules |
|---|---|---|
| M4.1 | Search from the home screen by name, 6-digit number or phone | Rule-2, Rule-44 |
| M4.2 | Open a member's detail from a search result, showing direct children | FR-1 |
| M4.3 | Visual chart of the structure beneath any chosen member | FR-2 |
| M4.4 | Member's full detail: contact, reward detail, direct team with figures, team total, direct-people count | FR-3 |
| M4.5 | Inactive member shown in a visually distinct colour everywhere they appear — chart, search result, list — informational only | Rule-28 |
| M4.6 | Search by phone number, in every search box, with the phone shown in the results | Rule-44 |
| M4.7 | Full hierarchy view — the whole structure from the top member, every branch expanded, in a separate read-only window | Rule-45, FR-10 |

### M5 — Monthly Close & Permanent Record

**Purpose:** Close each month deliberately, capture it permanently, make it impossible to lose. **Depends on M2** (decides which month it may record into — not whether it may record at all, see Rule-36 as amended), **M3** (figures must be current before the record is written), feeds **M6**.

| # | Function | Rules |
|---|---|---|
| M5.1 | Raise an undismissable alert the moment a month ends, naming it | Rule-20 |
| M5.2 | Keep the outstanding month open for entry, and the current month closed to entry, until that outstanding month is closed | Rule-36 |
| M5.3 | List every outstanding month; only the oldest can be closed | Rule-20 |
| M5.4 | Prompt for a backup; refuse to proceed unless it succeeds | Rule-18 |
| M5.5 | Write a permanent record for every member before anything is cleared | Rule-38 |
| M5.6 | Clear every live figure to zero | Rule-38 |
| M5.7 | Retain every backup permanently inside the system for retrieval | Rule-31 |
| M5.8 | Manual backup of the current in-progress month's data on demand | Rule-31 |
| M5.9 | Editing a closed-month entry recalculates the affected chain and writes a new record version | Rule-39 |
| M5.10 | The original backup for a corrected month is never touched — a new dated version is created alongside it | Rule-39, Rule-31 |

### M6 — Reporting & Extracts

**Purpose:** Get figures out of the system and into a spreadsheet. **Depends on M5** (permanent records), **M7** (yearly cycle, threshold, default columns).

| # | Function | Rules |
|---|---|---|
| M6.1 | Extract a month's figures, with columns chosen | Rule-19, Rule-33 |
| M6.2 | Extract yearly averages per member, with the month count each is based on | Rule-23 |
| M6.3 | Extract the list of members whose personal yearly average falls below threshold | Rule-24 |
| M6.4 | Re-download any past month's backup | Rule-31 |
| M6.5 | Inactive-member rows shown in a visually distinct colour in every extract, alongside the textual active/inactive column | Rule-28, Rule-33 |

### M7 — Settings & Configuration

**Purpose:** Let the admin change every parameter of the scheme themselves. **M3 and M6 both read from here.**

| # | Function | Rules |
|---|---|---|
| M7.1 | Edit thresholds and percentages; add and remove slab rows | Rule-4, Rule-27 |
| M7.2 | Set structure depth and level widths | Rule-1 |
| M7.3 | Set royalty qualifying count and rate | Rule-10 |
| M7.4 | Set yearly cycle and low-contribution threshold | Rule-23, Rule-24 |
| M7.5 | Set the reference unit value | Rule-14 |
| M7.6 | Set which columns are ticked by default on extracts | Rule-33 |
| M7.7 | Set the whole-console backup schedule (off/daily/weekly/monthly) and retention count | Rule-43 |

### M8 — Access & Alerts

**Purpose:** Keep the system to the admin alone; ensure no month is missed. **M5 raises the alerts this module displays.**

| # | Function | Rules |
|---|---|---|
| M8.1 | One administrator login | Rule-29 |
| M8.2 | Lock the account after repeated failed attempts | Rule-29 |
| M8.3 | Show the outstanding-month banner on every screen | Rule-20 |
| M8.4 | Keep a notification list | Rule-20 |
| M8.5 | Support a PIN and complex password configured simultaneously; either credential logs in | Rule-29 |
| M8.6 | Back up the entire console on a schedule or on demand | Rule-43 |
| M8.7 | Restore the console from a backup file, on any machine, with deliberate confirmation | Rule-43 |

### M9 — Audit & Technical Logging *(architecture-introduced, cross-cutting)*

**Purpose:** Record who changed what, when, from what to what — the client-visible recording log — plus a separate, never-client-visible technical/diagnostic log. Every write path in M1, M2, M5, M7 that changes a saved value calls into this module inside the same transaction as the change itself.

### 1.1 Module dependency graph

```
M8 → M1, M2
M1 → M2, M3, M4
M2 → M3
M7 → M3, M6
M3 → M4, M5
M5 → M6, M8 (unlocks alert)
```

**Note on M2's dependency on M5 (amended 7 Aug 2026, CR-2):** M5 no longer gates M2 wholesale. It gates only *which* month M2 may write into — an ended-but-unclosed month stays writable, and the current month becomes writable when the close completes (Rule-36 as amended). M2 is never fully unavailable.

---

## 2. Functional requirements — FR-1 to FR-10

| ID | Description | Screen | Command |
|---|---|---|---|
| **FR-1** | Search by member **name**, **6-digit ID** or **phone number** (Rule-44). Results show the phone alongside name and ID. Selecting a result opens the member's detail with hierarchy shown **one depth only** (direct children) | Home/Search | `search_members` |
| **FR-2** | Hierarchy chart: each node shows exactly **name, ID, own Business Volume** — nothing else, never Total Business Volume. Applies equally to the full hierarchy view (FR-10) | Structure | `get_direct_children_chart` |
| **FR-3** | Member detail: name, phone, address, all Rewards detail, direct children (1 depth) with figures, Total Business Volume, leg count | Member Detail | `get_member_detail` |
| **FR-4** | Add member: name, phone, email (optional), Reference ID (mandatory), address, remaining basic fields; assigns a unique 6-digit ID on save | Add Member modal | `add_member` |
| **FR-5** | Admin enters Business Volume directly, nothing else, up to 2 decimals | BV Entry | `record_entry` |
| **FR-6** | All settings from [02](02-business-rules.md) §6 editable here | Settings | `get_settings`/`update_settings`, slab-row commands |
| **FR-7** | Manual monthly reset, gated on a confirmed backup | Monthly Close wizard | `begin_close`, `confirm_backup_and_close` |
| **FR-8** | Three exports (monthly, yearly average, low-contribution), configurable columns | Reports | `export_*` |
| **FR-9** | One administrator account; failed-attempt lockout mandatory | Setup wizard, Login | `setup_first_run`, `login` |
| **FR-10** | Full hierarchy view: the whole structure from the top member, every branch expanded at once, in a **separate read-only window** that draws once and never updates (Rule-45). Gated behind a confirmation naming the member count above 60 descendants | Structure → Full Hierarchy window | `get_direct_children_chart` (`full_tree: true`) |

⚠️ **FR-2's consequence, confirmed and accepted by the client:** because slab is driven by Total Business Volume and the chart shows only *own* Business Volume, a node can display a small own-figure while the member sits on a high slab. **The chart alone will not explain why anyone is on the slab they are on** — that explanation lives on the member detail screen (FR-3). This holds for FR-10's full view exactly as it holds for FR-2's one-branch chart: showing *more of* the tree never means showing *more per node*.

---

## 3. User needs — UN-01 to UN-31

All 31, each traced to the rules and requirements that satisfy it. Priority: 🟢 Must (28), Should (3 — UN-16, UN-24, UN-31).

| UN | Need | Priority | Rules |
|---|---|---|---|
| **UN-01** | A single, trustworthy record of who introduced whom | Must | Rule-1, Rule-30, Rule-37 |
| **UN-02** | A permanent member number that reveals nothing about join order or network size | Must | Rule-2, Rule-35 |
| **UN-03** | One record per real person, and a way back for those who return | Must | Rule-34 |
| **UN-04** | Position that cannot be rewritten after the fact | Must | Rule-37 |
| **UN-05** | History that survives a member leaving | Must | Rule-28 |
| **UN-06** | Structural guidance that advises rather than obstructs | Must | Rule-1, Rule-32 |
| **UN-07** | Activity recording with nothing in the way — one field, no mode, no conversion | Must | Rule-15, Rule-16 |
| **UN-08** | Figures that reconcile against a calculator, exactly | Must | Rule-22 |
| **UN-09** | Team volume that is complete and counted once | Must | Rule-5, Rule-6 |
| **UN-10** | A slab that reflects the whole team, not the individual | Must | Rule-3, Rule-7 |
| **UN-11** | A differential reward that is always fair in both directions | Must | Rule-8, Rule-9, Rule-11 |
| **UN-12** | Recognition for members who build at the top of the scheme | Must | Rule-10, Rule-11, Rule-25 |
| **UN-13** | Rewards held entirely apart from volume | Must | Rule-12, Rule-13 |
| **UN-14** | Numbers that are already correct when looked at — no recalculate control | Must | Rule-26 |
| **UN-15** | Finding a person immediately, by name, number or phone | Must | Rule-2, Rule-44 |
| **UN-16** | Seeing the shape of a branch — name, number, own Business Volume, nothing more | **Should** | (checklist Q11) |
| **UN-17** | A single screen that explains a member's reward | Must | Rule-6, Rule-12 |
| **UN-18** | A month that means one thing — a calendar month, unambiguously | Must | Rule-21 |
| **UN-19** | Impossible to lose a month by forgetting | Must | Rule-20, Rule-36 (amended — the undismissable alert now carries this need on its own, no longer reinforced by a total entry lock) |
| **UN-20** | A close that cannot destroy anything | Must | Rule-18, Rule-31 |
| **UN-21** | A permanent record of every month — correctable afterward, not frozen | Must | Rule-38, Rule-39 |
| **UN-22** | The month's figures in a spreadsheet, with chosen columns | Must | Rule-19, Rule-33 |
| **UN-23** | A yearly average that does not punish a late joiner | Must | Rule-23 |
| **UN-24** | Seeing who is not contributing personally | **Should** | Rule-24 |
| **UN-25** | Changing the scheme without asking anyone | Must | Rule-4, Rule-14, Rule-27 |
| **UN-26** | Sole and protected access | Must | Rule-29 |
| **UN-27** | Language that reveals nothing about the business | Must | [01](01-product-and-scope.md) §3 |
| **UN-28** | The whole console, safe and movable | Must | Rule-43 |
| **UN-29** | Finding a member by the number they are calling from | Must | Rule-34, Rule-44 |
| **UN-30** | Recording a purchase reported after the month has turned | Must | Rule-36 (amended) |
| **UN-31** | Seeing the whole structure at once, without slowing the console down | **Should** | Rule-45 |

---

## 4. Client-answered questions — RQ-1 to RQ-23

All 23 answered and confirmed. Where an answer differs from what was originally recommended, marked 🔷. Where it reverses an earlier decision, marked ⚠️.

| RQ | Question | Answer |
|---|---|---|
| **RQ-1** | Should the settings screen refuse a non-monotonic slab table? | 🔶 **No — client-accepted risk.** No software safeguard is built. See Rule-41 |
| **RQ-2** | How do inactive members behave in calculations? | 🔷 **Zero calculation effect — display-only.** See Rule-28 corrected, [06](06-decision-log-and-open-items.md) C5 |
| **RQ-3** | Can the root member be deactivated? | **No — refused, reason shown** |
| **RQ-4** | Where does a past-month extract come from? | **The permanent record**, for any closed month; live figures for the month in progress |
| **RQ-5** | What must the backup file contain? | **Every field of the permanent record, plus the slab table, royalty rate and qualifying count in force that month** |
| **RQ-6** | What counts as a "successful" backup? | **The internal retained copy** — verifiable with certainty. The download is a convenience on top |
| **RQ-7** | How is a wrong figure corrected? | 🔷 ⚠️ **An entry is editable at any time, including in an already-closed month.** See Rule-39 |
| **RQ-8** | Personal data — retention, notification, correction/removal | Retention **permanent**. Client takes own advice on notification. Correction supported; removal is out of scope (Rule-42) |
| **RQ-9** | Should there be a change log? | **Yes** — date/time, member affected, before/after, cause. See M9 |
| **RQ-10** | How does the client recover from a lost credential? | **One-time recovery codes**, issued at setup |
| **RQ-11** | Is the hard recording-lock stop definitely wanted, no grace period? | ⚠️ **Reversed 7 Aug 2026 by CR-2.** Originally: *"Confirmed — hard stop kept, no grace period."* Now: the ended-but-unclosed month stays open for entry indefinitely; only the **current** month is blocked, and only until the outstanding month is closed. There is still no timed grace window and no new setting — the grace lasts exactly as long as the month stays unclosed. See Rule-36 as amended and [06](06-decision-log-and-open-items.md) §5 |
| **RQ-12** | What does the reference unit value apply to? | **Labelled as the value of one Reward** |
| **RQ-13** | What does "reward detail" on the member screen mean? | **One line per direct child** — name, number, team figure, their slab, this member's slab, the difference, the resulting amount — then royalty lines, then the total |
| **RQ-14** | Can a past month be viewed on screen? | **Not now** — extracts only. Deferred as future scope |
| **RQ-15** | Is joining date captured automatically? | **Yes**, on the day added, editable afterward |
| **RQ-16** | Does an empty month produce a zero record? | **No record at all**, excluded from the yearly average |
| **RQ-17** | Are zero and negative figures accepted? | 🔷 **Neither.** Stricter than the recommendation (which proposed accepting zero). See Rule-16a |
| **RQ-18** | Does a mid-month settings change re-work the month? | **Yes, immediately**, behind a pre-save warning. Closed months never affected |
| **RQ-19** | Should the two backup copies be physically independent? | **Yes** — the downloaded copy goes to a genuinely separate medium |
| **RQ-20** | What happens to the retained backup when a closed month is corrected? | **The original is never touched.** A new, dated backup version is created and retained alongside it |
| **RQ-21** | Can an entry's date move it across a month boundary? | **Not yet** — the date stays within its own month. Cross-month moves deferred as an explicit future action |
| **RQ-22** | Should consent be captured in the system? | **Yes** — a mandatory checkbox and auto-captured date at Add Member. See Rule-40 |
| **RQ-23** | Should the whole console, not just one month, be backed up and portable? | **Yes** — see Rule-43, [04](04-technical-architecture.md) §9.5 |

---

## 5. Screen-by-screen specification

Drawn directly from `documents/design/ui-prototype-v2.html` (2,861 lines), the client-signed UI behaviour of record. Nine primary views (`VIEWS.home`, `.member`, `.structure`, `.entry`, `.close`, `.settings`, `.reports`, `.audit`) plus the close wizard, correction panel, and five auth-phase screens.

This section describes screen *content* — what each screen shows and does. For exact colours, type sizes, spacing, and every component's precise behaviour (buttons, pills, modals, the Structure Tree Node, Impact Summary, Restore Option List), see **[07-design-system.md](07-design-system.md)**.

### 5.1 Home / Search

**Purpose:** FR-1, UN-15. The default landing screen.

- **Header:** "Home" title, subtitle "Search any member, or scan today's standing", primary action **Add member**.
- **Stat row (3 cards):** Members (total count, inactive count in the footer); Entries this period (count, with the **name of the month being recorded into** in the footer — the outstanding month when one is awaiting close, otherwise the current month); On top slab (count, threshold label).
- **Search field:** single input, placeholder "Search by name, 6-digit member number or phone", live results below as the admin types.
- **Search results:** list of matching members with name, ID, **phone**, TBV, slab, status pill. Empty query → no results shown, not an error, not "all members" (V4.1). Matching follows Rule-44 — name substring, ID digits, or phone digits with a four-digit floor.
- **Month switcher (only when more than one month is outstanding):** a compact month control in the stat row, defaulting to the oldest outstanding month. It is **not rendered at all** in the ordinary case where a single month is in play. Selecting a month changes which period's figures the screen shows.
- **Members-by-slab card:** a horizontal bar per slab row, showing the count of members currently on that slab out of the total.
- Selecting a search result opens **Member Detail**.

### 5.2 Member Detail

**Purpose:** FR-3, UN-17. "The screen that explains a member."

- **Header:** member name, ID (monospace), status pill (Active/Inactive, colour + label).
- **Contact block:** phone, address, email if present, introducer name/ID, joining date, consent date.
- **Figures block:** own Business Volume, Total Business Volume, slab %, leg count (direct children).
- **Reward detail (RQ-13's confirmed layout):** one row per direct child — name, ID, their Total Business Volume, their slab, this member's slab, the difference, the resulting differential amount — then royalty lines (if qualifying), then the Rewards total.
- **Direct children list:** name, ID, own Business Volume, slab, status — one depth only (FR-3).
- Action to open the **Structure** chart rooted at this member.
- Action to record a Business Volume entry for this member — always available; it opens the entry screen, which names the month it will record into (Rule-36 as amended). It is never disabled by an outstanding month.

### 5.3 Structure (Hierarchy Chart)

**Purpose:** FR-2, UN-16. Visual tree of a chosen member's branch.

- **Node content — exactly three fields, never more:** name, ID (monospace), own Business Volume. **Never Total Business Volume** (FR-2's binding constraint).
- Root node visually distinguished (indigo border/fill per `DESIGN.md`); every other node neutral until interacted with.
- Connector lines drawn as thin, neutral-coloured SVG — never the accent colour, never thicker than node borders (the diagram's data must outweigh its scaffolding).
- Inactive nodes shown in a distinct colour plus a labelled pill (M4.5) — never colour alone.
- **Toolbar:** member search (same component and matching rules as Home), zoom out / zoom level / zoom in, **Fit width**, **Collapse all**, and **View full hierarchy** (FR-10 — see §5.3a).
- **A confirm-before-render gate** for an unbounded node count (**>60 descendants**) now lives on the **View full hierarchy** action, which is the only thing that draws an unbounded number of nodes. The one-branch-at-a-time chart on this screen needs no gate — its node count is bounded by one generation. (Originally a prototype UX addition with no source rule; given a home and a rule by CR-3 — see Rule-45, and [06](06-decision-log-and-open-items.md) LOW-1.)
- **Node figures** follow the same period as every other screen: the oldest outstanding month when one is awaiting close, otherwise the current month.

### 5.3a Full Hierarchy Window *(reached from Structure, "View full hierarchy")*

**Purpose:** FR-10, UN-31, Rule-45. The whole structure in one draw, in its own window, so the main console is never made to carry it.

- **Root:** always the **top member**, whatever the Structure screen is currently rooted at. The full view is a view of the network, not of a branch.
- **Size gate:** on activating the action, the descendant count is read first. Above **60 descendants** a confirmation names the exact count — *"This will draw 4,182 members in a new window. It may take a moment."* — with **Open** and **Cancel**. At or below 60 it opens immediately. Cancelling does nothing at all; no window is opened.
- **A separate top-level window**, not a modal, not a route in the main console. It is opened once, drawn once, and thereafter has no connection to the console: it does not refresh, does not poll, and holds no handle on live state. Closing it discards it.
- **Header:** the top member's name, the total member count, and an **"as at &lt;date, time&gt;"** stamp — so a printed or screenshotted copy always says when it was true.
- **The chart:** the same top-down layout and the same node component as §5.3, with **every branch expanded simultaneously**.
- **Node content — the same three fields, unchanged:** name, ID (monospace), own Business Volume. **Never Total Business Volume.** FR-2's constraint is not relaxed by showing more of the tree.
- Inactive nodes keep the distinct colour plus the labelled pill (M4.5). Connectors stay thin and neutral (§5.3).
- **Toolbar:** zoom out (to **10%**, far wider than the main chart's range, because a whole network needs to be taken in at once), zoom in (to 150%), **Fit width**, a **search box** that highlights the matching member and scrolls them into view, and **Print**.
- **Read-only.** No node opens a member detail, nothing can be recorded or edited, and no action in this window changes anything in the console.
- **Theme:** inherits the console's light/dark theme as at the moment it was opened.
- **Figures shown** are those of the same period as the Structure screen (the oldest outstanding month, else the current month), fixed at the moment of drawing.
- **Empty structure:** if the top member has nobody beneath them, the window opens showing the single root node and states plainly that there is nothing beneath it — not an error.

⚠️ **Accepted scale limit (TR-7):** a top-down chart's width grows with the number of *leaves*, not with depth. At the NFR-2 ceiling of 25,000 members the canvas is tens of thousands of pixels wide, and a print spans many pages. The client chose this layout over a width-stable indented outline, with the 10% zoom floor, fit-width, search-and-scroll and the size gate as the agreed mitigations. See [04](04-technical-architecture.md) §11.

### 5.4 Business Volume Entry

**Purpose:** FR-5, Rule-15, Rule-16, Rule-36 (as amended). UG-1's daily action, target under 15 seconds (SC-5).

- **Search:** by name, 6-digit ID or phone, same search component and matching rules as Home (Rule-44).
- **Selected member:** name, ID, current slab shown for context.
- **Recording-month note (Rule-36, M2.3):** the form is always headed by the month it is recording into. When a month is awaiting close, it names that month and states plainly that the current month unlocks once it is closed — *"Recording into June 2026. August entries can be recorded once June is closed."* When nothing is outstanding, it simply names the current month.
- **Month selector (only when more than one month is outstanding):** a `<select>` of the outstanding months, defaulting to the oldest. **Not rendered at all** when there is only one month to record into — which is the ordinary case. Changing it changes the date bounds below.
- **Date field:** bounded to the recording month — first day to last day of that month, capped at today when the recording month is the current month. Defaults to today when recording into the current month, and to the last day of the month otherwise.
- **Single amount field:** Business Volume amount, numeric, up to 2 decimal places. **No currency field, no mode toggle, no second field on this fast path** (Rule-16, UN-07).
- **No locked state.** The entry form is always available. What varies is *which month* it writes into — never *whether* it can be used. (Superseded 7 Aug 2026: the entire form was previously replaced by a locked empty state whenever a reset was outstanding.)
- **Validation:** amount must be `> 0` (Rule-16a — zero explicitly refused, not just negative), ≤ 2 decimals; date must fall within the recording month (V2.6). A current-month date while an earlier month is outstanding is refused, naming the blocking month (V2.7). Save disabled until valid.
- On save: the ancestor chain recalculates immediately **within the month recorded into**; every affected figure is visible on screen with no further action (Rule-26). An entry into an outstanding month recalculates that month and touches no other.
- **This period's entries** list below the form shows the entries of the recording month.

### 5.5 Correction Panel *(reached from BV Entry, "Correct a closed month")*

**Purpose:** M2.4/M2.5, Rule-39.

- Search and select the entry to correct (by member — name, ID or phone per Rule-44 — and by month).
- **If the entry belongs to a closed month:** an explicit on-screen warning is shown *before* the change is applied — "Editing a record recalculates the affected chain and writes a new snapshot version — the original record is never overwritten."
- Same amount/date validation as BV Entry, scoped to the entry's own period bounds.
- On save: the affected chain recalculates; if the period is closed, a new `monthly_snapshots`/`backups` version is written; the original version is never touched.

### 5.6 Monthly Close — status page and wizard

**Purpose:** FR-7, M5, Rule-17–21, Rule-36, Rule-38.

- **Status page:** lists every outstanding month (oldest first), each with its own "Close this month" action — only the oldest is enabled.
- **Close wizard**, once triggered:
  1. **Confirmation step** — names the month being closed, explicitly, unambiguously.
  2. **Backup step** — generates and verifies the internal backup; prompts for the external-medium copy location.
  3. **Failure path:** if the backup fails or is cancelled, the wizard aborts entirely — nothing is zeroed, the alert stays up, and the month stays open for entry while the current month stays blocked (Rule-36 as amended).
  4. **Commit step** (only reachable after a verified backup): writes the permanent snapshot for every member, then zeroes every live figure.
  5. **Completion:** the alert clears for this month; if an older month is still outstanding, its alert takes over and the process repeats.

### 5.7 Settings

**Purpose:** FR-6, M7, all 16 settings from [02](02-business-rules.md) §6.

- **Slab table section:** editable rows (threshold, percentage), add/remove controls. The **last remaining row's remove control is disabled**, with an explanatory `aria-label` and an on-screen hint (LOW-2, built). Duplicate-threshold save attempts are refused outright, before any warning is offered.
- **Royalty section:** qualifying-child count, royalty rate.
- **Structure guidance section:** hierarchy depth, level 2/3/4 widths — explicitly labelled as guidance, never enforced.
- **Reporting section:** yearly cycle start/end, low-contribution threshold, default export columns.
- **Reference unit value** field — labelled per RQ-12, display-only, never read elsewhere.
- **Access section:** session inactivity timeout.
- **Backup schedule card** (RQ-23): off/daily/weekly/monthly segmented control, retention count, "Back up now" action.
- **Restore card** (RQ-23): lists retained backups of every kind (`period_close` labelled by month, `scheduled`/`manual` labelled by when taken), a "Restore from a file…" action, each entry selectable via the shared restore-option-list component.
- **Mid-period recalculation warning (RQ-18/V7.6, variant C — built and approved):** fires **only** on a Slab table or Royalty save. Names the open month, states closed months are unaffected, shows **Rewards before → after**, and lists the members actually affected (slab moves for a slab-table change; who starts/stops earning royalty for a royalty change). Cancel is a true no-op — nothing is saved, typed values remain exactly as entered. A duplicate-threshold refusal happens *before* this warning is ever offered. The other three sections (structure guidance, reporting, reference value) save silently — they change nothing already calculated.

### 5.8 Reports

**Purpose:** FR-8, M6.

- **Monthly data card:** period selector, column picker (four defaults pre-ticked and un-removable, plus the full optional list from Rule-33), export action.
- **Yearly average card:** cycle bounds from settings, export action. Extract shows both Total Business Volume and own-Business-Volume averages, each with its month count displayed alongside (Rule-23).
- **Low-contribution card:** threshold (default 100, overridable), export action. Filters on **own** Business Volume yearly average (Rule-24).
- **Closed month snapshot card:** re-download any past closed month's data as `.xlsx` — maps to `redownload_backup`, always the latest version. Used when entries in that month have since been corrected, or simply for another copy.
- All exports open correctly in a standard spreadsheet application; every export names the period, never a member, in its filename.
- Inactive-member rows shown in the same distinct colour as on screen, alongside the textual active/inactive column (M6.5).
- Empty state (e.g. low-contribution report with nobody below threshold) shown plainly, not as an error.

### 5.9 Audit

**Purpose:** NFR-5, M9. Read-only.

- Chronological list: date/time, member affected, field, value before, value after, cause (`entry`/`edit`/`correction`/`settings_change`/`period_close`/`manual_backup`).
- Filterable by member name, ID or phone.
- No entry is ever edited or removed.

### 5.10 Authentication screens

- **Setup (first-run):** step 0 — choose PIN or password mode (segmented control), enter and confirm the credential. Step 1 — recovery-code reveal, shown once, with a mandatory "I have saved this recovery code somewhere safe, outside this console" checkbox gating the "Enter the console" action. A plain link, "Restore from a backup file instead," offers the alternative path for a brand-new install with an existing backup — not a competing button, no separate welcome/choice screen.
- **Login:** PIN or password entry per the configured credential(s); either authenticates if both are set. Wrong credential shows a generic "incorrect" message (never revealing which part was wrong) and an attempts-remaining count. After 5 consecutive failures: a lockout screen with a live countdown (see [06](06-decision-log-and-open-items.md) O4 for the ladder beyond the first threshold).
- **Locked (inactivity):** a lock screen requiring re-authentication to resume; the encryption key is genuinely dropped from memory, not merely hidden behind an overlay.
- **Recovery:** enter a one-time recovery code, set a new credential; all prior codes are invalidated and a fresh set issued.
- **Data recovery (db-error, LOW-3, design D):** a full-screen state shown in place of sign-in when the encrypted database cannot be opened. States plainly that nothing has been lost, lists the most recent retained backups by the month/occasion each holds (marking corrected months), offers restore-from-backup and retry, and states that anything recorded after the chosen backup will need entering again. The same screen, reworded, serves the voluntary first-run restore path — no internal restore-points list is shown there, since a brand-new machine has none of its own yet.

### 5.11 Persistent, cross-screen elements

- **Outstanding-month banner:** appears on **every** screen once any month has ended without being closed. Names the month(s), states that entries dated in the outstanding month can still be recorded, and names the month that unlocks on close — *"June 2026 has ended and is awaiting close. You can still record entries dated in June. August entries unlock once June is closed."* **No dismiss control of any kind, not even a disguised one** — clears only on a completed close (Rule-20). It carries the close action.
- **Sidebar navigation:** Home, Structure, Business Volume Entry, Settings, Reports, Audit — plus theme toggle, lock session, sign out in the footer.
- **Notification list:** mirrors the outstanding-month alert as a persistent entry, not a dismissable toast.

---

## 6. Validation rules — V1.1 to V8.5

The complete set of 49, organised by module. "On failure" states the exact behaviour — refuse outright, warn-and-allow, or offer an alternative.

### M1 — Structure

| # | Rule | On failure |
|---|---|---|
| V1.1 | Name is required | Refuse, name the field |
| V1.2 | Phone is required and must not already exist (active or inactive) | Refuse; if inactive match, name the person and offer reactivation |
| V1.3 | Reference ID is required and must match an existing, active member | Refuse with a clear message |
| V1.4 | Email is optional; if given, must be a valid address | Refuse the field only |
| V1.5 | The assigned member number must not already be in use | Choose another automatically |
| V1.6 | A second top-level member can never be created | The route is unavailable after setup |
| V1.7 | Exceeding a level width or the maximum depth | **Warn, allow the user to continue** |
| V1.8 | Any attempt to change an existing member's introducer | **Refuse outright**, state the reason |
| V1.9 | Adding a member with consent unticked | Save disabled until ticked |

### M2 — Entry

| # | Rule | On failure |
|---|---|---|
| V2.1 | A member must be selected before a figure can be recorded | Refuse |
| V2.2 | The figure must be numeric, at most 2 decimal places | Refuse, state the format |
| V2.3 | Recording into the **current** month is refused while any earlier month is outstanding **[AMENDED 7 Aug 2026, CR-2 — was "recording is refused entirely"]** | Refuse, name the month that must be closed first |
| V2.4 | Neither zero nor a negative figure is permitted | Both refused (Rule-16a) |
| V2.5 | The date field is present on the recording screen, pre-filled and bounded to the recording month **[AMENDED 7 Aug 2026, CR-2 — was "never appears on the initial recording screen"]** | Defaults to today when recording into the current month, to the month's last day otherwise; never requires a keystroke on the fast path |
| V2.6 | An entry's date must fall within the month being recorded into | Refuse, name the month's bounds |
| V2.7 | Recording into an already-closed month is not offered on this screen | Direct to the correction panel (Rule-39) |

### M3 — Calculation

| # | Rule | Note |
|---|---|---|
| V3.1 | Own Business Volume is always included in own Total Business Volume, without exception | Structural — cannot fail |
| V3.2 | Only direct children contribute a differential term | Structural |
| V3.3 | Only direct children are counted and paid for royalty | Structural |
| V3.4 | 🔶 Nothing prevents a non-monotonic slab table | **By client decision, not built** — see Rule-41 |
| V3.5 | Inactive status has **no effect on any calculation** | See Rule-28 corrected |

### M4 — Viewing

| # | Rule | Note |
|---|---|---|
| V4.1 | A search returning nothing says so clearly, not an empty screen | |
| V4.2 | Member detail and home search show the direct team only, one level deep | Unaffected by FR-10, which is a separate window, not this screen |
| V4.3 | Reward detail = one line per direct child (name, number, their team figure, their slab, this member's slab, the difference, the amount), then royalty lines, then the total | Per RQ-13 |
| V4.4 | Phone matching engages only at 4 digits or more; below that only name and ID are matched | Not an error — the query simply does not match on phone (Rule-44) |
| V4.5 | The full hierarchy view is gated behind a confirmation naming the member count above 60 descendants | Cancelling opens nothing at all (Rule-45) |

### M5 — Monthly close

| # | Rule | Note |
|---|---|---|
| V5.1 | Only the oldest outstanding month may be closed | |
| V5.2 | Nothing is cleared until the backup is confirmed | |
| V5.3 | Nothing is cleared until the permanent record is written | |
| V5.4 | The confirmation screen must name the month being closed | |
| V5.5 | The retained in-system copy is the gate; the download is a convenience | Per RQ-6 |
| V5.6 | A month with no entries produces no record at all, excluded from the yearly average | Per RQ-16 |
| V5.7 | Editing a closed-month entry shows an explicit on-screen warning naming that month, before the change is accepted | Per M5.9 |

### M6 — Reporting

| # | Rule | Note |
|---|---|---|
| V6.1 | The four default columns are always present and cannot be removed | **See [06](06-decision-log-and-open-items.md) O1 — possible conflict with Rule-19's five-field wording** |
| V6.2 | Every yearly average is shown with the month count it is based on | |
| V6.3 | The low-contribution threshold must be a positive number | Refuse otherwise |
| V6.4 | A past month's extract reads from the permanent record — a re-extract after a correction automatically reflects it | Per RQ-4 |
| V6.5 | The backup file carries the threshold table in force that month | Per RQ-5 |

### M7 — Settings

| # | Rule | Note |
|---|---|---|
| V7.1 | Thresholds must be positive numbers | Refuse |
| V7.2 | Percentages must be between 0 and 100 | Refuse |
| V7.3 | At least one slab row must exist | Refuse the removal |
| V7.4 | The royalty qualifying count must be a positive whole number | Refuse |
| V7.5 | 🔶 Percentages must rise as thresholds rise | **By client decision, not built** — Rule-41 |
| V7.6 | A settings change applies immediately and re-works the month in progress, with a warning shown before saving | Closed months never affected |

### M8 — Access

| # | Rule | Note |
|---|---|---|
| V8.1 | Repeated failed attempts lock the account | Lock, state clearly |
| V8.2 | The alert cannot be dismissed by navigating away, logging out, or acknowledging it | |
| V8.3 | Recovery codes, issued at setup and kept by the client, are the route back in | Per RQ-10 |
| V8.4 | Setting a password does not require removing the PIN, and vice versa; either credential set unlocks the account | Per M8.5 |
| V8.5 | Restoring the console always names what will be replaced and requires deliberate confirmation; the console backs up its own current state first | Per M8.6/M8.7 |
