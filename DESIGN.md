---
name: Member Rewards Console
description: A private, single-operator ledger for a referral network's monthly business volume and rewards.
colors:
  indigo: "#4f46e5"
  indigo-weak: "#eef2ff"
  slate-bg: "#f8fafc"
  white-surface: "#ffffff"
  slate-border: "#e2e8f0"
  ink: "#0f172a"
  slate-muted: "#64748b"
  ledger-green: "#059669"
  ledger-green-weak: "#ecfdf5"
  amber: "#d97706"
  amber-weak: "#fffbeb"
  amber-text: "#92400e"
  red: "#dc2626"
  red-weak: "#fef2f2"
typography:
  headline:
    fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif"
    fontSize: "20px"
    fontWeight: 650
    lineHeight: 1.3
    letterSpacing: "-0.015em"
  title:
    fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif"
    fontSize: "15px"
    fontWeight: 650
    lineHeight: 1.4
    letterSpacing: "normal"
  body:
    fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: 1.5
    letterSpacing: "normal"
  label:
    fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif"
    fontSize: "11px"
    fontWeight: 650
    lineHeight: 1.3
    letterSpacing: "0.045em"
  numeric:
    fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif"
    fontSize: "22px"
    fontWeight: 650
    lineHeight: 1.2
    letterSpacing: "-0.01em"
  title-sm:
    fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif"
    fontSize: "13px"
    fontWeight: 650
    lineHeight: 1.4
    letterSpacing: "normal"
  numeric-lg:
    fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif"
    fontSize: "28px"
    fontWeight: 650
    lineHeight: 1.1
    letterSpacing: "-0.01em"
  caption:
    fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif"
    fontSize: "12px"
    fontWeight: 400
    lineHeight: 1.4
    letterSpacing: "normal"
rounded:
  xs: "3px"
  sm: "6px"
  lg: "8px"
  full: "999px"
spacing:
  xs: "4px"
  sm: "8px"
  md: "14px"
  lg: "18px"
  xl: "24px"
  2xl: "32px"
components:
  button-primary:
    backgroundColor: "{colors.indigo}"
    textColor: "#ffffff"
    rounded: "{rounded.sm}"
    padding: "0 13px"
    height: "32px"
  button-secondary:
    backgroundColor: "{colors.white-surface}"
    textColor: "{colors.ink}"
    rounded: "{rounded.sm}"
    padding: "0 13px"
    height: "32px"
  button-ghost:
    backgroundColor: "transparent"
    textColor: "{colors.slate-muted}"
    rounded: "{rounded.sm}"
    padding: "0 13px"
    height: "32px"
  pill-active:
    backgroundColor: "{colors.ledger-green-weak}"
    textColor: "{colors.ledger-green}"
    rounded: "{rounded.full}"
    padding: "0 9px"
    height: "21px"
  pill-inactive:
    backgroundColor: "{colors.red-weak}"
    textColor: "{colors.red}"
    rounded: "{rounded.full}"
    padding: "0 9px"
    height: "21px"
  card:
    backgroundColor: "{colors.white-surface}"
    rounded: "{rounded.lg}"
    padding: "18px"
  input-field:
    backgroundColor: "{colors.white-surface}"
    textColor: "{colors.ink}"
    rounded: "{rounded.sm}"
    height: "34px"
    padding: "0 11px"
---

# Design System: Member Rewards Console

## Overview

**Creative North Star: "The Single Ledger"**

This is a private operating console for one person, read like a ledger every day: dense, numeric, and built to be trusted at a glance rather than admired. There is exactly one accent color and it is spent carefully — on the action that matters (primary buttons), the place the operator currently is (active nav), and the thing that just gained focus (inputs, rings) — never as decoration. Everything else lives in a tight range of slate neutrals against a near-white page.

The system is flat by design. Surfaces sit directly on the page with a 1px hairline border, not a shadow — depth is reserved for the handful of things that genuinely float above the page (a modal, a toast, a search-results dropdown). Status is never color alone: an inactive member gets a labelled pill, not a colored dot, because the one operator here is not asked to rely on color perception to run their business.

