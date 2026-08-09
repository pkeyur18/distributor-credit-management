# 07 — Design System

The visual specification for every screen described in [03](03-functional-specification.md) §5. Colour tokens, typography, layout, elevation, shape, and every component's exact behaviour — restated in full from `documents/design/DESIGN.md`/`ui-theme.md`, the approved reference for the real Tauri build. Where 03 says *what* a screen shows, this file says *how it looks*.

**Creative north star: "The Single Ledger."** A private operating console for one person, read like a ledger every day — dense, numeric, built to be trusted at a glance rather than admired. There is exactly one accent colour and it is spent carefully: the action that matters (primary buttons), the place the operator currently is (active nav), the thing that just gained focus (inputs, rings) — never as decoration. Nothing here uses the vocabulary of a commercial platform, and that restraint extends to the visual language too: no gradients, no marketing hero type, no rounded-SaaS softness.

---

## 1. Colour tokens

A near-monochrome slate system with a single indigo accent. Both light and dark values are given — see §7's theme note for how the two coexist.

| Token | Light | Dark | Used for |
|---|---|---|---|
| **Accent** | `#4f46e5` | `#6366f1` | Primary buttons, links, active sidebar item, focus rings, selection colour, slab/status chips that need to read as "the system's own." A small minority of any given screen. |
| **Accent, weak** | `#eef2ff` | `#1e1b4b` | The tint under the accent — active nav background, focus-ring halo, slab pill fill, avatar backgrounds. Always the same hue as Accent, just diluted — never a second brand colour. |
| **Slate background** | `#f8fafc` | `#0f172a` | The page itself, and the recessed background inside inputs/tracks/segmented controls. |
| **White surface** | `#ffffff` | `#1e293b` | Every raised surface — cards, table rows, inputs, modals, the sidebar. |
| **Slate border** | `#e2e8f0` | `#334155` | The 1px hairline that does almost all separation work — between cards, table rows, sidebar sections, form fields. |
| **Ink** | `#0f172a` | `#f1f5f9` | Primary text and the strongest UI marks (card titles, values). |
| **Slate, muted** | `#64748b` | `#94a3b8` | Secondary text, labels, table headers, breadcrumbs, placeholder-weight copy. |
| **Ledger green** (success) | `#059669` | `#10b981` | Status only, never decorative — the "Active" pill, completed checklist steps, success toasts. |
| **Amber** (warning) | `#d97706` | `#f59e0b` | The outstanding-month banner and locked states. Amber, deliberately not red — it names a required action, not an error. |
| **Amber, text** | `#92400e` | `#f59e0b` | A darker step reserved for amber **text** on a light surface (validation-warning copy, disabled-state explanations) — `#d97706` itself falls short of 4.5:1 contrast on white. The icon/border/banner-fill still uses the plain warning token; only small warning text on a light background switches to this step. |
| **Red** (danger) | `#dc2626` | `#ef4444` | Validation errors, the "Inactive" pill, destructive actions (danger buttons, delete icon-button hover). |

Each status colour carries a `-weak` tint (success `#ecfdf5`/`#052e21`, warning `#fffbeb`/`#3a2a06`, danger `#fef2f2`/`#3d1414`) used as the pill/banner/note fill, with the full-strength colour reserved for text and small marks (dots, icons) inside it.

### Named rules

**The One Accent Rule.** Indigo is the only brand colour in the system. There is no secondary or tertiary accent — a second colour need is solved with a status colour or a neutral, never a new hue.

**The Colour-Plus-Label Rule.** No status is ever conveyed by colour alone. Active/Inactive, locked/unlocked, and every pill in the system pairs its colour with a text label — this is a hard requirement (NFR-8, M4.5/M6.5), not a preference. Directly implements Rule-28's inactive-member colour-coding requirement.

---

## 2. Typography

**Body font:** system UI stack in the prototype (`-apple-system, BlinkMacSystemFont, "Segoe UI", system-ui, sans-serif`) for CSP-safety in a sandboxed preview. **The real Tauri build bundles Inter locally** — no web fonts, ever, consistent with the offline constraint (NFR-14). Treat the system stack as a placeholder for Inter, not a final decision.

**Label/mono font:** `ui-monospace, SFMono-Regular, Menlo, Consolas, monospace` — used only for six-digit member numbers and other identifiers, and for `font-variant-numeric: tabular-nums` on every other figure.

