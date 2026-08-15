# Member Detail PDF Export Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an "Export PDF" button to the member detail screen that generates a
Rust-side PDF of one member's own record — identity, the four stat figures, the
Rewards-detail breakdown, and their direct legs' Total Business Volume — for
whatever period the screen is currently viewing.

**Architecture:** New `m4_search::pdf` module builds a `genpdf::Document` from the
existing `MemberDetail` struct (no new calculation logic). A new Tauri command
`export_member_detail_pdf` (API-46) wraps it, following the exact same
save-dialog-then-write-path shape every `.xlsx` export already uses (ADR-007,
extended by ADR-013). Frontend adds one button + one IPC binding, mirroring
`reports.tsx`'s existing export buttons.

**Tech Stack:** Rust (`genpdf` 0.2.0 on top of `printpdf`), static Inter TTF fonts
(already vendored at `src-tauri/assets/fonts/`), Tauri 2, React 19 + TypeScript.

**Spec:** `docs/superpowers/specs/2026-08-15-member-detail-pdf-export-design.md`

## Global Constraints

- All PDF generation happens Rust-side. The WebView never receives raw file bytes —
  only a destination path from the native save dialog (ADR-007/ADR-013).
- No new calculation logic. `get_member_detail` stays untouched; the PDF renders
  its existing output.
- Every figure is spelled out in full on the document — "Business Volume", "Total
  Business Volume" — never "BV"/"TBV" (CR-6).
- No currency figures, no company branding (BO-6, product-and-scope.md §1).
- Direct legs only, one level — never the full downline (Rule-45/FR-2).
- The command surface is exactly 46 after this feature (API-46) — `ALL_COMMAND_NAMES`,
  `capabilities/default.json`, and `tests/contract.rs`'s count assertion must all agree.
- Vocabulary guard (`scripts/vocabulary-grep.mjs`) must stay clean — no sale/purchase/
  order/cash/payment/commission/invoice in any user-visible string this feature adds.

---

## File Structure

- **`src-tauri/assets/fonts/Inter-Regular.ttf`, `Inter-SemiBold.ttf`, `Inter-LICENSE`**
  — already added (static Inter v3.19, OFL-1.1). Embedded via `include_bytes!`.
- **`src-tauri/src/m4_search/pdf.rs`** (new) — everything genpdf-specific: font
  loading, the five document sections (header, stats, two-column mid-section,
  direct-legs table, running header), and `render_member_detail_pdf`, the one
  function the rest of the app calls.
- **`src-tauri/src/m4_search/mod.rs`** (modify) — `pub mod pdf;` + the
  `export_member_detail_pdf` free function (resolves `MemberDetail`, calls
  `pdf::render_member_detail_pdf`, writes to `output_path`).
- **`src-tauri/src/commands.rs`** (modify) — `#[tauri::command] export_member_detail_pdf`
  wrapper (session + connection guard, same shape as every other command).
- **`src-tauri/src/command_names.rs`** (modify) — add to `ALL_COMMAND_NAMES`.
- **`src-tauri/src/lib.rs`** (modify) — add to the `generate_handler!` list.
- **`src-tauri/capabilities/default.json`** (modify) — add `allow-export-member-detail-pdf`.
- **`src-tauri/tests/contract.rs`** (modify) — bump the 45→46 count assertion, add
  `export_member_detail_pdf_requires_a_session` + `..._end_to_end_through_the_command_layer`.
- **`src-tauri/Cargo.toml`** (modify) — add `genpdf = "0.2.0"` dependency,
  `pdf-extract = "0.12.0"` dev-dependency.
- **`src/lib/ipc/m4-search.ts`** (modify) — `exportMemberDetailPdf()` binding.
- **`src/screens/member-detail.tsx`** (modify) — "Export PDF" button + handler.

---

### Task 1: Font loading

**Files:**
- Create: `src-tauri/src/m4_search/pdf.rs`
- Modify: `src-tauri/src/m4_search/mod.rs:1-12` (add `pub mod pdf;`)
- Modify: `src-tauri/Cargo.toml` (add `genpdf = "0.2.0"` under `[dependencies]`,
  next to `rust_xlsxwriter`)

**Interfaces:**
- Produces: `pub(super) fn load_font_family() -> genpdf::fonts::FontFamily<genpdf::fonts::FontData>`
  — used by every later task in this file.

- [ ] **Step 1: Add the dependency**

In `src-tauri/Cargo.toml`, right after the `rust_xlsxwriter` line:

```toml
rust_xlsxwriter = "0.90.1"
# ADR-013: extends ADR-007's Rust-side-generation boundary to a second file
# type. genpdf paginates flowing content (tables) automatically — the
# direct-legs table has no fixed row count, and printpdf alone would need
# hand-rolled page-break logic for it.
genpdf = "0.2.0"
```

**Step 2: Write the failing test**

Create `src-tauri/src/m4_search/pdf.rs`:

```rust
//! M4.8 — PDF rendering for the member detail export (CR-6, ADR-013).
//! Pure rendering: every value here comes from an already-computed
//! `MemberDetail` (m4_search::get_member_detail) — no calculation happens
//! in this module.

use genpdf::fonts::{FontData, FontFamily};

/// Static Inter TTFs (v3.19, OFL-1.1 — see assets/fonts/Inter-LICENSE),
/// embedded at compile time so the shipped binary needs no filesystem
/// access to render (NFR-14, offline). SemiBold stands in for both the
/// bold and bold_italic slots — the document never uses italic text, so
/// italic/bold_italic just reuse regular/SemiBold rather than shipping two
/// more font files nobody will ever see rendered.
pub(super) fn load_font_family() -> FontFamily<FontData> {
    let regular = FontData::new(
        include_bytes!("../../assets/fonts/Inter-Regular.ttf").to_vec(),
        None,
    )
    .expect("bundled Inter-Regular.ttf must parse — this is a build asset, not user input");
    let bold = FontData::new(
        include_bytes!("../../assets/fonts/Inter-SemiBold.ttf").to_vec(),
        None,
    )
    .expect("bundled Inter-SemiBold.ttf must parse — this is a build asset, not user input");
    FontFamily {
        regular: regular.clone(),
        bold: bold.clone(),
        italic: regular,
        bold_italic: bold,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bundled_font_family_loads_without_panicking() {
        let _family = load_font_family();
    }
}
```

- [ ] **Step 3: Run the test**

Run: `cd src-tauri && cargo test --lib m4_search::pdf::tests`
Expected: PASS — this test exists to catch a corrupt/mismatched font asset at
build time, not to prove behaviour; a panic here means the `.ttf` files
themselves are bad, not a logic bug.

