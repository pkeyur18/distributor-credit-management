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