**Character:** plain and functional throughout — one system stack carries every role via size and weight, not typeface changes. Deliberate: the product's discretion requirement (UN-27) extends to typography too. Nothing here should read as "designed" in a way that draws attention to itself.

| Role | Weight | Size | Line-height | Tracking | Used for |
|---|---|---|---|---|---|
| **Headline** | 650 | 20px | 1.3 | −0.015em | Page titles only — one per screen, top of the content column |
| **Title** | 650 | 15px | 1.4 | normal | Modal/section headers |
| **Title, small** | 650 | 13px | 1.4 | normal | Card titles — one step down from Title |
| **Body** | 400 | 14px | 1.5 | normal | Default for every paragraph, table cell, form value |
| **Label** | 650 | 11px | 1.3 | 0.045em | Table column headers, stat-card labels, sidebar section labels — always muted-coloured, never full ink |
| **Numeric** | 650 | 22px | 1.2 | −0.01em | Stat-card values — one of the few genuinely large numbers, sized to read before anything else on the screen |
| **Numeric, large** | 650 | 28px | 1.1 | −0.01em | The Business Volume entry field alone — the single largest text in the system, reserved for the one figure entered on the product's most frequent action |
| **Caption** | 400 | 12px | 1.4 | normal | Secondary explanatory text — card subtitles, chart row labels/counts, the structure legend, small inline notes under a form or wizard step. The one step that's regular weight rather than a weight/tracking extreme |

### Named rule

**The Tabular Rule.** Every number that could sit in a column — table cells, stat values, entered figures — uses `font-variant-numeric: tabular-nums`. Digits must line up; this system is read as a ledger, not prose.

---

## 3. Layout

A fixed 236px sidebar plus a fluid content column (`grid-template-columns: 236px 1fr`), sticky at full viewport height. Content is padded 32px horizontally, 20px above and 40px below, with a sticky top bar. Density is deliberately high for a single power user in daily use: **~40px table rows**, 4px as the smallest spacing increment, most gaps at 8/12/14px, section-level spacing at 18/24/32px. Narrow, single-purpose screens (the Business Volume entry form) cap at 620–640px and centre themselves rather than stretching to the full column — width is earned by content, not filled by default.

---

## 4. Elevation & depth

**Flat by default.** Cards, table rows, the sidebar, and form fields carry a 1px hairline border and **no shadow** — separation comes from the border and the background-shift between page and surface, not simulated lift. Shadow is reserved for things genuinely temporary and floating above the page.

| Use | Shadow |
|---|---|
| **Modal/overlay** | `0 20px 48px -12px rgb(15 23 42 / 0.28), 0 4px 12px -4px rgb(15 23 42 / 0.12)` (dark: alpha raised to 0.55/0.35 against pure black) — modals, toasts, the search-results dropdown. Nowhere else. |
| **Control-lift** | `0 1px 2px rgb(0 0 0 / 0.06)` — the *only* other shadow in the system, reserved for the active segment of a segmented control. Reads as "pressed in," not "floating." Do not extend to any other control. |

### Named rules

**The Flat-By-Default Rule.** Surfaces are flat at rest. A shadow appears only on something that overlays the page and will disappear again — never on a card, row, or anything part of the page's permanent layout.

**The Blended Alert Border Rule.** Warning and danger callouts (the outstanding-month banner, modal warning/danger notes) don't use a pure status-coloured border or a plain neutral one — they blend the status colour 35% into the neutral border (`color-mix(in srgb, var(--warning) 35%, var(--border))`), softer than a solid alert colour but still legible as one.

---

## 5. Shapes

Two structural radius steps carry the system: **6px** on every interactive control at the "row" scale (buttons, inputs, table-adjacent elements, icon buttons, keypad keys) and **8px** on containers one size up (cards, modals, the structure-tree wrapper, wizard panels). Pills and the PIN dots are the only fully-round shapes (`999px`/circular), reserved for status and identity marks, never buttons or cards. Borders are a uniform 1px hairline everywhere except two deliberately heavier 1.5px borders (structure-tree node, PIN dot) where a touch more definition earns its keep on an otherwise busy or small element. No decorative borders, no `border-left` accent bars.

### Named rules

**The Tiny-Mark Rule.** Decorative marks under ~12px (the legend swatch, the wizard step segments) round to roughly a third of their own size (`3px`), not `6px` — at that scale 6px reads as a circle, not a rounded square.