- [ ] **Step 4: Wire the module in**

In `src-tauri/src/m4_search/mod.rs`, after the existing `use` block (around line 12):

```rust
pub mod pdf;
```

- [ ] **Step 5: Run the full m4_search test suite**

Run: `cd src-tauri && cargo test --lib m4_search`
Expected: PASS, no new failures.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/m4_search/mod.rs src-tauri/src/m4_search/pdf.rs
git commit -m "feat(m4-pdf): load bundled Inter font family for PDF rendering"
```

---

### Task 2: Header + stat boxes section

**Files:**
- Modify: `src-tauri/src/m4_search/pdf.rs`

**Interfaces:**
- Consumes: `load_font_family()` (Task 1).
- Produces: `pub(super) fn header_section(member: &Member, total_business_volume: i64, slab_pct: i64, own_business_volume: i64, rewards_total: i64, period_label: &str, generated_at: &str, is_root: bool) -> genpdf::elements::LinearLayout`
  — used by Task 5's document assembly.

- [ ] **Step 1: Write the failing test**

Add to the `mod tests` block in `pdf.rs` (keep `use super::*;` at the top):

```rust
    use crate::m1_members::Member;
    use genpdf::Element as _;
    use genpdf::{Document, Size};

    fn sample_member(is_active: bool, introducer_member_id: Option<i64>) -> Member {
        Member {
            id: 100042,
            name: "Ravi Patel".into(),
            phone: "+91 98765 43210".into(),
            email: Some("ravi.patel@example.com".into()),
            address: "12 MG Road, Ahmedabad".into(),
            introducer_member_id,
            level: 3,
            is_active,
            joining_date: "2024-03-12".into(),
            consent_given: true,
            consent_date: "2024-03-12".into(),
            created_at: "2024-03-12".into(),
        }
    }

    /// Renders a single element into a one-page PDF and extracts its text —
    /// the only reliable way to assert genpdf output short of visual
    /// inspection. `pdf-extract` is a dev-dependency only; nothing in the
    /// shipped binary depends on it.
    fn render_and_extract(element: impl genpdf::Element + 'static) -> String {
        let mut doc = Document::new(load_font_family());
        doc.set_paper_size(Size::new(210, 297)); // A4
        doc.push(element);
        let mut bytes = Vec::new();
        doc.render(&mut bytes).expect("render must succeed");
        pdf_extract::extract_text_from_mem(&bytes).expect("extracted text must be present")
    }

    #[test]
    fn header_section_shows_identity_and_period() {
        let member = sample_member(true, Some(100001));
        let section = header_section(&member, 8_450, 12, 1_200, 612, "July 2026", "15 Aug 2026 14:32", false);
        let text = render_and_extract(section);
        assert!(text.contains("Ravi Patel"));
        assert!(text.contains("100042"));
        assert!(text.contains("Active"));
        assert!(text.contains("July 2026"));
        assert!(text.contains("Business Volume"));
        assert!(!text.contains("BV"), "every figure must be spelled out in full (CR-6)");
    }

    #[test]
    fn header_section_shows_inactive_status_as_text_not_colour_alone() {
        let member = sample_member(false, Some(100001));
        let section = header_section(&member, 0, 0, 0, 0, "July 2026", "15 Aug 2026 14:32", false);
        let text = render_and_extract(section);
        assert!(text.contains("Inactive"));
    }
```

- [ ] **Step 2: Run to verify it fails**

Run: `cd src-tauri && cargo test --lib m4_search::pdf::tests`
Expected: FAIL — `header_section` is not defined yet.

- [ ] **Step 3: Implement**

Add to `pdf.rs` (above the `mod tests` block):

```rust
use genpdf::elements::{Break, LinearLayout, Paragraph};
use genpdf::style::{Color, Style};
use genpdf::Alignment;

use crate::m1_members::Member;

// 07-design-system.md §1: accent #4f46e5, ledger green #059669, red #dc2626.
const ACCENT: Color = Color::Rgb(0x4f, 0x46, 0xe5);
const SUCCESS: Color = Color::Rgb(0x05, 0x96, 0x69);
const DANGER: Color = Color::Rgb(0xdc, 0x26, 0x26);
const MUTED: Color = Color::Rgb(0x64, 0x74, 0x8b);
const INK: Color = Color::Rgb(0x0f, 0x17, 0x2a);

