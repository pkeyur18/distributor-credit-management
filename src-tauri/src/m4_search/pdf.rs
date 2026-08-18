//! M4.8 — PDF rendering for the member detail export (CR-6, ADR-013).
//! Pure rendering: every value here comes from an already-computed
//! `MemberDetail` (m4_search::get_member_detail) — no calculation happens
//! in this module.
//!
//! **Testing approach (revised during implementation, 15 Aug 2026):** the
//! design spec originally called for asserting on PDF-extracted text.
//! `pdf-extract`'s `adobe-cmap-parser` dependency panics on *any* text
//! genpdf/printpdf-0.3.4 produces — confirmed with plain ASCII, digits, and
//! genpdf's own minimal-conformance mode, all crash identically. So: every
//! section builder below has its actual text composed by a small pure
//! function (`status_label`, `meta_line_text`, etc.) tested directly with
//! no PDF involved; the section builders and `render_member_detail_pdf`
//! itself are smoke-tested only (render succeeds, output is non-empty and
//! starts with the `%PDF` magic bytes). The one exception is the
//! pagination spike (below), which reads the rendered PDF's page *count*
//! via `lopdf` — raw object-structure access, no CMap/glyph decoding, so
//! it doesn't hit the same crash.

use genpdf::elements::{
    Break, CellDecorator, FrameCellDecorator, LinearLayout, PaddedElement, Paragraph, TableLayout,
};
use genpdf::fonts::{FontData, FontFamily};
use genpdf::render;
use genpdf::style::{Color, Style};
use genpdf::{Alignment, Document, Element, Margins, Position, SimplePageDecorator, Size};

use crate::error::AppError;
use crate::m1_members::Member;
use crate::m4_search::{MemberDetail, MemberDetailChild, RewardBreakdown};

// 07-design-system.md §1: accent #4f46e5, ledger green #059669, red #dc2626.
const ACCENT: Color = Color::Rgb(0x4f, 0x46, 0xe5);
const SUCCESS: Color = Color::Rgb(0x05, 0x96, 0x69);
const DANGER: Color = Color::Rgb(0xdc, 0x26, 0x26);
const MUTED: Color = Color::Rgb(0x64, 0x74, 0x8b);
const INK: Color = Color::Rgb(0x0f, 0x17, 0x2a);
// src/index.css's `--border` token — the exact hairline colour every
// `border-border` line on screen (table.tsx, StatCard) already uses.
const BORDER: Color = Color::Rgb(0xe2, 0xe8, 0xf0);

/// The on-screen table's own hairline convention (table.tsx: `TableWrap`'s
/// outer border, `TableHead`/`TableRow`'s `border-b`) — a full outer box
/// plus a horizontal line under the header and under every row, no
/// vertical lines between columns. genpdf's own `FrameCellDecorator`
/// couples inner horizontal and vertical lines together, which would add
/// column lines the approved on-screen design doesn't have, so this
/// exists instead. Always draws in `BORDER`, ignoring whatever ambient
/// style the table renders with — `draw_line` only reads `style.color()`,
/// so this can't accidentally recolour any text.
#[derive(Default)]
struct RowLineCellDecorator {
    num_columns: usize,
}

impl CellDecorator for RowLineCellDecorator {
    fn set_table_size(&mut self, num_columns: usize, _num_rows: usize) {
        self.num_columns = num_columns;
    }

    fn decorate_cell(
        &mut self,
        column: usize,
        row: usize,
        has_more: bool,
        area: render::Area<'_>,
        _style: Style,
    ) {
        let size = area.size();
        let line_style = Style::new().with_color(BORDER);

        if column == 0 {
            area.draw_line(
                vec![Position::default(), Position::new(0, size.height)],
                line_style,
            );
        }
        if column + 1 == self.num_columns {
            area.draw_line(
                vec![
                    Position::new(size.width, 0),
                    Position::new(size.width, size.height),
                ],
                line_style,
            );
        }
        if row == 0 {
            area.draw_line(
                vec![Position::default(), Position::new(size.width, 0)],
                line_style,
            );
        }
        // Skipped when `has_more`: this row continues onto the next page,
        // so the line belongs under whichever page it actually finishes
        // on, not here.
        if !has_more {
            area.draw_line(
                vec![
                    Position::new(0, size.height),
                    Position::new(size.width, size.height),
                ],
                line_style,
            );
        }
    }
}

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

