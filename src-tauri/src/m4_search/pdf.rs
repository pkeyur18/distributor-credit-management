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

use genpdf::elements::{Break, LinearLayout, PaddedElement, Paragraph, TableLayout};
use genpdf::fonts::{FontData, FontFamily};
use genpdf::style::{Color, Style};
use genpdf::{Alignment, Document, Element, Margins, SimplePageDecorator, Size};

use crate::error::AppError;
use crate::m1_members::Member;
use crate::m4_search::{MemberDetail, MemberDetailChild, RewardBreakdown};

// 07-design-system.md §1: accent #4f46e5, ledger green #059669, red #dc2626.
const ACCENT: Color = Color::Rgb(0x4f, 0x46, 0xe5);
const SUCCESS: Color = Color::Rgb(0x05, 0x96, 0x69);
const DANGER: Color = Color::Rgb(0xdc, 0x26, 0x26);
const MUTED: Color = Color::Rgb(0x64, 0x74, 0x8b);
const INK: Color = Color::Rgb(0x0f, 0x17, 0x2a);

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
    if is_active { "Active" } else { "Inactive" }
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
pub(super) fn header_section(member: &Member, period_label: &str, generated_at: &str) -> LinearLayout {
    let mut layout = LinearLayout::vertical();

    let mut name_line = Paragraph::new("");
    name_line.push_styled(&member.name, Style::new().bold().with_font_size(15).with_color(INK));
    name_line.push_styled("   ", Style::new());
    let status_color = if member.is_active { SUCCESS } else { DANGER };
    name_line.push_styled(status_label(member.is_active), Style::new().bold().with_color(status_color));
    layout.push(name_line);

    let mut meta_line = Paragraph::new("");
    meta_line.push_styled(meta_line_text(member), Style::new().with_color(MUTED).with_font_size(10));
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
    if negative { format!("-{grouped}") } else { grouped }
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
        ("Total Business Volume", format_amount(total_business_volume)),
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
    ]
}

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
/// `grid-cols-4` stat strip.
pub(super) fn stat_boxes(
    business_volume: i64,
    total_business_volume: i64,
    slab_pct: i64,
    rewards_total: i64,
) -> TableLayout {
    let values = stat_box_values(business_volume, total_business_volume, slab_pct, rewards_total);
    let mut table = TableLayout::new(vec![1, 1, 1, 1]);
    let mut row = table.row();
    for (label, value) in values {
        row = row.element(stat_box(label, value));
    }
    row.push().expect("a fixed 4-cell row always has the right cell count");
    table
}

/// Every table cell in this file goes through this — genpdf's `TableLayout`
/// columns sit flush against each other with no gutter, which (found
/// during the Task 8 visual check) makes right-aligned neighbouring
/// columns run together, e.g. "3,20010%" with no gap between the amount
/// and the next column. A little right-padding and row spacing fixes it.
fn cell(p: Paragraph) -> PaddedElement<Paragraph> {
    p.padded(Margins::trbl(0, 4, 3, 0))
}