/// Header block: name, active/inactive (colour-plus-label — 07-design-
/// system.md's Colour-Plus-Label Rule, never colour alone), member number,
/// phone, joining date, period, generation timestamp. `is_root` suppresses
/// nothing here — it's threaded through for a future root-member mark, not
/// used yet (M4.8's scope has no root-specific header content).
pub(super) fn header_section(
    member: &Member,
    _total_business_volume: i64,
    _slab_pct: i64,
    _own_business_volume: i64,
    _rewards_total: i64,
    period_label: &str,
    generated_at: &str,
    _is_root: bool,
) -> LinearLayout {
    let mut layout = LinearLayout::vertical();

    let mut name_line = Paragraph::new("");
    name_line.push_styled(&member.name, Style::new().bold().with_font_size(15).with_color(INK));
    name_line.push_styled("   ", Style::new());
    if member.is_active {
        name_line.push_styled("Active", Style::new().bold().with_color(SUCCESS));
    } else {
        name_line.push_styled("Inactive", Style::new().bold().with_color(DANGER));
    }
    layout.push(name_line);

    let mut meta_line = Paragraph::new("");
    meta_line.push_styled(
        format!(
            "Member #{} \u{b7} {} \u{b7} Joined {}",
            member.id, member.phone, member.joining_date
        ),
        Style::new().with_color(MUTED).with_font_size(10),
    );
    layout.push(meta_line);

    let mut period_line = Paragraph::new("");
    period_line.push_styled(
        format!("Period {period_label} \u{b7} Generated {generated_at}"),
        Style::new().with_color(MUTED).with_font_size(10),
    );
    period_line.set_alignment(Alignment::Right);
    layout.push(period_line);

    layout.push(Break::new(1));
    layout
}
```

- [ ] **Step 4: Run to verify it passes**

Run: `cd src-tauri && cargo test --lib m4_search::pdf::tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/m4_search/pdf.rs
git commit -m "feat(m4-pdf): render the member detail PDF header section"
```

---

### Task 3: Stat boxes + two-column mid-section — the pagination spike

This is the task the design spec (§3, "Why genpdf over printpdf") flagged as an
open risk: a nested `TableLayout` inside a `TableLayoutRow` cell, and whether it
paginates correctly when the outer row would otherwise overflow a page. Verify
this **before** building the direct-legs table (Task 4), since a failure here
changes the whole document's layout strategy.

**Files:**
- Modify: `src-tauri/src/m4_search/pdf.rs`

**Interfaces:**
- Consumes: `Member`, `MemberDetail`'s `RewardBreakdown`/`MemberDetailChild` types
  (`m4_search::{RewardBreakdown, MemberDetailChild}`).
- Produces:
  - `pub(super) fn stat_boxes(business_volume: i64, total_business_volume: i64, slab_pct: i64, rewards_total: i64) -> genpdf::elements::TableLayout`
  - `pub(super) fn mid_section(rewards: &crate::m4_search::RewardBreakdown, member: &Member, leg_count: i64) -> genpdf::elements::TableLayout`
  - both used by Task 5.

- [ ] **Step 1: Write the failing test — the spike**

Add to `mod tests`:

```rust
    use crate::m4_search::{DifferentialLine, MemberDetailChild, OwnRewardLine, RewardBreakdown, RoyaltyLine};

    fn differential_line(id: i64, name: &str, tbv: i64, slab: i64, own_slab: i64, amount: i64) -> DifferentialLine {
        DifferentialLine {
            child_id: id,
            child_name: name.into(),
            child_total_business_volume: tbv,
            child_slab_pct: slab,
            own_slab_pct: own_slab,
            differential_pct: own_slab - slab,
            amount,
        }
    }

    #[test]
    fn stat_boxes_spell_out_every_label_in_full() {
        let table = stat_boxes(1_200, 8_450, 12, 612);
        let text = render_and_extract(table);
        assert!(text.contains("Business Volume"));
        assert!(text.contains("Total Business Volume"));
        assert!(text.contains("Slab"));
        assert!(text.contains("Rewards this period"));
        assert!(text.contains("1,200") || text.contains("1200"));
    }

    #[test]
    fn mid_section_holds_rewards_detail_and_member_details_side_by_side() {
        let member = sample_member(true, Some(100001));
        let rewards = RewardBreakdown {
            own_reward: OwnRewardLine { own_business_volume: 1_200, own_slab_pct: 12, amount: 144 },
            differentials: vec![
                differential_line(100078, "Aarav Shah", 3_200, 10, 12, 64),
                differential_line(100091, "Priya Mehta", 2_900, 9, 12, 87),
            ],
            royalty: Some(RoyaltyLine { qualifying_children: 2, rate_percent: 5, amount: 190 }),
            rewards_total: 612,
        };
        let table = mid_section(&rewards, &member, 4);
        let text = render_and_extract(table);
        assert!(text.contains("Aarav Shah"));
        assert!(text.contains("Priya Mehta"));
        assert!(text.contains("Royalty"));
        assert!(text.contains("12 MG Road, Ahmedabad"));
        assert!(text.contains("100001"), "introducer link must appear in Details");
    }

    /// The spike: a member with enough direct legs that the mid-section's
    /// nested rewards-detail table alone would overflow one page. If every
    /// leg's name still comes back out of the extracted text, genpdf paginated
    /// the nested table correctly. If leg 80 (or any middle leg) is missing,
    /// the nested table was silently truncated — the design spec's documented
    /// fallback (drop to single-column for anything that needs to break
    /// mid-page, or hand-roll this one table with printpdf) must be applied
    /// before Task 4, and this comment block updated to say which was chosen.
    #[test]
    fn mid_section_paginates_a_long_rewards_detail_table_without_dropping_rows() {
        let member = sample_member(true, Some(100001));
        let differentials: Vec<DifferentialLine> = (1..=80)
            .map(|i| differential_line(100_000 + i, &format!("Leg Number {i}"), 1_000 + i, 5, 12, 10))
            .collect();
        let rewards = RewardBreakdown {
            own_reward: OwnRewardLine { own_business_volume: 1_200, own_slab_pct: 12, amount: 144 },
            differentials,
            royalty: Some(RoyaltyLine { qualifying_children: 0, rate_percent: 5, amount: 0 }),
            rewards_total: 944,
        };
        let table = mid_section(&rewards, &member, 80);
        let text = render_and_extract(table);
        assert!(text.contains("Leg Number 1"));
        assert!(text.contains("Leg Number 40"), "a middle row must survive pagination");
        assert!(text.contains("Leg Number 80"), "the last row must survive pagination");
    }
```

- [ ] **Step 2: Run to verify it fails**

Run: `cd src-tauri && cargo test --lib m4_search::pdf::tests`
Expected: FAIL — `stat_boxes`/`mid_section` not defined.

- [ ] **Step 3: Implement**

Add to `pdf.rs`:

```rust
use genpdf::elements::{PaddedElement, TableLayout};
use genpdf::{Element, Margins};

use crate::m4_search::{DifferentialLine, RewardBreakdown};

fn stat_box(label: &str, value: String) -> PaddedElement<LinearLayout> {
    let mut inner = LinearLayout::vertical();
    let mut label_p = Paragraph::new("");
    label_p.push_styled(label, Style::new().with_color(MUTED).with_font_size(9));
    inner.push(label_p);
    let mut value_p = Paragraph::new("");
    value_p.push_styled(value, Style::new().bold().with_font_size(13).with_color(INK));
    inner.push(value_p);
    inner.padded(Margins::all(4))
}

/// Four stat boxes, one row, equal width — mirrors the screen's own
/// `grid-cols-4` stat strip. Every label spelled in full (CR-6): no "BV".
pub(super) fn stat_boxes(
    business_volume: i64,
    total_business_volume: i64,
    slab_pct: i64,
    rewards_total: i64,
) -> TableLayout {
    let mut table = TableLayout::new(vec![1, 1, 1, 1]);
    table
        .row()
        .element(stat_box("Business Volume", format_amount(business_volume)))
        .element(stat_box("Total Business Volume", format_amount(total_business_volume)))
        .element(stat_box("Slab", format!("{slab_pct}%")))
        .element(stat_box("Rewards this period", format_amount(rewards_total)))
        .push()
        .expect("a fixed 4-cell row always has the right cell count");
    table
}

/// Thousands-separated, no decimals — Business Volume is always a whole
/// number (Rule-16a: zero/negative entries are refused, not just
/// fractional cents beyond what ADR-004's fixed-point scale allows).
fn format_amount(value: i64) -> String {
    let negative = value < 0;
    let digits = value.unsigned_abs().to_string();
    let mut grouped = String::new();
    for (i, c) in digits.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(c);
    }
    let grouped: String = grouped.chars().rev().collect();
    if negative { format!("-{grouped}") } else { grouped }
}