Numbers are the product. Every figure — a Business Volume entry, a team total, a stat card — is tabular and right-aligned so columns line up the way a real ledger's would; six-digit member numbers additionally take a monospace face so they read as fixed-width codes, distinct from prose. Nothing here uses the vocabulary of a commercial platform — the visible word list is closed (*member, Business Volume, Rewards, royalty, volume, slab, level, leg*) and that restraint extends to the visual language too: no gradients, no marketing hero type, no rounded-SaaS softness.

**Key Characteristics:**
- One accent (indigo), spent sparingly, never decoratively
- Flat by default; shadows only on things that float (modal, toast, search dropdown)
- Status is a labelled pill, never a color alone
- Every number is tabular; identifiers are monospace
- Dense, ~40px table rows, restrained 6–8px radius, no ornament

## Colors

A near-monochrome slate system with a single indigo accent — the palette of a tool meant to disappear into daily use, not announce itself.

### Primary
- **Indigo** (`#4f46e5`, dark: `#6366f1`): the one accent in the system. Primary buttons, links, the active sidebar item, focus rings, the selection color, and slab/status chips that need to read as "the system's own." Used on a small minority of any given screen.
- **Indigo, weak** (`#eef2ff`, dark: `#1e1b4b`): the tint under the accent — active nav background, focus-ring halo, the slab pill fill, avatar backgrounds. Never a second brand color; always the same hue as Indigo, just diluted.

### Neutral
- **Slate background** (`#f8fafc`, dark: `#0f172a`): the page itself, and the recessed background inside inputs/tracks/segmented controls.
- **White surface** (`#ffffff`, dark: `#1e293b`): every raised surface — cards, table rows, inputs, modals, the sidebar.
- **Slate border** (`#e2e8f0`, dark: `#334155`): the 1px hairline that does almost all of this system's separation work — between cards, table rows, sidebar sections, form fields.
- **Ink** (`#0f172a`, dark: `#f1f5f9`): primary text and the strongest UI marks (card titles, values).
- **Slate, muted** (`#64748b`, dark: `#94a3b8`): secondary text, labels, table headers, breadcrumbs, placeholder-weight copy.

### Status
- **Ledger green** (`#059669`, dark: `#10b981`): status only, never decorative — the "Active" pill, completed checklist steps, success toasts.
- **Amber** (`#d97706`, dark: `#f59e0b`): the outstanding-month banner and locked states. Amber, deliberately not red — it names a required action, not an error.
- **Amber, text** (`#92400e`, dark: `#f59e0b`): a darker same-hue step reserved for amber text on a light surface (validation-warning copy, disabled-state explanations) — `#d97706` itself falls short of 4.5:1 on white and amber is exactly the color a scanning-not-reading user needs to actually read. The icon/border/banner-fill uses `--warning` as before; only small warning *text on a light background* switches to this step.
- **Red** (`#dc2626`, dark: `#ef4444`): validation errors, the "Inactive" pill, destructive actions (danger buttons, delete icon-button hover).
- Each status color carries a `-weak` tint (`#ecfdf5` / `#fffbeb` / `#fef2f2`, dark: `#052e21` / `#3a2a06` / `#3d1414`) used as the pill/banner/note fill, with the full-strength color reserved for text and small marks (dots, icons) inside it.

### Named Rules
**The One Accent Rule.** Indigo is the only brand color in the system. There is no secondary or tertiary accent — a second color need is solved with a status color or a neutral, never a new hue.

**The Color-Plus-Label Rule.** No status is ever conveyed by color alone. Active/Inactive, locked/unlocked, and every pill in the system pairs its color with a text label — this is a hard requirement (§11.8 accessibility, M4.5/M6.5), not a preference.

## Typography

**Body Font:** System UI stack — `-apple-system, BlinkMacSystemFont, "Segoe UI", system-ui, sans-serif` (with generic `sans-serif` fallback).
**Label/Mono Font:** `ui-monospace, SFMono-Regular, Menlo, Consolas, monospace`, used only for six-digit member numbers and other identifiers (`.mono`), and for `font-variant-numeric: tabular-nums` on every other figure (`.num`).