fn rewards_detail_table(rewards: &RewardBreakdown) -> TableLayout {
    let mut table = TableLayout::new(vec![6, 3, 2]);

    let mut leg_header = Paragraph::new("");
    leg_header.push_styled("Leg", Style::new().bold().with_font_size(9).with_color(MUTED));
    let mut bv_header = Paragraph::new("");
    bv_header.push_styled("Business Volume", Style::new().bold().with_font_size(9).with_color(MUTED));
    bv_header.set_alignment(Alignment::Right);
    let mut amt_header = Paragraph::new("");
    amt_header.push_styled("Amount", Style::new().bold().with_font_size(9).with_color(MUTED));
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
        let desc_style = if row.emphasized { Style::new().bold() } else { Style::new() };
        desc.push_styled(row.description, desc_style);

        let mut bv = Paragraph::new("");
        if let Some(bv_text) = row.business_volume {
            bv.push_styled(bv_text, Style::new());
        }
        bv.set_alignment(Alignment::Right);

        let mut amt = Paragraph::new("");
        amt.push_styled(row.amount, Style::new().bold());
        amt.set_alignment(Alignment::Right);

        table.row().element(cell(desc)).element(cell(bv)).element(cell(amt)).push().unwrap();
    }

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

    let headers = ["Name", "Member #", "Total Business Volume", "Slab", "Status"];
    let mut row = table.row();
    for (i, h) in headers.iter().enumerate() {
        let mut p = Paragraph::new("");
        p.push_styled(*h, Style::new().bold().with_font_size(9).with_color(MUTED));
        if i == 2 {
            p.set_alignment(Alignment::Right);
        }
        row = row.element(cell(p));
    }
    row.push().expect("a fixed 5-cell header row always has the right cell count");

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
        let status_color = if leg.status == "Active" { SUCCESS } else { DANGER };
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m1_members::Member;
    use crate::m4_search::{DifferentialLine, OwnRewardLine, RoyaltyLine};
    use genpdf::{Document, Size};

    #[test]
    fn the_bundled_font_family_loads_without_panicking() {
        let _family = load_font_family();
    }

    pub(super) fn sample_member(is_active: bool, introducer_member_id: Option<i64>) -> Member {
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

    /// Renders a single element into a one-page PDF — a structural smoke
    /// test only (non-empty, starts with the `%PDF` magic bytes). Content
    /// correctness is asserted on the pure text-composition functions
    /// instead (see the module doc comment).
    pub(super) fn render_bytes(element: impl genpdf::Element + 'static) -> Vec<u8> {
        let mut doc = Document::new(load_font_family());
        doc.set_paper_size(Size::new(210, 297)); // A4
        doc.push(element);
        let mut bytes = Vec::new();
        doc.render(&mut bytes).expect("render must succeed");
        bytes
    }

    #[test]
    fn status_label_is_spelled_out_in_full_never_bv_style_shorthand() {
        assert_eq!(status_label(true), "Active");
        assert_eq!(status_label(false), "Inactive");
    }

    #[test]
    fn meta_line_text_includes_member_number_phone_and_joining_date() {
        let member = sample_member(true, Some(100001));
        let text = meta_line_text(&member);
        assert!(text.contains("100042"));
        assert!(text.contains("+91 98765 43210"));
        assert!(text.contains("2024-03-12"));
    }

    #[test]
    fn period_line_text_names_the_period_and_generation_time() {
        let text = period_line_text("July 2026", "15 Aug 2026 14:32");
        assert!(text.contains("July 2026"));
        assert!(text.contains("15 Aug 2026 14:32"));
        assert!(!text.contains("BV"), "every figure must be spelled out in full (CR-6)");
    }

    #[test]
    fn header_section_renders_without_panicking() {
        let member = sample_member(true, Some(100001));
        let section = header_section(&member, "July 2026", "15 Aug 2026 14:32");
        let bytes = render_bytes(section);
        assert!(!bytes.is_empty());
        assert!(bytes.starts_with(b"%PDF"));
    }

    #[test]
    fn header_section_renders_for_an_inactive_member_without_panicking() {
        let member = sample_member(false, Some(100001));
        let section = header_section(&member, "July 2026", "15 Aug 2026 14:32");
        let bytes = render_bytes(section);
        assert!(bytes.starts_with(b"%PDF"));
    }

    #[test]
    fn format_amount_groups_thousands_and_keeps_the_sign() {
        assert_eq!(format_amount(0), "0");
        assert_eq!(format_amount(1_200), "1,200");
        assert_eq!(format_amount(8_450), "8,450");
        assert_eq!(format_amount(1_000_000), "1,000,000");
        assert_eq!(format_amount(-612), "-612");
    }

    #[test]
    fn stat_box_values_spells_out_every_label_in_full() {
        let values = stat_box_values(1_200, 8_450, 12, 612);
        assert_eq!(values[0], ("Business Volume", "1,200".to_string()));
        assert_eq!(values[1], ("Total Business Volume", "8,450".to_string()));
        assert_eq!(values[2], ("Slab", "12%".to_string()));
        assert_eq!(values[3], ("Rewards this period", "612".to_string()));
    }

    fn differential_line(
        id: i64,
        name: &str,
        tbv: i64,
        slab: i64,
        own_slab: i64,
        amount: i64,
    ) -> DifferentialLine {
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
    fn rewards_detail_rows_orders_own_reward_first_then_legs_then_royalty_then_total() {
        let rewards = RewardBreakdown {
            own_reward: OwnRewardLine { own_business_volume: 1_200, own_slab_pct: 12, amount: 144 },
            differentials: vec![
                differential_line(100078, "Aarav Shah", 3_200, 10, 12, 64),
                differential_line(100091, "Priya Mehta", 2_900, 9, 12, 87),
            ],
            royalty: Some(RoyaltyLine { qualifying_children: 2, rate_percent: 5, amount: 190 }),
            rewards_total: 612,
        };
        let rows = rewards_detail_rows(&rewards);
        assert_eq!(rows.len(), 5); // own + 2 legs + royalty + total
        assert!(rows[0].description.contains("Own Business Volume"));
        assert_eq!(rows[1].description, "Aarav Shah");
        assert_eq!(rows[2].description, "Priya Mehta");
        assert!(rows[3].description.contains("Royalty"));
        assert_eq!(rows[4].description, "Rewards total");
        assert_eq!(rows[4].amount, "612");
    }

    #[test]
    fn rewards_detail_rows_with_no_direct_legs_has_no_royalty_row() {
        let rewards = RewardBreakdown {
            own_reward: OwnRewardLine { own_business_volume: 1_200, own_slab_pct: 12, amount: 144 },
            differentials: vec![],
            royalty: None,
            rewards_total: 144,
        };
        let rows = rewards_detail_rows(&rewards);
        assert_eq!(rows.len(), 2); // own + total, no royalty
        assert!(!rows.iter().any(|r| r.description.contains("Royalty")));
    }

    #[test]
    fn details_rows_shows_introducer_link_and_direct_leg_count() {
        let member = sample_member(true, Some(100001));
        let rows = details_rows(&member, 4);
        let by_label: std::collections::HashMap<&str, &str> =
            rows.iter().map(|(l, v)| (*l, v.as_str())).collect();
        assert_eq!(by_label["Address"], "12 MG Road, Ahmedabad");
        assert_eq!(by_label["Introduced by"], "#100001");
        assert_eq!(by_label["Direct legs"], "4");
    }

    #[test]
    fn details_rows_for_the_root_member_names_them_as_root() {
        let member = sample_member(true, None);
        let rows = details_rows(&member, 0);
        let by_label: std::collections::HashMap<&str, &str> =
            rows.iter().map(|(l, v)| (*l, v.as_str())).collect();
        assert!(by_label["Introduced by"].contains("root member"));
    }

    #[test]
    fn stat_boxes_renders_without_panicking() {
        let bytes = render_bytes(stat_boxes(1_200, 8_450, 12, 612));
        assert!(bytes.starts_with(b"%PDF"));
    }

    #[test]
    fn mid_section_renders_without_panicking() {
        let member = sample_member(true, Some(100001));
        let rewards = RewardBreakdown {
            own_reward: OwnRewardLine { own_business_volume: 1_200, own_slab_pct: 12, amount: 144 },
            differentials: vec![differential_line(100078, "Aarav Shah", 3_200, 10, 12, 64)],
            royalty: Some(RoyaltyLine { qualifying_children: 1, rate_percent: 5, amount: 190 }),
            rewards_total: 612,
        };
        let bytes = render_bytes(mid_section(&rewards, &member, 4));
        assert!(bytes.starts_with(b"%PDF"));
    }

    /// The pagination spike (design spec §3, "Why genpdf over printpdf"): a
    /// nested `TableLayout` inside a `TableLayoutRow` cell, with enough
    /// direct legs that the nested rewards-detail table alone would
    /// overflow one page. Verified via `lopdf`'s page count, not
    /// `pdf-extract` (see the module doc comment) — if genpdf silently
    /// dropped rows instead of paginating, the page count would still come
    /// back as 1, since dropping rows doesn't create a page-count signal
    /// either way. This test's real job is narrower than the original
    /// design intended: it confirms genpdf renders more than one page
    /// rather than panicking or clipping visibly. Whether every one of 80
    /// rows survived can only be confirmed by the Task 8 manual visual
    /// check now that content-level PDF assertions aren't available.
    #[test]
    fn mid_section_with_many_direct_legs_renders_more_than_one_page() {
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
        let bytes = render_bytes(mid_section(&rewards, &member, 80));
        assert!(bytes.starts_with(b"%PDF"));

        let doc = lopdf::Document::load_mem(&bytes).expect("lopdf must parse genpdf's output");
        let page_count = doc.get_pages().len();
        assert!(
            page_count > 1,
            "80 direct legs must overflow a single A4 page (got {page_count} page(s)) \u{2014} \
             if this is 1, genpdf silently clipped instead of paginating; see the design spec's \
             documented fallback (single-column, or hand-roll this table with printpdf)"
        );
    }

    fn sample_child(member_id: i64, name: &str, tbv: i64, slab: i64, is_active: bool) -> MemberDetailChild {
        MemberDetailChild {
            member_id,
            name: name.into(),
            total_business_volume: tbv,
            slab_pct: slab,
            is_active,
        }
    }

    #[test]
    fn direct_leg_rows_formats_every_field_and_status_as_text() {
        let children = vec![
            sample_child(100078, "Aarav Shah", 3_200, 10, true),
            sample_child(100117, "Kunal Verma", 1_300, 6, false),
        ];
        let rows = direct_leg_rows(&children);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].name, "Aarav Shah");
        assert_eq!(rows[0].member_id, "100078");
        assert_eq!(rows[0].total_business_volume, "3,200");
        assert_eq!(rows[0].slab_pct, "10%");
        assert_eq!(rows[0].status, "Active");
        assert_eq!(rows[1].status, "Inactive", "Rule-28: inactive still displays, colour-plus-label");
    }

    #[test]
    fn direct_leg_rows_with_no_children_is_empty() {
        assert!(direct_leg_rows(&[]).is_empty());
    }

    #[test]
    fn direct_legs_table_renders_without_panicking() {
        let children = vec![sample_child(100078, "Aarav Shah", 3_200, 10, true)];
        let bytes = render_bytes(direct_legs_table(&children));
        assert!(bytes.starts_with(b"%PDF"));
    }

    #[test]
    fn direct_legs_table_with_no_children_still_renders_a_header_only() {
        let bytes = render_bytes(direct_legs_table(&[]));
        assert!(bytes.starts_with(b"%PDF"));
    }

    fn sample_detail(direct_children: Vec<MemberDetailChild>) -> MemberDetail {
        let differentials: Vec<DifferentialLine> = direct_children
            .iter()
            .map(|c| {
                differential_line(c.member_id, &c.name, c.total_business_volume, c.slab_pct, 12, 10)
            })
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
    fn render_member_detail_pdf_writes_a_file_starting_with_the_pdf_magic_bytes() {
        let dir = std::env::temp_dir().join(format!("bvconsole-pdf-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let output_path = dir.join("member-detail.pdf");

        let children = vec![sample_child(100078, "Aarav Shah", 3_200, 10, true)];
        let detail = sample_detail(children);

        render_member_detail_pdf(&detail, "July 2026", "15 Aug 2026 14:32", output_path.to_str().unwrap())
            .expect("render must succeed");

        assert!(output_path.exists());
        let bytes = std::fs::read(&output_path).unwrap();
        assert!(bytes.starts_with(b"%PDF"));
        assert!(bytes.len() > 100, "a rendered document should be more than a trivial stub");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn render_member_detail_pdf_with_zero_direct_legs_still_renders() {
        let dir = std::env::temp_dir().join(format!("bvconsole-pdf-test-zero-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let output_path = dir.join("member-detail.pdf");

        let detail = sample_detail(vec![]);
        render_member_detail_pdf(&detail, "July 2026", "15 Aug 2026 14:32", output_path.to_str().unwrap())
            .expect("render must succeed");

        let bytes = std::fs::read(&output_path).unwrap();
        assert!(bytes.starts_with(b"%PDF"));

        std::fs::remove_dir_all(&dir).ok();
    }
}