fn rewards_detail_table(rewards: &RewardBreakdown) -> TableLayout {
    let mut table = TableLayout::new(vec![5, 2, 2]);

    let mut leg_header = Paragraph::new("");
    leg_header.push_styled("Leg", Style::new().bold().with_font_size(9).with_color(MUTED));
    let mut bv_header = Paragraph::new("");
    bv_header.push_styled("Business Volume", Style::new().bold().with_font_size(9).with_color(MUTED));
    bv_header.set_alignment(Alignment::Right);
    let mut amt_header = Paragraph::new("");
    amt_header.push_styled("Amount", Style::new().bold().with_font_size(9).with_color(MUTED));
    amt_header.set_alignment(Alignment::Right);
    table.row().element(leg_header).element(bv_header).element(amt_header).push().unwrap();

    let mut own_leg = Paragraph::new("");
    own_leg.push_styled(
        format!("Own Business Volume \u{2014} {}%", rewards.own_reward.own_slab_pct),
        Style::new(),
    );
    let mut own_bv = Paragraph::new("");
    own_bv.push_styled(format_amount(rewards.own_reward.own_business_volume), Style::new());
    own_bv.set_alignment(Alignment::Right);
    let mut own_amt = Paragraph::new("");
    own_amt.push_styled(format_amount(rewards.own_reward.amount), Style::new().bold());
    own_amt.set_alignment(Alignment::Right);
    table.row().element(own_leg).element(own_bv).element(own_amt).push().unwrap();

    for line in &rewards.differentials {
        push_differential_row(&mut table, line);
    }

    if let Some(royalty) = &rewards.royalty {
        let mut royalty_leg = Paragraph::new("");
        royalty_leg.push_styled(
            format!(
                "Royalty \u{2014} {} of {} legs qualifying",
                royalty.qualifying_children,
                rewards.differentials.len()
            ),
            Style::new(),
        );
        let mut royalty_amt = Paragraph::new("");
        royalty_amt.push_styled(format_amount(royalty.amount), Style::new().bold());
        royalty_amt.set_alignment(Alignment::Right);
        table.row().element(royalty_leg).element(Paragraph::new("")).element(royalty_amt).push().unwrap();
    }

    let mut total_label = Paragraph::new("");
    total_label.push_styled("Rewards total", Style::new().bold());
    let mut total_amt = Paragraph::new("");
    total_amt.push_styled(format_amount(rewards.rewards_total), Style::new().bold());
    total_amt.set_alignment(Alignment::Right);
    table.row().element(total_label).element(Paragraph::new("")).element(total_amt).push().unwrap();

    table
}

fn push_differential_row(table: &mut TableLayout, line: &DifferentialLine) {
    let mut leg = Paragraph::new("");
    leg.push_styled(&line.child_name, Style::new());
    let mut bv = Paragraph::new("");
    bv.push_styled(format_amount(line.child_total_business_volume), Style::new());
    bv.set_alignment(Alignment::Right);
    let mut amt = Paragraph::new("");
    amt.push_styled(format_amount(line.amount), Style::new().bold());
    amt.set_alignment(Alignment::Right);
    table.row().element(leg).element(bv).element(amt).push().unwrap();
}

fn details_block(member: &Member, leg_count: i64) -> LinearLayout {
    let mut layout = LinearLayout::vertical();
    let rows: Vec<(&str, String)> = vec![
        ("Address", member.address.clone()),
        ("Email", member.email.clone().unwrap_or_else(|| "Not provided".into())),
        (
            "Introduced by",
            member
                .introducer_member_id
                .map(|id| format!("#{id}"))
                .unwrap_or_else(|| "None \u{2014} root member".into()),
        ),
        ("Direct legs", leg_count.to_string()),
        ("Consent captured", member.consent_date.clone()),
    ];
    for (label, value) in rows {
        let mut label_p = Paragraph::new("");
        label_p.push_styled(label, Style::new().with_color(MUTED).with_font_size(9));
        layout.push(label_p);
        let mut value_p = Paragraph::new("");
        value_p.push_styled(value, Style::new().with_font_size(11));
        layout.push(value_p);
        layout.push(Break::new(0.5));
    }
    layout
}

/// The screen's own two-column grid (`lg:grid-cols-[1.4fr_1fr]`,
/// member-detail.tsx:175): Rewards detail beside member Details. This is
/// the layout the design spec flagged as an unverified pagination risk —
/// see `mid_section_paginates_a_long_rewards_detail_table_without_dropping_rows`.
pub(super) fn mid_section(rewards: &RewardBreakdown, member: &Member, leg_count: i64) -> TableLayout {
    let mut outer = TableLayout::new(vec![7, 5]);
    outer
        .row()
        .element(rewards_detail_table(rewards).padded(Margins::all(3)))
        .element(details_block(member, leg_count).padded(Margins::all(3)))
        .push()
        .expect("a fixed 2-cell row always has the right cell count");
    outer
}
```

- [ ] **Step 4: Run to verify it passes**

Run: `cd src-tauri && cargo test --lib m4_search::pdf::tests`
Expected: PASS, including the 80-leg pagination test.

**If the pagination test fails:** stop here. Read the design spec's "Why genpdf
over printpdf" section, pick one of its two documented fallbacks, apply it to
`mid_section`, update the doc comment above to record which one and why, then
re-run this task's tests before moving to Task 4.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/m4_search/pdf.rs
git commit -m "feat(m4-pdf): render stat boxes and the two-column rewards/details section"
```

---

### Task 4: Direct legs table

**Files:**
- Modify: `src-tauri/src/m4_search/pdf.rs`

**Interfaces:**
- Consumes: `crate::m4_search::MemberDetailChild`, `format_amount` (Task 3).
- Produces: `pub(super) fn direct_legs_table(children: &[crate::m4_search::MemberDetailChild]) -> genpdf::elements::TableLayout`
  — used by Task 5. Empty slice renders a table with header row only (no
  panic) — Task 5 decides whether to show the section at all when there are
  zero direct legs, matching the screen's own `{detail.directChildren.length > 0 && ...}` guard.

- [ ] **Step 1: Write the failing test**