**Character:** Plain and functional throughout — a single system stack carries every role via size and weight, not typeface changes. This is deliberate: the product's discretion requirement extends to typography too, nothing here should read as "designed" in a way that draws attention to itself.

> **Committed for the real build, not yet in this prototype:** the production Tauri app bundles **Inter** locally (no web fonts, ever — offline constraint). This prototype uses the system-ui stack instead for CSP-safety in a sandboxed preview; treat the system stack as a placeholder for Inter, not as the final typographic decision.

### Hierarchy
- **Headline** (650, 20px, letter-spacing -0.015em): page titles only — one per screen, at the top of the content column.
- **Title** (650, 15px): modal/section headers; a smaller 13px step of the same weight covers card titles.
- **Title, small** (650, 13px): card titles — one step down from Title, same weight, for headers inside a smaller container.
- **Body** (400, 14px, line-height 1.5): the default for every paragraph, table cell, and form value.
- **Label** (650, 11px, letter-spacing 0.045em, uppercase): table column headers, stat-card labels, sidebar section labels — always muted-colored, never full ink.
- **Numeric** (650, 22px, letter-spacing -0.01em, tabular-nums): stat-card values — one of the system's few genuinely large numbers, deliberately sized to read before anything else on the screen.
- **Numeric, large** (650, 28px, letter-spacing -0.01em, tabular-nums): the Business Volume entry field alone — the single largest text in the system, reserved for the one figure the operator types on the product's most frequent action.
- **Caption** (400, 12px, muted-colored): secondary explanatory text below the size where a Label's uppercase-tracked treatment would be too loud — card subtitles, the bar-chart's own row labels/counts, the structure legend, small inline notes under a form or wizard step. The one step in the scale that's regular weight rather than a weight/tracking extreme, because at this size and this frequency, restraint reads calmer than emphasis.

### Named Rules
**The Tabular Rule.** Every number that could sit in a column — table cells, stat values, entered figures — uses `font-variant-numeric: tabular-nums`. Digits must line up; this system is read as a ledger, not prose.

## Layout

A fixed 236px sidebar plus a fluid content column (`grid-template-columns: 236px 1fr`), sticky at full viewport height. Content is padded 32px horizontally, 20px above and 40px below, with a top bar that stays sticky as the page scrolls. Density is deliberately high for a single power user in daily use: ~40px table rows, 4px as the smallest spacing increment, most gaps sitting at 8/12/14px and section-level spacing at 18/24/32px. Narrow, single-purpose screens (the Business Volume entry form) cap at 620–640px and center themselves rather than stretching to the full column — width is earned by content, not filled by default.

## Elevation & Depth

Flat by default. Cards, table rows, the sidebar, and form fields carry a 1px hairline border and no shadow — separation comes from the border and the background-shift between `--bg` and `--surface`, not from simulated lift. Shadow is reserved for things that are genuinely temporary and float above the page: the modal/toast shadow (`--shadow-modal`) and the search-results dropdown, both using the same value so "floating" always looks the same.

### Shadow Vocabulary
- **Modal/overlay** (`box-shadow: 0 20px 48px -12px rgb(15 23 42 / 0.28), 0 4px 12px -4px rgb(15 23 42 / 0.12)`; dark: alpha raised to 0.55/0.35 against pure black): modals, toasts, the search-results dropdown. Nowhere else.
- **Control-lift** (`box-shadow: 0 1px 2px rgb(0 0 0 / 0.06)`): the *only* other shadow in the system, reserved for the active segment of a segmented control — a 1px tactile lift that reads as "pressed in," not "floating." Do not extend this to any other control; it earns its exception because a segmented control's active state has no color-fill affordance of its own to lean on.

### Named Rules
**The Flat-By-Default Rule.** Surfaces are flat at rest. A shadow appears only on something that overlays the page and will disappear again (modal, toast, dropdown) — never on a card, row, or anything that's part of the page's permanent layout.

