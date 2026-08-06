---
target: documents/design/ui-prototype-v2.html
total_score: 28
max_score: 40
na_heuristics: 
p0_count: 1
p1_count: 2
timestamp: 2026-08-06T14-07-55Z
slug: documents-design-ui-prototype-v2-html
---
Method: dual-agent (A: a17684b2c736c164a · B: a227c2aa734dd8415)

## Design Health Score

| # | Heuristic | Score | Key Issue |
|---|-----------|-------|-----------|
| 1 | Visibility of System Status | 3 | Backup step shows only a spinner + one checklist row — no filename/size/destination on the highest-stakes gate in the app. |
| 2 | Match System / Real World | 4 | Solid — export filenames, hint copy, and business-rule text are all client-grounded, not generic (e.g. "Differential and royalty never pay on the same leg"). |
| 3 | User Control and Freedom | 2 | No `Escape`-key handler anywhere (grep-confirmed). Close wizard's "Continue" auto-finalizes via a 700ms `setTimeout` with no further confirm gate. |
| 4 | Consistency and Standards | 2 | Detector-confirmed: 11 off-scale border-radius values and ~25 off-scale font-size values against DESIGN.md's documented 2-step radius / 5-role type scale, plus one undocumented second shadow (`.segmented button.active`) contradicting the system's "one shadow" rule. Adjusted down from the design review's inline read once the deterministic count came in. |
| 5 | Error Prevention | 3 | Phone-duplicate → reactivate flow (real-time) is excellent. Slab-table's known non-monotonic-% risk gets one skimmable `.hint` line. |
| 6 | Recognition Rather Than Recall | 4 | Solid — breadcrumbs, selected-member card with "Change" link, sticky settings nav. |
| 7 | Flexibility and Efficiency | 2 | Zero `.focus()` calls anywhere in the file (grep-confirmed) — the amount field isn't auto-focused after picking a member, adding a click to the product's stated 15-second flagship action. |
| 8 | Aesthetic and Minimalist Design | 3 | Visually clean at a glance, but the browser-overlay detector pass caught what static reading missed: `flat-type-hierarchy` (six sizes 11.5–18px clustered at only 1.6:1 ratio) and `tiny-text` (11.5px body copy at 8 locations) — the type scale is muddier underneath than the surface reads. Adjusted down from the design review's 4/4. |
| 9 | Error Recovery | 3 | Named, specific messages (phone-dupe banner names the exact member). Backup-step copy never says in plain terms where the file actually went. |
| 10 | Help and Documentation | 2 | Appropriately near-absent for a "won't read documentation" user — but where copy is the only help available, it under-delivers (see #9). |
| **Total** | | **28/40** | **Good (low end)** |

## Design Specificity Verdict

**LLM assessment:** Not a generic admin dashboard wearing indigo. The vocabulary discipline is enforced in the code, not just labels: export filenames are literally `member-rewards-monthly-2026-08.xlsx`; the entry-amount hint reads "Up to two decimals · no currency field"; the introducer field is hard-disabled with "Cannot be changed after onboarding — a member's introducer is permanent" — a direct restatement of the product's no-override rule. Mock data is India-specific (DD/MM/YYYY, `en-IN` grouping, Indian mobile prefixes/localities) rather than placeholder Lorem. This is a design built from the spec, not despite it. **Verdict: specific, not generic.**

**Deterministic scan:** `detect.mjs --json` against the file (DESIGN.md/design.json context loaded): **exit code 2, 49 findings** — 48 advisory, 1 non-advisory (`layout-transition`, animating `width` instead of `transform: scaleX()` on the bar-chart fill). Breakdown: `design-system-font-size` ×36 (of which ~11 are arguably false positives — DESIGN.md's YAML type scale omits two steps its own prose documents, a DESIGN.md authoring gap rather than a prototype defect; the remaining ~25 are real drift), `design-system-radius` ×11 (all genuine — none of the 11 values appear anywhere in DESIGN.md), `design-system-color` ×1 (the undocumented second shadow noted above). Mobile-viewport CLI pass was unavailable (Puppeteer not installed in this sandbox; static-markup engine ignores `--viewport`) — not silently skipped, confirmed unavailable and substituted with a browser-overlay pass instead.

**Visual overlays:** Overlay injection succeeded via a Playwright fallback (the skill's prescribed live-server-serves-the-file flow 404'd — that server variant expects a file registered through its own live-inject step, not arbitrary static serving — documented and abandoned after two tries per protocol). The injected `detect.js` overlay found `tiny-text` and `flat-type-hierarchy`, genuinely complementary to the CLI scan (computed-style checks the static engine can't do). 14 screenshots were captured (desktop 1440×900 + mobile 390×844) across setup, home, structure (both toggle states), and settings.

## Overall Impression

The system holds together as a genuine, specific product — the copy, the vocabulary, the calculation-aware UI states are all doing real work, not decoration. What's missing is emotional calibration at the one moment that matters most (the irreversible monthly close reads exactly like routine navigation) and a set of small, structural gaps — no focus management anywhere, no label/input association anywhere, a design-token scale that's drifted more than the visual read alone would suggest — that are individually minor but collectively explain why a 28/40 "Good" score sits well below the visual polish would predict. The single biggest opportunity: the close wizard is the product's one truly irreversible, highest-stakes action, and right now the interface is the only safety net for an operator who will never read documentation — it needs to *feel* as weighty as it actually is.

## What's Working

1. **Business-rule copy is load-bearing, not decorative.** The differential/royalty explanation, the "no override, ever" introducer lock, and the "no currency field" hint encode the actual calculation engine and product exclusions directly into UI text — this is what makes it read as a real product.
2. **The phone-duplicate → reactivate flow** turns an abstract data rule ("deactivate, never delete") into a concrete, real-time recovery affordance a non-technical operator will discover exactly when he needs it, not something he has to remember exists.
3. **Snapshot versioning is visible, not just true.** Reports' closed-month dropdown labels "v2 (corrected)"; the abstract "history is reconstructable" guarantee is checkable on screen, matching the product's own stated success criterion directly.

## Priority Issues

**[P0] Monthly close's commit step doesn't communicate irreversibility.**
Why it matters: this is the single highest-stakes, most irreversible action in the product, for a user who will never read documentation — the screen is his only safety net, and it currently reads no differently than routine navigation. No wizard copy ever says "irreversible" or "cannot be undone" (grep-confirmed); the final commit is an ordinary `btn-primary` "Continue" that auto-finalizes via a 700ms `setTimeout` with no further confirm gate.
Fix: state plainly, in step-0 or step-1 copy, that this cannot be undone; give the final commit control deliberately more visual weight than a routine "Continue."
Suggested command: `/impeccable harden`

**[P1] No focus management anywhere in the app.**
Why it matters: zero `.focus()` calls exist in the entire file (grep-confirmed). Volume Entry is explicitly the product's most frequent action with a stated 15-second, one-field target — this silently adds a click to every one of potentially hundreds of monthly entries, and leaves keyboard/screen-reader users with no indication where focus went on any modal open.
Fix: `.focus()` the amount input the instant a member is selected on Volume Entry; focus the first field on every modal open.
Suggested command: `/impeccable polish`

**[P1] Zero label/input association anywhere in the app.**
Why it matters: all `<label>` elements are siblings of their inputs, not wrapping them, and there is not a single `for="..."` attribute in the file (grep-confirmed) — a screen reader has no programmatic link between a field's label and its input, on the most-used form in the product and during the highest-anxiety recovery flow (PIN reset).
Fix: add matching `id`/`for` pairs in the shared field-rendering helper — a small, centralized change fixes it everywhere at once.
Suggested command: `/impeccable harden`

**[P2] "Full hierarchy" structure view doesn't scale to the product's real member counts.**
Why it matters: independently measured by both assessments at the identical **25,484px** wide for this prototype's 201 mock members (Assessment A via its own Playwright pass; Assessment B via direct `scrollWidth` measurement of `.tree-scroll`) — the product's real range is 500–5,000 members, so at scale this becomes hundreds of thousands of pixels of horizontal canvas with no warning before the operator triggers it. At the current mobile-width equivalent (a narrowed desktop window, not phone support — see Minor Observations), the visible scroll window is already ~90px against 25,484px of content, a ~283:1 ratio with no scroll affordance beyond static caption text.
Fix: cap full-hierarchy rendering to a bounded depth/count with an explicit warning before rendering past it, or replace the flat row layout with a collapsible/zoomable rendering.
Suggested command: `/impeccable optimize`

**[P2] Design-token drift, detector-confirmed.**
Why it matters: 11 border-radius values and ~25 font-size values appear nowhere in DESIGN.md's documented scale (2 radius steps; a 5-role type scale) — genuine accumulated drift from iterative prototyping, not aesthetic nitpicking; each value is a concrete file:line the detector already located.
Fix: sweep the 11 radius outliers (lines 86, 106, 223, 230, 307, 351, 367, 382, 404, 412, 414) to the nearest system step or promote a genuinely recurring value (e.g. the small-control `5px` cluster) into a documented third radius step; do the same triage for the font-size outliers, and add the two steps DESIGN.md's own prose already documents (13px card-title, 28px entry-amount) into its YAML scale so future detector runs don't flag intentional values.
Suggested command: `/impeccable polish`

## Persona Red Flags

**Alex (Power User):** No global keyboard shortcuts (no "/" to jump to search, no command palette). No sortable table headers anywhere in the file — Home's search results, Reports' preview tables, and the Audit log are all static; Alex can't reorder by Total Business Volume to eyeball network extremes. No bulk/CSV entry path despite a 500–5,000-member network — every Business Volume figure is one search-select-type-save cycle.

**Sam (Accessibility-Dependent):** Zero `for`/`id` label associations app-wide — every field in setup, recovery, and daily entry is unlabeled for assistive tech. The PIN keypad has no keyboard-digit binding — six Tab-and-Enter cycles per login, every day, with no way to type digits directly. Zero focus management on any modal open. The password-visibility toggle's `aria-label="Show"` is hardcoded and never updates to "Hide" — the icon changes, but the accessible name lies about the button's current state.

**Siddharth (the sole low/moderate-skill operator, confirmed by PRODUCT.md to never read documentation):** On his daily action (search + record BV), the amount field isn't auto-focused after picking a member — nothing signals where to click next. On his highest-stakes monthly action (closing the month), the backup step's own copy — "The copy is downloaded here, and retained permanently inside the console on a separate medium" — never says in plain terms where his backup actually went, and the prototype's `generateBackup()` is a pure `setTimeout` producing no actual file, unlike the real `.xlsx` exports two clicks away in Reports. He's told a backup exists with nothing he can point to and verify.

## Minor Observations

- Two different "find a member" interaction shapes for the same task: Home renders results inline/permanently, while Entry/Correction/the reference picker render a floating dropdown.
- `.hint.warn` (`#d97706` on white) measures ≈3.2:1 contrast, below WCAG AA's 4.5:1 for body text — it's the exact copy explaining why Save is disabled on the flagship Volume Entry form.
- A second, undocumented shadow exists on `.segmented button.active` (`0 1px 2px rgb(0 0 0 / 0.06)`), contradicting DESIGN.md's "one shadow in the system" rule — either fold it into DESIGN.md as a legitimate small "active-toggle lift" tier, or drop it to strictly enforce flat-by-default; worth a deliberate call either way, not a silent drift.
- **Mobile-viewport measurements are real but likely not a defect**: at a 390px width the fixed 236px sidebar consumes 60% of the viewport, leaving 154px for content, and zero width-based `@media` queries exist in the file. However, PRODUCT.md explicitly confirms this is a desktop-only application with phone/tablet support "never discussed" and out of scope — so this isn't a missed requirement. The one place it could still matter: a Tauri app runs in a resizable native desktop window, so a user narrowing the window on a small laptop screen (not phone-width, but below 1440px) would hit a version of the same squeeze. Worth a light look during `/impeccable adapt` at moderate desktop widths only, not phone breakpoints.
- The close wizard fakes a 700ms "Writing the permanent record…" spinner even though the product's own principle is "the screen is never stale" / calculation is instant — manufacturing that pause risks training the operator that "long pause = working," which will misfire the day something actually hangs.
- Toast auto-dismiss visually overlapped form content in one captured Settings screenshot — confirmed to be a timing artifact of automated fast-forwarding past the toast's real 3.4s window, not a defect an interactive user would hit at normal speed.

## Questions to Consider

1. The prototype already builds a real, byte-correct `.xlsx` for every export — so why is the one moment gated on a "confirmed backup" the one place that produces no actual file? Is the backup gate meant to read as a genuine transactional precondition, or does it currently just look like one?
2. At 201 mock members the full-hierarchy tree is already 25,000+ pixels wide. Has this view ever been exercised at the client's real 500–5,000-member range, or does "Full hierarchy" quietly assume nobody will actually reach for it at scale?
3. If the interface is Siddharth's entire safety net for an action he can never undo, what would it look like to make the close wizard's final step feel as weighty as closing a month actually is — without adding a step he'd learn to click through on autopilot?