```rust
    use crate::m4_search::MemberDetailChild;

    #[test]
    fn direct_legs_table_lists_every_leg_with_status_as_text() {
        let children = vec![
            MemberDetailChild { member_id: 100078, name: "Aarav Shah".into(), total_business_volume: 3_200, slab_pct: 10, is_active: true },
            MemberDetailChild { member_id: 100117, name: "Kunal Verma".into(), total_business_volume: 1_300, slab_pct: 6, is_active: false },
        ];
        let table = direct_legs_table(&children);
        let text = render_and_extract(table);
        assert!(text.contains("Aarav Shah"));
        assert!(text.contains("100078"));
        assert!(text.contains("3,200"));
        assert!(text.contains("Active"));
        assert!(text.contains("Kunal Verma"));
        assert!(text.contains("Inactive"), "Rule-28: inactive still displays, colour-plus-label");
    }

    #[test]
    fn direct_legs_table_with_no_children_still_renders_a_header() {
        let table = direct_legs_table(&[]);
        let text = render_and_extract(table);
        assert!(text.contains("Name"));
        assert!(text.contains("Total Business Volume"));
    }
```

- [ ] **Step 2: Run to verify it fails**

Run: `cd src-tauri && cargo test --lib m4_search::pdf::tests`
Expected: FAIL — `direct_legs_table` not defined.

- [ ] **Step 3: Implement**

Add to `pdf.rs`:

```rust
use crate::m4_search::MemberDetailChild;

pub(super) fn direct_legs_table(children: &[MemberDetailChild]) -> TableLayout {
    let mut table = TableLayout::new(vec![4, 2, 3, 2, 2]);

    let headers = ["Name", "Member #", "Total Business Volume", "Slab", "Status"];
    let mut row = table.row();
    for (i, h) in headers.iter().enumerate() {
        let mut p = Paragraph::new("");
        p.push_styled(*h, Style::new().bold().with_font_size(9).with_color(MUTED));
        if i == 2 {
            p.set_alignment(Alignment::Right);
        }
        row = row.element(p);
    }
    row.push().expect("a fixed 5-cell header row always has the right cell count");

    for child in children {
        let mut name = Paragraph::new("");
        name.push_styled(&child.name, Style::new());
        let mut id = Paragraph::new("");
        id.push_styled(child.member_id.to_string(), Style::new());
        let mut tbv = Paragraph::new("");
        tbv.push_styled(format_amount(child.total_business_volume), Style::new());
        tbv.set_alignment(Alignment::Right);
        let mut slab = Paragraph::new("");
        slab.push_styled(format!("{}%", child.slab_pct), Style::new().with_color(ACCENT));
        let mut status = Paragraph::new("");
        if child.is_active {
            status.push_styled("Active", Style::new().with_color(SUCCESS));
        } else {
            status.push_styled("Inactive", Style::new().with_color(DANGER));
        }
        table
            .row()
            .element(name)
            .element(id)
            .element(tbv)
            .element(slab)
            .element(status)
            .push()
            .expect("a fixed 5-cell row always has the right cell count");
    }

    table
}
```

- [ ] **Step 4: Run to verify it passes**

Run: `cd src-tauri && cargo test --lib m4_search::pdf::tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/m4_search/pdf.rs
git commit -m "feat(m4-pdf): render the direct legs table"
```

---

### Task 5: Full document assembly

**Files:**
- Modify: `src-tauri/src/m4_search/pdf.rs`

**Interfaces:**
- Consumes: `header_section`, `stat_boxes`, `mid_section`, `direct_legs_table`
  (Tasks 2-4); `crate::m4_search::MemberDetail`.
- Produces: `pub fn render_member_detail_pdf(detail: &MemberDetail, period_label: &str, generated_at: &str, output_path: &str) -> Result<(), crate::error::AppError>`
  — this is the function `m4_search::mod.rs` calls in Task 6.

- [ ] **Step 1: Write the failing test**

```rust
    use crate::m4_search::MemberDetail;

    fn sample_detail(direct_children: Vec<MemberDetailChild>) -> MemberDetail {
        let differentials: Vec<DifferentialLine> = direct_children
            .iter()
            .map(|c| differential_line(c.member_id, &c.name, c.total_business_volume, c.slab_pct, 12, 10))
            .collect();
        let royalty = if direct_children.is_empty() {
            None
        } else {
            Some(RoyaltyLine { qualifying_children: 1, rate_percent: 5, amount: 50 })
        };
        MemberDetail {
            member: sample_member(true, Some(100001)),
            total_business_volume: 8_450,
            slab_pct: 12,
            leg_count: direct_children.len() as i64,
            rewards: RewardBreakdown {
                own_reward: OwnRewardLine { own_business_volume: 1_200, own_slab_pct: 12, amount: 144 },
                differentials,
                royalty,
                rewards_total: 612,
            },
            direct_children,
        }
    }

    #[test]
    fn render_member_detail_pdf_writes_a_file_with_every_section() {
        let dir = std::env::temp_dir().join(format!("bvconsole-pdf-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let output_path = dir.join("member-detail.pdf");

        let children = vec![MemberDetailChild {
            member_id: 100078,
            name: "Aarav Shah".into(),
            total_business_volume: 3_200,
            slab_pct: 10,
            is_active: true,
        }];
        let detail = sample_detail(children);

        render_member_detail_pdf(&detail, "July 2026", "15 Aug 2026 14:32", output_path.to_str().unwrap())
            .expect("render must succeed");

        assert!(output_path.exists());
        let bytes = std::fs::read(&output_path).unwrap();
        let text = pdf_extract::extract_text_from_mem(&bytes).unwrap();
        assert!(text.contains("Ravi Patel"));
        assert!(text.contains("July 2026"));
        assert!(text.contains("Aarav Shah"));
        assert!(text.contains("Business Volume"));
        assert!(!text.contains("TBV"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn render_member_detail_pdf_with_zero_direct_legs_has_no_royalty_line() {
        let dir = std::env::temp_dir().join(format!("bvconsole-pdf-test-zero-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let output_path = dir.join("member-detail.pdf");

        let detail = sample_detail(vec![]);
        render_member_detail_pdf(&detail, "July 2026", "15 Aug 2026 14:32", output_path.to_str().unwrap())
            .expect("render must succeed");

        let bytes = std::fs::read(&output_path).unwrap();
        let text = pdf_extract::extract_text_from_mem(&bytes).unwrap();
        assert!(!text.contains("Royalty"), "no direct legs means no royalty line, matching the screen's own empty state");

        std::fs::remove_dir_all(&dir).ok();
    }
```

- [ ] **Step 2: Run to verify it fails**

Run: `cd src-tauri && cargo test --lib m4_search::pdf::tests`
Expected: FAIL — `render_member_detail_pdf` not defined.

- [ ] **Step 3: Implement**

Add to `pdf.rs`:

```rust
use genpdf::{Document, SimplePageDecorator, Size};

use crate::error::AppError;
use crate::m4_search::MemberDetail;

fn pdf_err(e: genpdf::error::Error) -> AppError {
    AppError::Export(e.to_string())
}

/// Assembles the full document and writes it to `output_path`. This is the
/// only function `m4_search::export_member_detail_pdf` calls — everything
/// else in this file is a private section builder.
pub fn render_member_detail_pdf(
    detail: &MemberDetail,
    period_label: &str,
    generated_at: &str,
    output_path: &str,
) -> Result<(), AppError> {
    let mut doc = Document::new(load_font_family());
    doc.set_title(format!("{} \u{2014} member record", detail.member.name));
    doc.set_paper_size(Size::new(210, 297)); // A4
    doc.set_font_size(11);

    let is_root = detail.member.introducer_member_id.is_none();

    let mut decorator = SimplePageDecorator::new();
    decorator.set_margins(Margins::trbl(18, 18, 16, 18));
    // genpdf 0.2 has no footer callback, only a running header — used here
    // for continuation pages of a long direct-legs table, per-page rather
    // than a bottom "Page N of M" (the total page count isn't known until
    // rendering finishes, so "of M" isn't available with this API).
    let member_name = detail.member.name.clone();
    decorator.set_header(move |page| {
        let mut layout = LinearLayout::vertical();
        if page > 1 {
            let mut p = Paragraph::new("");
            p.push_styled(
                format!("{member_name} \u{2014} continued \u{2014} page {page}"),
                Style::new().with_color(MUTED).with_font_size(9),
            );
            layout.push(p);
            layout.push(Break::new(0.5));
        }
        layout
    });
    doc.set_page_decorator(decorator);

    doc.push(header_section(
        &detail.member,
        detail.total_business_volume,
        detail.slab_pct,
        detail.rewards.own_reward.own_business_volume,
        detail.rewards.rewards_total,
        period_label,
        generated_at,
        is_root,
    ));
    doc.push(stat_boxes(
        detail.rewards.own_reward.own_business_volume,
        detail.total_business_volume,
        detail.slab_pct,
        detail.rewards.rewards_total,
    ));
    doc.push(Break::new(1));
    doc.push(mid_section(&detail.rewards, &detail.member, detail.leg_count));

    if !detail.direct_children.is_empty() {
        doc.push(Break::new(1));
        let mut section_title = Paragraph::new("");
        section_title.push_styled(
            format!("Direct legs ({})", detail.direct_children.len()),
            Style::new().bold().with_font_size(12),
        );
        doc.push(section_title);
        doc.push(direct_legs_table(&detail.direct_children));
    }

    doc.render_to_file(output_path).map_err(pdf_err)
}
```

- [ ] **Step 4: Run to verify it passes**