/// Active/Inactive as a plain label — the colour is layered on top of this
/// text in `header_section`, never a substitute for it (07-design-
/// system.md's Colour-Plus-Label Rule).
pub(super) fn status_label(is_active: bool) -> &'static str {
    if is_active {
        "Active"
    } else {
        "Inactive"
    }
}

/// "Member #100042 · +91 98765 43210 · Joined 2024-03-12" — pulled out as a
/// pure function so the exact composed string is testable without
/// rendering a PDF (see the module doc comment on why: `pdf-extract`
/// cannot read genpdf's output).
pub(super) fn meta_line_text(member: &Member) -> String {
    format!(
        "Member #{} \u{b7} {} \u{b7} Joined {}",
        member.id, member.phone, member.joining_date
    )
}

/// "Period July 2026 · Generated 15 Aug 2026 14:32"
pub(super) fn period_line_text(period_label: &str, generated_at: &str) -> String {
    format!("Period {period_label} \u{b7} Generated {generated_at}")
}

/// Header block: name, active/inactive, member number, phone, joining date,
/// period, generation timestamp. The text this produces is composed by
/// `status_label`, `meta_line_text` and `period_line_text` above — test
/// those directly for content; this function is only smoke-tested for
/// "renders without panicking" (see the module doc comment).
pub(super) fn header_section(
    member: &Member,
    period_label: &str,
    generated_at: &str,
) -> LinearLayout {
    let mut layout = LinearLayout::vertical();

    let mut name_line = Paragraph::new("");
    name_line.push_styled(
        &member.name,
        Style::new().bold().with_font_size(15).with_color(INK),
    );
    name_line.push_styled("   ", Style::new());
    let status_color = if member.is_active { SUCCESS } else { DANGER };
    name_line.push_styled(
        status_label(member.is_active),
        Style::new().bold().with_color(status_color),
    );
    layout.push(name_line);

    let mut meta_line = Paragraph::new("");
    meta_line.push_styled(
        meta_line_text(member),
        Style::new().with_color(MUTED).with_font_size(10),
    );
    layout.push(meta_line);

    let mut period_line = Paragraph::new("");
    period_line.push_styled(
        period_line_text(period_label, generated_at),
        Style::new().with_color(MUTED).with_font_size(10),
    );
    period_line.set_alignment(Alignment::Right);
    layout.push(period_line);

    layout.push(Break::new(1));
    layout
}

/// Thousands-separated, no decimals — Business Volume is always a whole
/// number (Rule-16a: zero/negative entries are refused, not just
/// fractional cents beyond what ADR-004's fixed-point scale allows).
pub(super) fn format_amount(value: i64) -> String {
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
    if negative {
        format!("-{grouped}")
    } else {
        grouped
    }
}