**The Blended Alert Border Rule.** Warning and danger callouts (the outstanding-month banner, modal warning/danger notes) don't use a pure status-colored border or a plain neutral one — they blend the status color 35% into the neutral border (`color-mix(in srgb, var(--warning) 35%, var(--border))`), softer than a solid alert color but still legible as one.

## Shapes

Two structural radius steps carry the system: `6px` on every interactive control at the "row" scale (buttons, inputs, table-adjacent elements, icon buttons, keypad keys) and `8px` on containers one size up (cards, modals, the structure-tree wrapper, wizard panels). Pills and the PIN dots are the only fully-round shapes (`999px` / circular), reserved for status and identity marks, never for buttons or cards. Borders are a uniform 1px hairline everywhere except two deliberately slightly heavier 1.5px borders (structure-tree node, PIN dot) where a touch more definition earns its keep on an otherwise busy or small element. No decorative borders, no `border-left` accent bars.

Two further tiers exist below and above those two steps — not drift, but real proportional rules the frontmatter's fixed pixel values can't express directly:

### Named Rules
**The Tiny-Mark Rule.** Decorative marks under ~12px (the legend swatch, the wizard step segments) round to roughly a third of their own size (`3px`), not to `6px` — at that scale `6px` reads as a circle, not a rounded square. This tier is `rounded.xs` (`3px`) in the frontmatter.

**The Large-Icon-Container Rule.** Containers built to hold a single icon at 40–52px (the avatar, the auth-brand mark, the wizard icon wrap) round to roughly a quarter of their own width (`10–14px`), scaling with the box rather than snapping to `8px` — an `8px` radius on a 52px box reads noticeably sharper than the rest of the system's rounder large-icon language. This tier isn't a frontmatter token; it's a formula (`~25% of box width`), applied per icon-container size.