Run: `cd src-tauri && cargo test --lib m4_search::pdf::tests`
Expected: PASS, all `pdf` module tests green.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/m4_search/pdf.rs
git commit -m "feat(m4-pdf): assemble the full member detail PDF document"
```

---

### Task 6: Backend command — `export_member_detail_pdf`

**Files:**
- Modify: `src-tauri/src/m4_search/mod.rs`
- Modify: `src-tauri/Cargo.toml` (add `pdf-extract = "0.12.0"` under `[dev-dependencies]`,
  next to `proptest`)

**Interfaces:**
- Consumes: `get_member_detail` (existing), `pdf::render_member_detail_pdf` (Task 5),
  `crate::m6_reports::ExportResult { pub file_path: String }` (existing, reused).
- Produces: `pub fn export_member_detail_pdf(conn: &Connection, member_id: i64, period_month: Option<&str>, output_path: &str) -> Result<ExportResult, AppError>`
  — used by Task 7's `commands.rs` wrapper.

- [ ] **Step 1: Add the test dependency**

In `src-tauri/Cargo.toml`, right after `proptest = "1.8.0"` under `[dev-dependencies]`:

```toml
proptest = "1.8.0"
pdf-extract = "0.12.0"
```

- [ ] **Step 2: Write the failing test**

Add to the existing `mod tests` block at the bottom of `src-tauri/src/m4_search/mod.rs`
(around line 517), reusing the module's own `seeded()` and `insert_member()`
helpers (defined at lines 523-525 and 580-595) — the same ones
`get_member_detail_defaults_to_zero_with_no_activity_yet` (line 620) already
uses, including for a member with no period rows at all:

```rust
    #[test]
    fn export_member_detail_pdf_writes_a_real_file_matching_get_member_detail() {
        let conn = seeded();
        let root = insert_member(&conn, "Root", None);
        let dir = std::env::temp_dir().join(format!("bvconsole-export-pdf-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let output_path = dir.join("member.pdf");

        let result = export_member_detail_pdf(&conn, root, None, output_path.to_str().unwrap()).unwrap();

        assert_eq!(result.file_path, output_path.to_string_lossy());
        assert!(output_path.exists());
        let bytes = std::fs::read(&output_path).unwrap();
        let text = pdf_extract::extract_text_from_mem(&bytes).unwrap();
        assert!(text.contains("Root"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn export_member_detail_pdf_refuses_an_unknown_member() {
        let conn = seeded();
        let result = export_member_detail_pdf(&conn, 999_999, None, "unused.pdf");
        assert!(matches!(result, Err(AppError::NotFound { .. })));
    }
```

- [ ] **Step 3: Run to verify it fails**

Run: `cd src-tauri && cargo test --lib m4_search::tests::export_member_detail_pdf`
Expected: FAIL — `export_member_detail_pdf` not defined.

- [ ] **Step 4: Implement**

Add to `src-tauri/src/m4_search/mod.rs`, after `get_member_detail`:

```rust
use crate::m6_reports::ExportResult;

/// API-46 (CR-6, M4.8). Reuses `get_member_detail` unchanged — no new
/// calculation logic, same period resolution, same reward breakdown. The
/// period label shown on the document comes from whatever `period_month`
/// resolved to; `generated_at` is wall-clock time at export, not a stored
/// value (this document is a point-in-time snapshot, same spirit as the
/// full hierarchy window's own timestamp, Rule-45).
pub fn export_member_detail_pdf(
    conn: &Connection,
    member_id: i64,
    period_month: Option<&str>,
    output_path: &str,
) -> Result<ExportResult, AppError> {
    let detail = get_member_detail(conn, member_id, period_month)?;
    let period_id = resolve_view_period_id(conn, period_month)?;
    let period_label: String = conn.query_row(
        "SELECT period_month FROM periods WHERE id = ?1",
        [period_id],
        |r| r.get(0),
    )?;
    let generated_at = chrono::Local::now().format("%d %b %Y %H:%M").to_string();

    pdf::render_member_detail_pdf(&detail, &period_label, &generated_at, output_path)?;

    Ok(ExportResult {
        file_path: output_path.to_string(),
    })
}
```

- [ ] **Step 5: Run to verify it passes**

Run: `cd src-tauri && cargo test --lib m4_search`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/m4_search/mod.rs
git commit -m "feat(m4-pdf): add export_member_detail_pdf, reusing get_member_detail"
```

---

### Task 7: Tauri command wiring

**Files:**
- Modify: `src-tauri/src/command_names.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/capabilities/default.json`
- Modify: `src-tauri/tests/contract.rs`

**Interfaces:**
- Consumes: `m4_search::export_member_detail_pdf` (Task 6).
- Produces: the `export_member_detail_pdf` Tauri command, invokable from the
  frontend as `"export_member_detail_pdf"` — used by Task 8's IPC binding.

- [ ] **Step 1: Write the failing tests**

In `src-tauri/tests/contract.rs`, update the count assertion (around line 117-146):

```rust
// T-QA.2-3: the surface holds exactly 46 commands, API-01 to API-46, no
// gaps (amended for API-43/44 — see 06-decision-log-and-open-items.md;
// amended for API-45; amended again for API-46, CR-6's member detail PDF
// export) — and this list is the same one `lib.rs` feeds to
// `generate_handler!` and `build.rs` feeds to the ACL generator (via
// `command_names.rs`), so the three can never quietly drift apart.
#[test]
fn the_command_surface_holds_exactly_forty_six_commands() {
    assert_eq!(
        ALL_COMMAND_NAMES.len(),
        46,
        "API-01 to API-46, no gaps (C2)"
    );

    let capabilities = include_str!("../capabilities/default.json");
    let allow_count = capabilities.matches("\"allow-").count();
    assert_eq!(
        allow_count, 46,
        "the Tauri capability allowlist must have exactly 46 allow-* entries"
    );
    for name in ALL_COMMAND_NAMES {
        let slug = name.replace('_', "-");
        assert!(
            capabilities.contains(&format!("\"allow-{slug}\"")),
            "capabilities/default.json is missing allow-{slug} (for command {name})"
        );
    }
}
```

(Rename the function from `the_command_surface_holds_exactly_forty_five_commands`
to `..._forty_six_commands` — same test, updated name and numbers.)

Add two new tests near `export_monthly_requires_a_session`/`..._end_to_end...`
(around line 1398, right after `export_monthly_end_to_end_through_the_command_layer`):

```rust
#[test]
fn export_member_detail_pdf_requires_a_session() {
    let app = app_with_seeded_db();
    let result = commands::export_member_detail_pdf(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        1,
        None,
        "unused.pdf".into(),
    );
    assert!(matches!(result, Err(AppError::AuthRequired)));
}

#[test]
fn export_member_detail_pdf_end_to_end_through_the_command_layer() {
    let app = app_with_seeded_db();
    app.state::<SessionState>().mark_authenticated();
    let root = commands::create_root_member(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        root_input("9876599907"),
    )
    .unwrap();

    let output_dir = TempAppDir::new("export-member-detail-pdf-output");
    let output_path = output_dir.0.join("member.pdf");
    let result = commands::export_member_detail_pdf(
        app.state::<SessionState>(),
        app.state::<DbState>(),
        root.id,
        None,
        output_path.to_string_lossy().into_owned(),
    )
    .unwrap();

    assert_eq!(result.file_path, output_path.to_string_lossy());
    assert!(output_path.exists());
}
```

- [ ] **Step 2: Run to verify these fail**

Run: `cd src-tauri && cargo test --test contract export_member_detail_pdf`
Expected: FAIL to compile — `commands::export_member_detail_pdf` doesn't exist yet.

- [ ] **Step 3: Register the command — four files**

In `src-tauri/src/command_names.rs`, add to `ALL_COMMAND_NAMES` (after
`"add_closed_month_entry"`, near the other M4 entries — order within the array
doesn't matter functionally, but keep it near `"get_ancestor_chain"` for
readability):

```rust
    "get_ancestor_chain",
    "export_member_detail_pdf",
```

Update the doc comment above `ALL_COMMAND_NAMES`:

```rust
/// The complete, closed list — API-01 to API-46, no gaps (C2, amended for
/// API-43/44's addition, API-45's addition, and API-46's addition — see
/// 06-decision-log-and-open-items.md).
```

In `src-tauri/src/commands.rs`, add the wrapper after `get_ancestor_chain`:

```rust
/// API-46.
#[tauri::command]
pub fn export_member_detail_pdf(
    session: tauri::State<'_, SessionState>,
    db: tauri::State<'_, DbState>,
    member_id: i64,
    period_month: Option<String>,
    output_path: String,
) -> Result<m6_reports::ExportResult, AppError> {
    require_session(&session)?;
    let guard = locked_conn(&db);
    let conn = guard.as_ref().expect(
        "an authenticated session implies an open database connection — see S5's login flow",
    );
    m4_search::export_member_detail_pdf(conn, member_id, period_month.as_deref(), &output_path)
}
```

Also in `commands.rs`, add `"export_member_detail_pdf"` to the `HAS_REAL_LOGIC`
list inside `call_stub_by_name_covers_every_stub_command`'s test (this command
has real logic from day one, so it must not be counted as a stub):

```rust
            "get_ancestor_chain",
            "export_member_detail_pdf",
```

In `src-tauri/src/lib.rs`, add to the `generate_handler!` list (after
`commands::get_ancestor_chain,`, around line 55):

```rust
            commands::get_ancestor_chain,
            commands::export_member_detail_pdf,
```

In `src-tauri/capabilities/default.json`, add to `"permissions"` (after
`"allow-get-ancestor-chain",`):

```json
    "allow-get-ancestor-chain",
    "allow-export-member-detail-pdf",
```

- [ ] **Step 4: Run to verify the tests pass**

Run: `cd src-tauri && cargo test --test contract`
Expected: PASS — all contract tests, including the two new ones and the
updated 46-count assertion.

- [ ] **Step 5: Run the full Rust suite**

Run: `cd src-tauri && cargo test`
Expected: PASS, no regressions anywhere in the workspace.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/command_names.rs src-tauri/src/commands.rs src-tauri/src/lib.rs \
  src-tauri/capabilities/default.json src-tauri/tests/contract.rs
git commit -m "feat(m4-pdf): wire export_member_detail_pdf into the Tauri command surface (API-46)"
```

---

### Task 8: Frontend — IPC binding and the Export PDF button

**Files:**
- Modify: `src/lib/ipc/m4-search.ts`
- Modify: `src/screens/member-detail.tsx`

**Interfaces:**
- Consumes: the `export_member_detail_pdf` Tauri command (Task 7).
- Produces: `exportMemberDetailPdf(memberId: number, periodMonth: string | undefined, outputPath: string): Promise<ExportResult>`,
  imported by `member-detail.tsx`.

**No new frontend test is added in this task.** This codebase's four other
export buttons (`reports.tsx`'s `.xlsx` exports) and its restore-from-file flow
all use the identical save-dialog-then-invoke shape, and none of them has a
component test for it (`grep -rl "plugin-dialog" src` matches zero `*.test.tsx`
files at time of writing) — the design spec's testing section assumed one
would exist here, but adding it would establish a new pattern this codebase
doesn't otherwise follow, for a button whose only logic is "call a save dialog,
then call an already-tested command." Coverage for the actual behaviour lives
in Task 5-7's Rust tests. Flagging this as a deliberate deviation from the
spec's §6, not a silent drop.

- [ ] **Step 1: Add the IPC binding**

In `src/lib/ipc/m4-search.ts`, add near the top (after the `ExportResult`-shaped
types aren't defined in this file yet — import it from `m6-reports.ts` instead
of redefining, exactly as the Rust side reuses `m6_reports::ExportResult`):

```typescript
import type { ExportResult } from "./m6-reports";
```

Add at the end of the file:

```typescript
// API-46 (CR-6, M4.8) — reuses get_member_detail's data unchanged; no new
// calculation logic. `periodMonth`: same default-to-oldest-recordable
// behaviour as `getMemberDetail`.
export function exportMemberDetailPdf(
  memberId: number,
  periodMonth: string | undefined,
  outputPath: string,
): Promise<ExportResult> {
  return invokeCommand("export_member_detail_pdf", { memberId, periodMonth, outputPath });
}
```

- [ ] **Step 2: Add the button and handler**

In `src/screens/member-detail.tsx`, add to the imports:

```typescript
import { Download } from "lucide-react";
import { save as saveFileDialog } from "@tauri-apps/plugin-dialog";
```

and change the `getMemberDetail` import line to also pull in the new function:

```typescript
import { getMemberDetail, exportMemberDetailPdf } from "@/lib/ipc/m4-search";
```

Add state near the other `useState` calls (around line 46):

```typescript
  const [exportingPdf, setExportingPdf] = useState(false);
```

Add a handler near `handleConfirm` (around line 97):

```typescript
  async function handleExportPdf() {
    if (!detail) return;
    const outputPath = await saveFileDialog({
      defaultPath: `member-detail-${detail.member.id}-${viewMonth ?? "current"}.pdf`,
      filters: [{ name: "PDF Document", extensions: ["pdf"] }],
    });
    if (!outputPath) return;
    setExportingPdf(true);
    try {
      await exportMemberDetailPdf(detail.member.id, viewMonth, outputPath);
      toast.add({ title: "Member record exported", type: "success" });
    } catch (raw) {
      toast.add({ title: toErrorPresentation(raw).message, type: "danger" });
    } finally {
      setExportingPdf(false);
    }
  }
```

Add the button next to "Record volume" (around line 143-149):

```tsx
            <Button
              variant="secondary"
              size="sm"
              disabled={exportingPdf}
              onClick={handleExportPdf}
            >
              <Download />
              Export PDF
            </Button>
            <Button
              variant="primary"
              size="sm"
              onClick={() => navigate(`/entry?member=${member.id}`)}
            >
              Record volume
            </Button>
```

- [ ] **Step 3: Type-check and lint**

Run: `npx tsc --noEmit` (the project's `build` script runs `tsc` with no
dedicated `typecheck` script — this is the same check without the Vite build)
Run: `npm run lint`
Expected: both pass, no new errors.

- [ ] **Step 4: Run the vocabulary guard**

Run: `node scripts/vocabulary-grep.mjs`
Expected: `vocabulary grep: clean`

- [ ] **Step 5: Manual verification**

Run: `npm run tauri dev` (launches the Tauri shell so `invoke`/the native save
dialog actually work — plain `npm run dev` only serves the Vite frontend in a
browser, where Tauri's IPC bridge isn't present). Navigate to any member's
detail screen, click "Export PDF", save to a known path, open the resulting
file and visually confirm: identity, four stat boxes, rewards detail table,
member details, direct legs table (if any), every label spelled in full, no
currency symbol, no company branding.

- [ ] **Step 6: Commit**

```bash
git add src/lib/ipc/m4-search.ts src/screens/member-detail.tsx
git commit -m "feat(m4-pdf): add Export PDF button to the member detail screen"
```

---

### Task 9: Full verification pass

**Files:** none (verification only)

- [ ] **Step 1: Full Rust test suite**

Run: `cd src-tauri && cargo test`
Expected: PASS, 46-command contract test included.

- [ ] **Step 2: Full frontend test suite**

Run: `npm test`
Expected: PASS, no regressions.

- [ ] **Step 3: Frontend build**

Run: `npm run build`
Expected: succeeds, no type errors.

- [ ] **Step 4: Rust release build sanity check**

Run: `cd src-tauri && cargo build --release`
Expected: succeeds — confirms the embedded font assets and the `genpdf`
dependency build cleanly in release mode (`opt-level = "s"`, `lto = true` per
`Cargo.toml`'s `[profile.release]`), not just in debug.

- [ ] **Step 5: Vocabulary guard, one more time**

Run: `node scripts/vocabulary-grep.mjs`
Expected: `vocabulary grep: clean`

- [ ] **Step 6: Final commit (if any verification step required fixes)**

```bash
git add -A
git commit -m "chore(m4-pdf): fix verification-pass findings"
```

(Skip this step if Steps 1-5 all passed with no changes needed.)

---

## Self-Review Notes

- **Spec coverage:** Problem/Scope (§1-2) → Tasks 5-8. Architecture (§3, incl. the
  genpdf-over-printpdf risk) → Tasks 1, 3 (the spike), 6-7. Visual design (§4) →
  Tasks 2-4. Error handling (§5) → Task 6 (`pdf_err`, reuses `AppError::Export`).
  Testing (§6) → every task's TDD steps; the frontend-test deviation is called
  out explicitly in Task 8 rather than silently dropped. Out of scope (§7) →
  nothing in this plan touches full-downline aggregation, bulk export, a period
  picker, or branding.
- **Font gap:** not in the original spec — found during planning (genpdf has no
  built-in font) and resolved by vendoring static Inter TTFs at
  `src-tauri/assets/fonts/`, recorded in the spec's amended §3.
- **Type consistency:** `ExportResult` is defined once (`m6_reports`) and reused
  by both `m4_search::export_member_detail_pdf` (Task 6) and the frontend
  binding (Task 8) — not redefined.