**The Large-Icon-Container Rule.** Containers built to hold a single icon at 40–52px (the avatar, the auth-brand mark, the wizard icon wrap) round to roughly a quarter of their own width (`10–14px`), scaling with the box rather than snapping to 8px. A formula (`~25% of box width`), applied per icon-container size, not a fixed frontmatter token.

**The Nested-Radius Rule.** A control nested inside a rounded container (the segmented control's inner buttons, inside the 6px-radius track with 2px padding) takes the outer radius minus the padding — currently 5px for a 6px outer radius with 2px padding. Keeps concentric corners visually parallel.

**The Half-Height Bar Rule.** Thin bar/track elements (the slab-distribution bar track and fill) round to half their own height, giving true pill end-caps at any thickness, rather than reusing the 6px/8px tiers.

**Reused, not duplicated (8 August 2026):** the Home "Rewards by slab" chart (CR-5) is the same bar-list/bar-row component as "Members by slab," reused verbatim — same track/fill/label shapes, just summing Rewards per slab bucket instead of counting members. No new chart component was introduced for it.

---

## 6. Components

### 6.1 Buttons
- **Shape:** 6px radius, 32px height (27px at small), 1px border (transparent on primary/ghost, border-token on secondary/danger).
- **Primary:** indigo fill, white text — the one call-to-action colour per screen.
- **Secondary:** white surface, border outline, ink text — the default, most-used variant.
- **Ghost:** no fill, muted text — low-emphasis actions (icon-adjacent, toolbar).
- **Danger:** white surface at rest, danger-coloured text; fills to the danger-weak tint with a danger border on hover — a deliberate two-stage confirmation feel for destructive actions.
- **Commit** (layered on Primary): reserved for the single control that triggers a genuinely irreversible action — **closing a month** (Rule-18). Taller (36px vs 32px) and bolder (700 vs 550 weight) than a routine Primary, still indigo, no new colour. Weight communicates stakes here, not colour, so it never fights the One Accent Rule.
- **Hover/focus:** primary brightens (`filter: brightness(1.08)`); secondary/danger shift background toward the page background/danger-weak; disabled drops to 0.45 opacity, no hover.

### 6.2 Pills (status)
- **Style:** fully rounded, 21px height, a 6px status dot before the label (suppressed on the neutral/slab variants, which carry no implied state).
- **Variants:** Active (green), Inactive (red), Slab/band (indigo, no dot — it's a value, not a state), Locked (amber), Neutral (muted, bordered).
- **Rule:** the label text is always present. The dot reinforces colour; it never substitutes for the word.

### 6.3 Cards / containers
- 8px radius, white surface against the slate page, no shadow (see §4). 1px hairline border. 18px internal padding standard; stat cards use a tighter 14–16px.

### 6.4 Inputs / fields
- White surface, 1px border, 6px radius, 34px height, 11px horizontal padding.
- **Focus:** border shifts to indigo plus a 3px indigo-weak glow — no outline ring on top of it.
- **Error:** border shifts to danger; an 11.5px danger-coloured hint line appears below the field.
- **Disabled:** background drops to the page-background token, text to muted, cursor not-allowed.

### 6.5 Navigation
- **Sidebar item:** 13.5px body weight, 6px radius, full-width hit target, icon at 16px/0.75 opacity. Hover shifts background toward the page background; the active item gets the indigo-weak fill, indigo text, 600 weight, full-opacity icon — the single strongest "you are here" signal in the system.
- **Breadcrumb:** 12.5px muted trail with `›`-style separators at 0.5 opacity; only the current page is ink-coloured and bold.

### 6.6 Modals
- **Shape:** 480px max width (640px at wide variant), 8px radius, `max-height: 88vh` with the body scrolling rather than the whole dialog.
- **Elevation:** the one modal shadow (§4) — modals, toasts, and the search dropdown are the only things that get it. Backdrop is ink at 50% with a 1px blur.
- **Structure:** header (15px/650 title, hairline below, ✕ at the right), body at 18–20px padding, footer with a hairline above and actions right-aligned — **Cancel first, then the action**, never reversed.
- **Focus:** Cancel takes focus on open, never the confirming button. A destructive action requires a deliberate move to reach, not a stray Enter press.
- **Dismissal:** backdrop click and Escape both close a dismissable modal. Modals that must not be dismissed accidentally (add/edit member) opt out of both — closeable only by Cancel or ✕.
- **Motion:** 0.14s rise-and-settle (`translateY(6px) scale(0.98)` → none), clamped to nothing under `prefers-reduced-motion`.

### 6.7 Toasts
- Ink fill with page-background text by default; success fill for confirmations, danger fill for refusals. 15px icon, 12.5px label, 6px radius, the modal shadow.
- **Placement:** bottom-right stack, 8px gaps, `aria-live="polite"` so a confirmation is announced without stealing focus.
- **Lifetime:** ~3.4s, then a 0.2s fade. Toasts confirm; they never carry information the operator must act on — anything actionable belongs in a banner or a modal, which do not disappear.

### 6.8 Alert notes (in-modal)
- **Variants:** `.modal-warn` (amber — a consequence worth reading) and `.modal-danger-note` (red — a refusal or a risk).
- **Style:** the Blended Alert Border Rule (§4) in component form — weak-tinted fill, border mixed 35% status colour into the neutral border, 6px radius, 12.5px copy. Never a solid status-coloured border.
- **Contrast:** copy on the amber variant uses the darker amber-text token (§1), not the plain warning colour — the fill colour measures ≈3.2:1 on white and fails AA for body text.
- **Composition:** 15px status-coloured icon, 9px gap, text with the consequence in 650 weight and the qualification in normal weight. The severe half of the sentence must be readable on its own.

### 6.9 Impact summary
- **Purpose:** shows what a pending change would do, before it is committed — backs the settings pre-save warning (RQ-18/V7.6, [03](03-functional-specification.md) §5.7).
- **Style:** bordered 6px container, rows separated by hairlines, muted label left, value right.
- **Before/after:** the old figure in muted normal weight, a muted arrow, the new figure in 650 weight — direction of travel reads at a glance without colour. Tabular numerals throughout.
- **Unchanged state:** shows the single current figure followed by a muted "unchanged" rather than an identical pair either side of an arrow, which would read as a change that isn't one.

### 6.10 Restore option list
- **Purpose:** picking one item from a small set of consequential choices (backups to restore from) where a native radio would be too small a target and too quiet.
- **Style:** full-width card rows, 6px radius, 1px border, 10–12px padding, stacked with 8px gaps. Custom 15px round radio at the left, filled indigo when selected.
- **Selection:** border shifts to indigo plus the 3px indigo-weak glow — the same treatment as input focus, deliberately reused rather than inventing a second selection language.
- **Content:** a 13px/600 primary line naming the thing in the operator's own terms (the month a backup holds, not its filename), and an 11.5px muted line for provenance (version, whether it was corrected).
- **Reused, not duplicated, for the whole-console mechanism (Rule-43):** the Settings "Restore" card lists whole-console backups in this exact component — a scheduled/manual backup's primary line names *when* it was taken ("Weekly — 3 Aug 2026, 6:02 PM") in place of a month, same provenance line underneath. One list component for every kind of backup. The voluntary first-run path skips this list entirely and goes straight to a file-browse action — a brand-new machine has no local backups of its own to list yet.

### 6.11 Restore confirmation (checklist)
- **Purpose:** confirming a whole-console restore — replacing everything currently in the console — before it happens.
- **Style:** reuses the month-close wizard's checklist pattern exactly: a `.modal-warn` note naming what will be replaced, one checklist checkbox ("I understand this overwrites all current data and cannot be undone"), Cancel first then a disabled-until-checked danger-button Restore action. No new confirmation pattern — this earns the same weight already given to closing a month (V8.5), not heavier or lighter.
- **Safety net:** the console takes one more backup of its own current state immediately before overwriting it, on every restore path, regardless of entry point — a property of the action, not of this particular modal.

### 6.12 Structure Tree Node *(the signature component — FR-2/UN-16)*
- **Shape:** 172px-wide card (190px for the root node), 8px radius, 1.5px border (heavier than the system's usual 1px, since these sit inside a busy diagram).
- **Root distinction:** the root node alone gets an indigo border and indigo-weak fill — every other node is neutral until interacted with.
- **Content:** exactly three fields, per FR-2/UN-16 — name (13px/650), member number (11px muted, monospace), and **own** Business Volume as a tabular numeric value with a small uppercase label above a 1px top-border divider. **Never Total Business Volume.**
- **Interaction:** hover lifts the border to indigo and nudges the card up 1px (`translateY(-1px)`) — a small, honest affordance that it's clickable, not a shadow-based lift.
- **Connector lines:** thin (1.5px) neutral-border-coloured SVG lines, never coloured, so the tree's data (the nodes) always reads louder than its scaffolding (the connectors).
- **Reused verbatim in the Full Hierarchy Window (§6.13).** Same card, same three fields, same connector rule. Showing more of the tree never means showing more per node — FR-2's constraint is a property of the component, not of the screen it sits on.

### 6.13 Full Hierarchy Window *(FR-10/UN-31/Rule-45)*
- **What it is:** a separate top-level window showing the whole structure from the top member, every branch expanded, drawn once and never updated. Not a modal, not a route — its own window, so its rendering cost never lands on the console.
- **Header:** the top member's name, the total member count, and an **"as at &lt;date, time&gt;"** stamp, at the same weight as a page title. The stamp is not decoration — it is what makes a printed copy honest about when it was true, and it must survive printing.
- **Toolbar:** the Structure screen's zoom control, extended — the zoom **floor drops to 10%** (against the main chart's 50%) because a whole network has to be takeable-in at once, while the ceiling stays 150%. Plus fit-width, a search field, and a Print action. Same control shapes and sizes as the Structure toolbar; nothing new is invented here.
- **Search highlight:** the matched node gets a 2px indigo ring and is scrolled to centre. The ring is a focus treatment, not a fill — the node's own three fields must stay exactly as legible as every other node's.
- **Size gate:** above 60 descendants, a confirmation modal names the **exact count** before anything is drawn ("This will draw 4,182 members in a new window. It may take a moment."), Cancel first, then the primary Open action. The count must be real, never an estimate or a rounded figure — a number the user cannot trust is worse than no number.
- **Print:** a print stylesheet drops the toolbar, keeps the header and stamp on the first page, and lets the chart break across pages. A wide chart spans many pages by nature (TR-7); do not scale it down to fit one page, which would render the node text unreadable.
- **Theme:** inherits the console's light/dark theme as at the moment it opens, through the same tokens. It does not follow later theme changes — consistent with it not following data changes either.
- **Read-only, visibly:** no node is a link, no control writes anything, nothing hovers as though clickable. The absence of affordances is the design; do not add a hover lift to a node here (§6.12's `translateY(-1px)` is an affordance for opening a branch, and there are no branches left to open).

---

## 7. Theming

Light default, dark mode available via the same tokens — never a separately-designed second theme. `prefers-color-scheme` carries the OS preference; an in-app toggle stamps `data-theme="dark"`/`data-theme="light"` on the root, which must override the media query in both directions. Style every component through the tokens in §1, never with a colour value hardcoded inside a media query.

---

## 8. Do's and Don'ts

### Do
- Keep the accent to one hue (indigo) system-wide; solve a second-colour need with a status colour or a neutral.
- Pair every status colour with a text label — pills, banners, and inactive-member rows are never colour-only.
- Use tabular numerals for every figure that could sit in a column, and monospace specifically for member ID numbers.
- Keep the outstanding-month banner undismissable — no close icon, no auto-hide, ever (Rule-20).
- Stay inside the restricted vocabulary (§3 of [01](01-product-and-scope.md)) in every visible string, including placeholder/empty-state copy and error messages.
- Reserve shadow for things that float temporarily above the page, plus the one narrow control-lift exception on a segmented control's active state — nothing beyond those.
- Gate any view that renders an unbounded number of nodes at once behind an explicit confirmation naming the real count once it passes a readable threshold (>60) — a scrollable container is not the same thing as a readable one. In this system that view is the **Full Hierarchy Window** (§6.13), which is where the gate lives; the Structure screen's one-branch-at-a-time chart is bounded by a single generation and needs none.
- Show the phone number in member search results (Rule-44) — the administrator has to confirm they picked the right person, and a name alone does not do that. It is personal data on a landing screen, deliberately, and visible only to the one administrator who already sees it everywhere else.
- Keep any anything-could-be-slow view in its own window rather than making the main console carry it, and say plainly, before it opens, how much work it is about to do.

### Don't
- Add a shadow to a card, table row, or sidebar — flat-by-default is load-bearing to this system's density, not a stylistic default that can flex.
- Introduce a second accent colour, a gradient, or decorative colour anywhere — this is a discreet, non-commercial tool by explicit client requirement, not a marketing surface.
- Use commercial/MLM vocabulary anywhere, including in mock or placeholder data.
- Invent a new radius value outside the documented tiers (3px tiny marks, 6px controls, 8px containers, 10–14px large-icon containers by formula, 999px fully round) — every value traces to one of these.
- Let a structure-tree connector line take on the accent colour or grow thicker than the node borders it connects — the diagram's data must always outweigh its scaffolding.