**The Nested-Radius Rule.** A control nested inside a rounded container (the segmented control's inner buttons, sitting inside the `6px`-radius segmented track with `2px` padding) takes the outer radius minus the padding, not a separately chosen value — currently `5px` for a `6px` outer radius with `2px` padding. This keeps concentric corners visually parallel instead of arbitrary.

**The Half-Height Bar Rule.** Thin bar/track elements (the slab-distribution bar track and fill) round to half their own height, giving true pill end-caps at any thickness, rather than reusing `rounded.sm`/`rounded.lg`.

## Components

### Buttons
- **Shape:** 6px radius, 32px height (27px at `.btn-sm`), 1px border (transparent on primary/ghost, `--border` on secondary/danger).
- **Primary:** indigo fill, white text — the one call-to-action color per screen.
- **Secondary:** white surface, `--border` outline, ink text — the default, most-used variant.
- **Ghost:** no fill, muted text — for low-emphasis actions (icon-adjacent, toolbar).
- **Danger:** white surface at rest, `--danger` text; fills to `--danger-weak` with a `--danger` border on hover — a deliberate two-stage confirmation feel for destructive actions.
- **Commit** (`.btn-commit`, layered on Primary): reserved for the single control in the system that triggers a genuinely irreversible action (closing a month) — taller (36px vs 32px) and bolder (700 vs 550 weight) than a routine Primary, still indigo, no new color. Weight communicates stakes here, not color, so it never fights the One Accent Rule.
- **Hover/Focus:** primary brightens (`filter: brightness(1.08)`); secondary/danger shift background to `--bg`/`--danger-weak`; disabled drops to 0.45 opacity, no hover.

### Pills (status)
- **Style:** fully rounded, 21px height, a 6px status dot before the label (suppressed on the neutral/slab variants, which carry no implied state).
- **Variants:** Active (green), Inactive (red), Slab/band (indigo, no dot — it's a value, not a state), Locked (amber), Neutral (muted, bordered).
- **Rule:** the label text is always present. The dot reinforces color; it never substitutes for the word.

### Cards / Containers
- **Corner Style:** 8px radius.
- **Background:** white surface against the slate page background.
- **Shadow Strategy:** none — see Elevation & Depth. Separation comes from the border alone.
- **Border:** 1px `--border` hairline.
- **Internal Padding:** 18px standard; stat cards use a tighter 14px/16px.

### Inputs / Fields
- **Style:** white surface, 1px `--border`, 6px radius, 34px height, 11px horizontal padding.
- **Focus:** border shifts to indigo plus a 3px indigo-weak glow (`box-shadow: 0 0 0 3px var(--accent-weak)`) — no outline ring on top of it.
- **Error:** border shifts to `--danger`; an 11.5px danger-colored hint line appears below the field.
- **Disabled:** background drops to `--bg`, text to muted, cursor not-allowed.

### Navigation
- **Sidebar item:** 13.5px body weight, 6px radius, full-width hit target, icon at 16px/0.75 opacity. Hover shifts background to `--bg`; the active item gets the indigo-weak fill, indigo text, 600 weight, and full-opacity icon — the single strongest "you are here" signal in the system.
- **Breadcrumb:** 12.5px muted trail with `›`-style separators at 0.5 opacity; only the current page is ink-colored and bold.

### Modals
- **Shape:** 480px max width (640px at `.wide`), 8px radius, `max-height: 88vh` with the body scrolling rather than the whole dialog.
- **Elevation:** the one shadow in the system (`--shadow-modal`) — modals, toasts and the search dropdown are the only things that get it. Backdrop is ink at 50% with a 1px blur.
- **Structure:** header (15px/650 title, hairline below, ✕ at the right), body at 18px/20px padding, footer with a hairline above and actions right-aligned — **Cancel first, then the action**, never reversed.
- **Focus:** Cancel takes focus on open, never the confirming button. A destructive action should require a deliberate move to reach, not sit under a stray Enter press.
- **Dismissal:** backdrop click and Escape both close a dismissable modal. Modals that must not be dismissed accidentally (add/edit member) opt out of both and can only be closed by Cancel or ✕.
- **Motion:** 0.14s rise-and-settle (`translateY(6px) scale(0.98)` → none), clamped to nothing under `prefers-reduced-motion`.

### Toasts
- **Style:** ink fill with page-background text by default; `--success` fill for confirmations, `--danger` for refusals. 15px icon, 12.5px label, 6px radius, the modal shadow.
- **Placement:** bottom-right stack, 8px gaps, `aria-live="polite"` so a confirmation is announced without stealing focus.
- **Lifetime:** ~3.4s, then a 0.2s fade. Toasts confirm; they never carry information the operator must act on — anything actionable belongs in a banner or a modal, which do not disappear.

### Alert notes (in-modal)
- **Variants:** `.modal-warn` (amber — a consequence worth reading) and `.modal-danger-note` (red — a refusal or a risk).
- **Style:** the **Blended Alert Border Rule** in component form — weak-tinted fill, border mixed 35% status colour into `--border`, 6px radius, 12.5px copy. Never a solid status-coloured border.
- **Contrast:** copy on the amber variant uses `--warning-text` (`#92400e`), not `--warning` — the fill colour measures ≈3.2:1 on white and fails AA for body text.
- **Composition:** 15px status-coloured icon, 9px gap, text with the consequence in **650** weight and the qualification in normal weight. The severe half of the sentence should be readable on its own.

### Impact summary
- **Purpose:** shows what a pending change would do, before it is committed — used by the settings pre-save warning.
- **Style:** bordered 6px container, rows separated by hairlines, muted label left, value right.
- **Before/after:** the old figure in muted normal weight, a muted arrow, the new figure in 650 — so the direction of travel reads at a glance without colour. Tabular numerals throughout, per the Tabular Rule.
- **Unchanged state:** shows the single current figure followed by a muted "unchanged" rather than an identical pair either side of an arrow, which reads as a change that isn't one.

### Restore option list
- **Purpose:** picking one item from a small set of consequential choices (backups to restore from) where a native radio would be too small a target and too quiet.
- **Style:** full-width card rows, 6px radius, 1px `--border`, 10px/12px padding, stacked with 8px gaps. Custom 15px round radio at the left, filled indigo when selected.
- **Selection:** border shifts to indigo plus the 3px indigo-weak glow — **the same treatment as input focus**, deliberately reused rather than inventing a second selection language.
- **Content:** a 13px/600 primary line naming the thing in the operator's own terms (the month a backup holds, not its filename), and an 11.5px muted line for provenance (version, whether it was corrected).
- **Reused, not duplicated (7 August 2026):** the Settings "Restore" card lists whole-console backups in this exact component — a scheduled/manual backup's primary line names *when* it was taken ("Weekly — 3 Aug 2026, 6:02 PM") in place of a month, same provenance line underneath. One list component for every kind of backup, not a second one for the new kind. The voluntary first-run path (reached via a plain link on the setup screen, not a competing button) skips this list entirely and goes straight to a file-browse action — a brand-new machine has no local backups of its own to list yet, only the one the operator brings with them.

### Restore confirmation (checklist)
- **Purpose:** confirming a whole-console restore — replacing everything currently in the console — before it happens.
- **Style:** reuses the month-close wizard's checklist pattern exactly: a `.modal-warn` note naming what will be replaced, one checklist checkbox ("I understand this overwrites all current data and cannot be undone"), Cancel first then a disabled-until-checked `.btn-danger` Restore action. No new confirmation pattern was introduced for this — this action earns the same weight already given to closing a month, not a heavier or lighter one.
- **Safety net:** the console takes one more backup of its own current state immediately before overwriting it, on every restore path, regardless of entry point — stated here because it's a property of the action, not of this particular modal.

### Structure Tree Node (signature component)
- **Shape:** 172px-wide card (190px for the root node), 8px radius, 1.5px border (heavier than the system's usual 1px, since these sit inside a busy diagram).
- **Root distinction:** the root node alone gets an indigo border and indigo-weak fill — every other node is neutral until interacted with.
- **Content:** exactly three fields, per FR-2/UN-16 — name (13px/650), member number (11px muted), and own Business Volume as a tabular numeric value with a small uppercase label above a 1px top-border divider.
- **Interaction:** hover lifts the border to indigo and nudges the card up 1px (`translateY(-1px)`) — a small, honest affordance that it's clickable, not a shadow-based lift.
- **Connector lines:** drawn as thin (`1.5px`) `--border`-colored SVG lines, never colored, so the tree's data (the nodes) always reads louder than its scaffolding (the connectors).

## Do's and Don'ts

### Do:
- **Do** keep the accent to one hue (Indigo) system-wide; solve a second-color need with a status color or a neutral.
- **Do** pair every status color with a text label — pills, banners, and inactive-member rows are never color-only.
- **Do** use tabular numerals for every figure that could sit in a column, and monospace specifically for member ID numbers.
- **Do** keep the outstanding-month banner undismissable — no close icon, no auto-hide, ever (Rule 20).
- **Do** stay inside the restricted vocabulary (*member, Business Volume, Rewards, royalty, volume, slab, level, leg*) in every visible string, including placeholder/empty-state copy and error messages.
- **Do** reserve shadow for things that float temporarily above the page (modal, toast, search dropdown), plus the one narrow control-lift exception on a segmented control's active state — nothing beyond those.
- **Do** gate any view that renders an unbounded number of nodes at once (the structure screen's full hierarchy) behind an explicit confirmation once the count passes a readable threshold — a scrollable container is not the same thing as a readable one.

### Don't:
- **Don't** add a shadow to a card, table row, or sidebar — flat-by-default is load-bearing to this system's density, not a stylistic default that can flex.
- **Don't** introduce a second accent color, a gradient, or decorative color anywhere — this is a discreet, non-commercial tool by explicit client requirement, not a marketing surface.
- **Don't** use commercial/MLM vocabulary (sale, purchase, order, cash, payment, commission, invoice) anywhere, including in mock or placeholder data.
- **Don't** invent a new radius value outside the documented tiers (`3px` tiny marks, `6px` controls, `8px` containers, `10–14px` large icon containers by formula, `999px` fully round) — every value in the system traces to one of these, not a one-off.
- **Don't** let a structure-tree connector line take on the accent color or grow thicker than the node borders it connects — the diagram's data must always outweigh its scaffolding.