/// The four stat-box (label, value) pairs, in display order. Every label
/// spelled in full (CR-6) — no "BV"/"TBV".
pub(super) fn stat_box_values(
    business_volume: i64,
    total_business_volume: i64,
    slab_pct: i64,
    rewards_total: i64,
) -> [(&'static str, String); 4] {
    [
        ("Business Volume", format_amount(business_volume)),
        (
            "Total Business Volume",
            format_amount(total_business_volume),
        ),
        ("Slab", format!("{slab_pct}%")),
        ("Rewards this period", format_amount(rewards_total)),
    ]
}

/// One row of the rewards-detail table, already formatted for display.
pub(super) struct RewardRow {
    pub description: String,
    pub business_volume: Option<String>,
    pub amount: String,
    pub emphasized: bool,
}

/// Own-Business-Volume reward first (Rule-46/CR-4), then every direct
/// leg's differential line, then royalty, then the total — same order the
/// screen shows (member-detail.tsx's Rewards detail table).
pub(super) fn rewards_detail_rows(rewards: &RewardBreakdown) -> Vec<RewardRow> {
    let mut rows = vec![RewardRow {
        description: format!(
            "Own Business Volume \u{2014} {}%",
            rewards.own_reward.own_slab_pct
        ),
        business_volume: Some(format_amount(rewards.own_reward.own_business_volume)),
        amount: format_amount(rewards.own_reward.amount),
        emphasized: true,
    }];

    for line in &rewards.differentials {
        rows.push(RewardRow {
            description: line.child_name.clone(),
            business_volume: Some(format_amount(line.child_total_business_volume)),
            amount: format_amount(line.amount),
            emphasized: false,
        });
    }

    if let Some(royalty) = &rewards.royalty {
        rows.push(RewardRow {
            description: format!(
                "Royalty \u{2014} {} of {} legs qualifying",
                royalty.qualifying_children,
                rewards.differentials.len()
            ),
            business_volume: None,
            amount: format_amount(royalty.amount),
            emphasized: true,
        });
    }

    rows.push(RewardRow {
        description: "Rewards total".into(),
        business_volume: None,
        amount: format_amount(rewards.rewards_total),
        emphasized: true,
    });

    rows
}

/// The member Details block's (label, value) pairs — address, email,
/// introducer link, direct-leg count, consent date, same fields the
/// screen's Details card shows.
pub(super) fn details_rows(member: &Member, leg_count: i64) -> Vec<(&'static str, String)> {
    vec![
        ("Address", member.address.clone()),
        (
            "Email",
            member
                .email
                .clone()
                .unwrap_or_else(|| "Not provided".into()),
        ),
        (
            "Introduced by",
            member
                .introducer_member_id
                .map(|id| format!("#{id}"))
                .unwrap_or_else(|| "None \u{2014} root member".into()),
        ),
        ("Direct legs", leg_count.to_string()),
        ("Consent captured", member.consent_date.clone()),
    ]
}

fn stat_box(label: &str, value: String) -> PaddedElement<LinearLayout> {
    let mut inner = LinearLayout::vertical();
    let mut label_p = Paragraph::new("");
    label_p.push_styled(label, Style::new().with_color(MUTED).with_font_size(9));
    inner.push(label_p);
    let mut value_p = Paragraph::new("");
    value_p.push_styled(
        value,
        Style::new().bold().with_font_size(13).with_color(INK),
    );
    inner.push(value_p);
    inner.padded(Margins::all(4))
}

/// Four stat boxes, one row, equal width — mirrors the screen's own
/// `grid-cols-4` stat strip.
pub(super) fn stat_boxes(
    business_volume: i64,
    total_business_volume: i64,
    slab_pct: i64,
    rewards_total: i64,
) -> impl Element {
    let values = stat_box_values(
        business_volume,
        total_business_volume,
        slab_pct,
        rewards_total,
    );
    let mut table = TableLayout::new(vec![1, 1, 1, 1]);
    let mut row = table.row();
    for (label, value) in values {
        row = row.element(stat_box(label, value));
    }
    row.push()
        .expect("a fixed 4-cell row always has the right cell count");
    // On screen each stat is its own separately-bordered `StatCard` — a
    // single-row, 4-column grid with both inner and outer lines produces
    // the same four boxes. Continuation borders don't apply: this table
    // never spans a page break. `FrameCellDecorator` draws in whatever
    // ambient style it's rendered with, so the whole table is wrapped in
    // `BORDER` here — each stat box's own label/value colours still win
    // for their own text, since `Style::merge` lets the more specific
    // (child) style override the ambient one.
    table.set_cell_decorator(FrameCellDecorator::new(true, true, false));
    table.styled(Style::new().with_color(BORDER))
}

/// Every table cell in this file goes through this — genpdf's `TableLayout`
/// columns sit flush against each other with no gutter, which (found
/// during the Task 8 visual check) makes right-aligned neighbouring
/// columns run together, e.g. "3,20010%" with no gap between the amount
/// and the next column. A little right-padding and row spacing fixes it.
fn cell(p: Paragraph) -> PaddedElement<Paragraph> {
    p.padded(Margins::trbl(0, 4, 3, 0))
}

/// Column weights `[4, 4, 5]`, not the more description-heavy `[6, 3, 2]`
/// this started as: measuring actual glyph widths (see `mid_section`'s doc
/// comment) against a comma-formatted 8-digit worst case showed `[6, 3, 2]`
/// left Business Volume and Amount too narrow once this table is nested
/// beside the Details column, and genpdf drops any single number wider
/// than its column with no error. "Leg" text is mostly names and short
/// phrases that wrap safely across multiple words, so it can afford to
/// give width back to the two numeric columns.
fn rewards_detail_table(rewards: &RewardBreakdown) -> TableLayout {
    let mut table = TableLayout::new(vec![4, 4, 5]);

    let mut leg_header = Paragraph::new("");
    leg_header.push_styled(
        "Leg",
        Style::new().bold().with_font_size(9).with_color(MUTED),
    );
    let mut bv_header = Paragraph::new("");
    bv_header.push_styled(
        "Business Volume",
        Style::new().bold().with_font_size(9).with_color(MUTED),
    );
    bv_header.set_alignment(Alignment::Right);
    let mut amt_header = Paragraph::new("");
    amt_header.push_styled(
        "Amount",
        Style::new().bold().with_font_size(9).with_color(MUTED),
    );
    amt_header.set_alignment(Alignment::Right);
    table
        .row()
        .element(cell(leg_header))
        .element(cell(bv_header))
        .element(cell(amt_header))
        .push()
        .unwrap();

    for row in rewards_detail_rows(rewards) {
        let mut desc = Paragraph::new("");
        let desc_style = if row.emphasized {
            Style::new().bold()
        } else {
            Style::new()
        };
        desc.push_styled(row.description, desc_style);

        let mut bv = Paragraph::new("");
        if let Some(bv_text) = row.business_volume {
            bv.push_styled(bv_text, Style::new());
        }
        bv.set_alignment(Alignment::Right);

        let mut amt = Paragraph::new("");
        amt.push_styled(row.amount, Style::new().bold());
        amt.set_alignment(Alignment::Right);

        table
            .row()
            .element(cell(desc))
            .element(cell(bv))
            .element(cell(amt))
            .push()
            .unwrap();
    }

    table.set_cell_decorator(RowLineCellDecorator::default());
    table
}

fn details_block(member: &Member, leg_count: i64) -> LinearLayout {
    let mut layout = LinearLayout::vertical();
    for (label, value) in details_rows(member, leg_count) {
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
/// member-detail.tsx:175): Rewards detail beside member Details.
///
/// Found 15 Aug 2026 via a real export with 8 direct legs: some Amount
/// cells rendered blank — no panic, no error. Root cause, confirmed by
/// measuring actual glyph widths against the column budget (not a
/// TableLayout-nesting issue, despite it looking that way from the
/// symptom): genpdf's word wrapper (`wrap::Wrapper::next`, genpdf 0.2.0)
/// silently discards a word — and returns no error — the instant that
/// word's rendered width exceeds the *entire* available column width, not
/// just the remaining space on the current line. `rewards_detail_table`'s
/// old column weights (`vec![6, 3, 2]`) left the Amount column only
/// ~13.4mm wide once nested here, and any amount whose comma-formatted
/// width crossed that line (e.g. "24,524" at 13.6mm) vanished, while
/// shorter ones (e.g. "72,598" at 13.3mm) survived — explaining why the
/// drops looked scattered rather than a clean cutoff. Fixed at the source
/// in `rewards_detail_table`'s own column weights, not here.
pub(super) fn mid_section(
    rewards: &RewardBreakdown,
    member: &Member,
    leg_count: i64,
) -> TableLayout {
    let mut rewards_wrapper = LinearLayout::vertical();
    rewards_wrapper.push(rewards_detail_table(rewards));

    let mut outer = TableLayout::new(vec![7, 5]);
    outer
        .row()
        .element(rewards_wrapper.padded(Margins::all(3)))
        .element(details_block(member, leg_count).padded(Margins::all(3)))
        .push()
        .expect("a fixed 2-cell row always has the right cell count");
    outer
}

/// One row of the direct-legs table, already formatted for display.
pub(super) struct DirectLegRow {
    pub name: String,
    pub member_id: String,
    pub total_business_volume: String,
    pub slab_pct: String,
    pub status: &'static str,
}

/// Same columns as the screen's own "Direct legs" table (member-
/// detail.tsx:301-339): name, member number, Total Business Volume, slab,
/// status.
pub(super) fn direct_leg_rows(children: &[MemberDetailChild]) -> Vec<DirectLegRow> {
    children
        .iter()
        .map(|c| DirectLegRow {
            name: c.name.clone(),
            member_id: c.member_id.to_string(),
            total_business_volume: format_amount(c.total_business_volume),
            slab_pct: format!("{}%", c.slab_pct),
            status: status_label(c.is_active),
        })
        .collect()
}

/// Empty `children` still renders a table with a header row only — Task 5
/// decides whether to show this section at all when there are zero direct
/// legs, matching the screen's own `{detail.directChildren.length > 0 &&
/// ...}` guard.
pub(super) fn direct_legs_table(children: &[MemberDetailChild]) -> TableLayout {
    let mut table = TableLayout::new(vec![4, 2, 3, 2, 2]);

    let headers = [
        "Name",
        "Member #",
        "Total Business Volume",
        "Slab",
        "Status",
    ];
    let mut row = table.row();
    for (i, h) in headers.iter().enumerate() {
        let mut p = Paragraph::new("");
        p.push_styled(*h, Style::new().bold().with_font_size(9).with_color(MUTED));
        if i == 2 {
            p.set_alignment(Alignment::Right);
        }
        row = row.element(cell(p));
    }
    row.push()
        .expect("a fixed 5-cell header row always has the right cell count");

    for leg in direct_leg_rows(children) {
        let mut name = Paragraph::new("");
        name.push_styled(leg.name, Style::new());
        let mut id = Paragraph::new("");
        id.push_styled(leg.member_id, Style::new());
        let mut tbv = Paragraph::new("");
        tbv.push_styled(leg.total_business_volume, Style::new());
        tbv.set_alignment(Alignment::Right);
        let mut slab = Paragraph::new("");
        slab.push_styled(leg.slab_pct, Style::new().with_color(ACCENT));
        let mut status = Paragraph::new("");
        let status_color = if leg.status == "Active" {
            SUCCESS
        } else {
            DANGER
        };
        status.push_styled(leg.status, Style::new().with_color(status_color));

        table
            .row()
            .element(cell(name))
            .element(cell(id))
            .element(cell(tbv))
            .element(cell(slab))
            .element(cell(status))
            .push()
            .expect("a fixed 5-cell row always has the right cell count");
    }

    table.set_cell_decorator(RowLineCellDecorator::default());
    table
}

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

    doc.push(header_section(&detail.member, period_label, generated_at));
    doc.push(stat_boxes(
        detail.rewards.own_reward.own_business_volume,
        detail.total_business_volume,
        detail.slab_pct,
        detail.rewards.rewards_total,
    ));
    doc.push(Break::new(1));
    doc.push(mid_section(
        &detail.rewards,
        &detail.member,
        detail.leg_count,
    ));

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m4_search::{DifferentialLine, OwnRewardLine, RoyaltyLine};

    fn sample_member(is_active: bool, introducer_member_id: Option<i64>) -> Member {
        Member {
            id: 896044,
            name: "Asha Verma".into(),
            phone: "+91 98765 43210".into(),
            email: Some("asha@example.com".into()),
            address: "12 MG Road, Pune".into(),
            introducer_member_id,
            level: 2,
            is_active,
            joining_date: "2024-03-12".into(),
            consent_given: true,
            consent_date: "2024-03-12".into(),
            created_at: "2024-03-12T10:00:00Z".into(),
        }
    }

    fn differential_line(
        child_id: i64,
        child_name: &str,
        child_total_business_volume: i64,
        child_slab_pct: i64,
        own_slab_pct: i64,
        amount: i64,
    ) -> DifferentialLine {
        DifferentialLine {
            child_id,
            child_name: child_name.to_string(),
            child_total_business_volume,
            child_slab_pct,
            own_slab_pct,
            differential_pct: own_slab_pct - child_slab_pct,
            amount,
        }
    }

    fn render_bytes(element: impl Element + 'static) -> Vec<u8> {
        let mut doc = Document::new(load_font_family());
        doc.set_paper_size(Size::new(210, 297));
        doc.push(element);
        let mut bytes = Vec::new();
        doc.render(&mut bytes)
            .expect("a minimal document must render");
        bytes
    }

    #[test]
    fn status_label_is_active_or_inactive() {
        assert_eq!(status_label(true), "Active");
        assert_eq!(status_label(false), "Inactive");
    }

    #[test]
    fn meta_line_text_composes_member_number_phone_and_joining_date() {
        let member = sample_member(true, None);
        assert_eq!(
            meta_line_text(&member),
            "Member #896044 \u{b7} +91 98765 43210 \u{b7} Joined 2024-03-12"
        );
    }

    #[test]
    fn period_line_text_composes_period_and_generated_timestamp() {
        assert_eq!(
            period_line_text("2026-07", "15 Aug 2026 14:32"),
            "Period 2026-07 \u{b7} Generated 15 Aug 2026 14:32"
        );
    }

    #[test]
    fn format_amount_groups_thousands_and_keeps_the_sign() {
        assert_eq!(format_amount(0), "0");
        assert_eq!(format_amount(7), "7");
        assert_eq!(format_amount(999), "999");
        assert_eq!(format_amount(1_000), "1,000");
        assert_eq!(format_amount(9_585_639), "9,585,639");
        assert_eq!(format_amount(-2_500), "-2,500");
    }

    #[test]
    fn stat_box_values_spells_out_every_label_in_full() {
        let values = stat_box_values(896_044, 9_585_639, 14, 169_933);
        assert_eq!(
            values,
            [
                ("Business Volume", "896,044".to_string()),
                ("Total Business Volume", "9,585,639".to_string()),
                ("Slab", "14%".to_string()),
                ("Rewards this period", "169,933".to_string()),
            ]
        );
    }

    #[test]
    fn rewards_detail_rows_orders_own_reward_first_then_legs_then_royalty_then_total() {
        let rewards = RewardBreakdown {
            own_reward: OwnRewardLine {
                own_business_volume: 100_000,
                own_slab_pct: 14,
                amount: 14_000,
            },
            differentials: vec![
                differential_line(1, "Mohit Shah", 2_147_185, 14, 14, 0),
                differential_line(2, "Diya Patel", 118_847, 4, 14, 11_885),
            ],
            royalty: Some(RoyaltyLine {
                qualifying_children: 1,
                rate_percent: 5.0,
                amount: 5_942,
            }),
            rewards_total: 31_827,
        };
        let rows = rewards_detail_rows(&rewards);
        let descriptions: Vec<&str> = rows.iter().map(|r| r.description.as_str()).collect();
        assert_eq!(
            descriptions,
            vec![
                "Own Business Volume \u{2014} 14%",
                "Mohit Shah",
                "Diya Patel",
                "Royalty \u{2014} 1 of 2 legs qualifying",
                "Rewards total",
            ]
        );
        assert_eq!(rows[0].amount, "14,000");
        assert_eq!(rows[1].amount, "0");
        assert_eq!(rows[2].amount, "11,885");
        assert_eq!(rows[3].amount, "5,942");
        assert_eq!(rows[4].amount, "31,827");
        assert!(rows[0].emphasized);
        assert!(!rows[1].emphasized);
        assert!(rows[3].emphasized);
        assert!(rows[4].emphasized);
    }

    #[test]
    fn rewards_detail_rows_omits_royalty_row_when_there_is_no_royalty() {
        let rewards = RewardBreakdown {
            own_reward: OwnRewardLine {
                own_business_volume: 0,
                own_slab_pct: 14,
                amount: 0,
            },
            differentials: vec![],
            royalty: None,
            rewards_total: 0,
        };
        let rows = rewards_detail_rows(&rewards);
        assert_eq!(rows.len(), 2, "own reward + total only, no royalty row");
        assert_eq!(rows[1].description, "Rewards total");
    }

    #[test]
    fn details_rows_shows_introducer_link_and_direct_leg_count() {
        let member = sample_member(true, Some(42));
        let rows = details_rows(&member, 8);
        let map: std::collections::HashMap<_, _> = rows.into_iter().collect();
        assert_eq!(map["Introduced by"], "#42");
        assert_eq!(map["Direct legs"], "8");
        assert_eq!(map["Address"], "12 MG Road, Pune");
        assert_eq!(map["Email"], "asha@example.com");
    }

    #[test]
    fn details_rows_for_the_root_member_names_them_as_root() {
        let member = sample_member(true, None);
        let rows = details_rows(&member, 0);
        let map: std::collections::HashMap<_, _> = rows.into_iter().collect();
        assert_eq!(map["Introduced by"], "None \u{2014} root member");
    }

    #[test]
    fn details_rows_uses_a_placeholder_when_email_is_missing() {
        let mut member = sample_member(true, None);
        member.email = None;
        let rows = details_rows(&member, 0);
        let map: std::collections::HashMap<_, _> = rows.into_iter().collect();
        assert_eq!(map["Email"], "Not provided");
    }

    #[test]
    fn direct_leg_rows_formats_every_field_and_status_as_text() {
        let children = vec![
            MemberDetailChild {
                member_id: 5,
                name: "Kavya Reddy".into(),
                total_business_volume: 613_088,
                slab_pct: 10,
                is_active: true,
            },
            MemberDetailChild {
                member_id: 6,
                name: "Neha Joshi".into(),
                total_business_volume: 467_811,
                slab_pct: 8,
                is_active: false,
            },
        ];
        let rows = direct_leg_rows(&children);
        assert_eq!(rows[0].name, "Kavya Reddy");
        assert_eq!(rows[0].member_id, "5");
        assert_eq!(rows[0].total_business_volume, "613,088");
        assert_eq!(rows[0].slab_pct, "10%");
        assert_eq!(rows[0].status, "Active");
        assert_eq!(rows[1].status, "Inactive");
    }

    #[test]
    fn header_section_renders_without_panicking() {
        let member = sample_member(true, None);
        let bytes = render_bytes(header_section(&member, "2026-07", "15 Aug 2026 14:32"));
        assert!(bytes.starts_with(b"%PDF"));
    }

    #[test]
    fn stat_boxes_renders_without_panicking() {
        let bytes = render_bytes(stat_boxes(896_044, 9_585_639, 14, 169_933));
        assert!(bytes.starts_with(b"%PDF"));
    }

    #[test]
    fn mid_section_renders_without_panicking() {
        let member = sample_member(true, Some(42));
        let rewards = RewardBreakdown {
            own_reward: OwnRewardLine {
                own_business_volume: 0,
                own_slab_pct: 14,
                amount: 0,
            },
            differentials: vec![differential_line(1, "Mohit Shah", 2_147_185, 14, 14, 0)],
            royalty: None,
            rewards_total: 0,
        };
        let bytes = render_bytes(mid_section(&rewards, &member, 1));
        assert!(bytes.starts_with(b"%PDF"));
    }

    #[test]
    fn direct_legs_table_renders_with_zero_or_many_legs() {
        assert!(render_bytes(direct_legs_table(&[])).starts_with(b"%PDF"));
        let children: Vec<MemberDetailChild> = (0..5)
            .map(|i| MemberDetailChild {
                member_id: i,
                name: format!("Member {i}"),
                total_business_volume: 1_000 * i,
                slab_pct: 10,
                is_active: true,
            })
            .collect();
        assert!(render_bytes(direct_legs_table(&children)).starts_with(b"%PDF"));
    }

    #[test]
    fn mid_section_with_many_direct_legs_renders_more_than_one_page() {
        let member = sample_member(true, Some(1));
        let differentials: Vec<DifferentialLine> = (1..=80)
            .map(|i| differential_line(i, &format!("Leg {i}"), 10_000, 10, 14, 10))
            .collect();
        let rewards = RewardBreakdown {
            own_reward: OwnRewardLine {
                own_business_volume: 0,
                own_slab_pct: 14,
                amount: 0,
            },
            differentials,
            royalty: Some(RoyaltyLine {
                qualifying_children: 80,
                rate_percent: 5.0,
                amount: 800,
            }),
            rewards_total: 1_600,
        };
        let bytes = render_bytes(mid_section(&rewards, &member, 80));
        let doc = lopdf::Document::load_mem(&bytes).expect("lopdf must parse genpdf's output");
        assert!(
            doc.get_pages().len() > 1,
            "80 legs must overflow a single page"
        );
    }

    /// Regression for the bug found 15 Aug 2026 in real use: rendering
    /// silently dropped some cells' Amount text (no panic, no error — the
    /// figures were just missing on the page) when the rewards-detail
    /// table was nested inside another `TableLayout`'s row alongside the
    /// Details column. Root cause: genpdf 0.2.0's `TableLayout`-in-
    /// `TableLayout` nesting is unreliable — bisection produced different
    /// failure patterns across near-identical structures, so no safe
    /// structural workaround exists within `TableLayout` nesting itself.
    /// Fixed by `TwoColumn` (see its doc comment above `mid_section`),
    /// which never nests a `TableLayout` inside another one.
    ///
    /// This test can only assert structurally (renders, one page,
    /// plausible byte size) — `pdf-extract` crashes on this document's
    /// text (module doc comment) and `lopdf::Document::extract_text`
    /// decodes the embedded font's subset encoding into unreadable
    /// mojibake, not assertable text. The real regression check for this
    /// bug is visual: render a sample through `render_member_detail_pdf`
    /// and read the actual PDF file.
    #[test]
    fn mid_section_with_many_nonuniform_amounts_renders_one_page() {
        let member = sample_member(true, Some(896044));
        let differentials = vec![
            differential_line(1, "Mohit Shah", 2_147_185, 14, 14, 0),
            differential_line(2, "Vivek Rao 2", 2_958_990, 14, 14, 0),
            differential_line(3, "Diya Patel 7", 118_847, 4, 14, 11_885),
            differential_line(4, "Pooja Menon", 2_153_632, 14, 14, 0),
            differential_line(5, "Suresh Naidu", 953_841, 12, 14, 19_077),
            differential_line(6, "Kavya Reddy 3", 613_088, 10, 14, 24_524),
            differential_line(7, "Neha Joshi", 467_811, 8, 14, 28_069),
            differential_line(8, "Ishita Iyer 2", 172_245, 6, 14, 13_780),
        ];
        let rewards = RewardBreakdown {
            own_reward: OwnRewardLine {
                own_business_volume: 0,
                own_slab_pct: 14,
                amount: 0,
            },
            differentials,
            royalty: Some(RoyaltyLine {
                qualifying_children: 3,
                rate_percent: 5.0,
                amount: 72_598,
            }),
            rewards_total: 169_933,
        };
        let bytes = render_bytes(mid_section(&rewards, &member, 8));
        assert!(bytes.starts_with(b"%PDF"));
        let doc = lopdf::Document::load_mem(&bytes).expect("lopdf must parse genpdf's output");
        assert_eq!(
            doc.get_pages().len(),
            1,
            "8 legs must fit one page — page count alone doesn't prove no cells were dropped"
        );
    }

    #[test]
    fn render_member_detail_pdf_writes_a_valid_pdf_with_direct_children() {
        let member = sample_member(true, None);
        let differentials = vec![differential_line(1, "Mohit Shah", 2_147_185, 14, 14, 0)];
        let children = vec![MemberDetailChild {
            member_id: 1,
            name: "Mohit Shah".into(),
            total_business_volume: 2_147_185,
            slab_pct: 14,
            is_active: true,
        }];
        let detail = MemberDetail {
            member,
            total_business_volume: 2_147_185,
            slab_pct: 14,
            leg_count: 1,
            rewards: RewardBreakdown {
                own_reward: OwnRewardLine {
                    own_business_volume: 0,
                    own_slab_pct: 14,
                    amount: 0,
                },
                differentials,
                royalty: None,
                rewards_total: 0,
            },
            direct_children: children,
        };
        let path = std::env::temp_dir().join("member-detail-pdf-render-test.pdf");
        render_member_detail_pdf(
            &detail,
            "2026-07",
            "15 Aug 2026 14:32",
            path.to_str().unwrap(),
        )
        .expect("rendering a normal member detail must succeed");
        let bytes = std::fs::read(&path).expect("render_to_file must write the file");
        assert!(bytes.starts_with(b"%PDF"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn render_member_detail_pdf_writes_a_valid_pdf_with_no_direct_children() {
        let member = sample_member(false, Some(42));
        let detail = MemberDetail {
            member,
            total_business_volume: 0,
            slab_pct: 0,
            leg_count: 0,
            rewards: RewardBreakdown {
                own_reward: OwnRewardLine {
                    own_business_volume: 0,
                    own_slab_pct: 0,
                    amount: 0,
                },
                differentials: vec![],
                royalty: None,
                rewards_total: 0,
            },
            direct_children: vec![],
        };
        let path = std::env::temp_dir().join("member-detail-pdf-render-empty-test.pdf");
        render_member_detail_pdf(
            &detail,
            "2026-07",
            "15 Aug 2026 14:32",
            path.to_str().unwrap(),
        )
        .expect("rendering a member with no direct legs must succeed");
        let bytes = std::fs::read(&path).expect("render_to_file must write the file");
        assert!(bytes.starts_with(b"%PDF"));
        std::fs::remove_file(&path).ok();
    }
}
