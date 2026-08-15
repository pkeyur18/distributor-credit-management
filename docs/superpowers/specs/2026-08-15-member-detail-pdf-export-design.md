# Member Detail PDF Export — Design

**Date:** 2026-08-15
**Status:** Approved for planning
**Requirement refs:** CR-6, US-M4.5, M4.8, ADR-013, API-46, AC-48 (see `documents/refinement/`)

## 1. Problem

The administrator can currently only show a member their figures on screen, or hand
them a spreadsheet extract covering every member (M6's `.xlsx` reports). There is no
single, self-contained document for one member's own record — their identity, their
Business Volume figures, the reward calculation behind them, and the direct legs whose
figures feed that calculation. Product objective 4 (`01-product-and-scope.md` §1.2)
names this directly: *"When a member questions their figure, I can show them exactly
which people below them contributed what."* This feature is that document.

## 2. Scope

- One member, one period (whatever period the member-detail screen is currently
  viewing) per PDF.
- Direct legs only — the same one level already shown in the screen's "Direct legs"
  table. Not the full downline. Consistent with Rule-45/FR-2: deeper tree views never
  show more per node than name, ID and own Business Volume, and this document doesn't
  attempt to be a hierarchy view at all.
- Reuses `get_member_detail` (`src-tauri/src/m4_search/mod.rs:233`) unchanged — same
  query, same period resolution, same reward breakdown. No new calculation logic, no
  new business rule.
- Every abbreviation is spelled out in full on the document ("Business Volume", "Total
  Business Volume" — never "BV"/"TBV") since it may be handed directly to a member
  unfamiliar with the shorthand used internally.
- No currency, no company branding — consistent with BO-6 and the product's "figures
  are unitless" rule (`01-product-and-scope.md` §1).

## 3. Architecture

Follows ADR-007's established boundary for `.xlsx` exports, extended to a second file
type (recorded as ADR-013): **all PDF generation happens Rust-side.** The WebView never
receives raw file bytes — it only supplies a destination path chosen through the same
native save dialog (`tauri-plugin-dialog`) the `.xlsx` exports and backup/restore flows
already use.

```
member-detail.tsx                 m4_search (Rust)                  genpdf
  [Export PDF] click
    -> saveFileDialog()  ---------------------------------------------->  (native dialog)
    <- outputPath (or null, user cancelled)
    -> exportMemberDetailPdf(memberId, periodMonth, outputPath)
                                   -> get_member_detail(conn, member_id, period_month)
                                        (existing function, unchanged)
                                   -> render_member_detail_pdf(&detail, &member_context)
                                                                        -> genpdf::Document
                                                                        -> .render_to_file(outputPath)
                                   <- ExportResult { file_path }
    <- toast "Member record exported"
```

New code is additive only:
- `src-tauri/src/m4_search/pdf.rs` — new module: builds the `genpdf::Document` from a
  `MemberDetail` + the export timestamp, writes it to the given path.
- `src-tauri/src/m4_search/mod.rs` — new command `export_member_detail_pdf`, thin
  wrapper: resolve detail via the existing function, hand it to `pdf::render`.
- `src/lib/ipc/m4-search.ts` — new `exportMemberDetailPdf()` binding, same shape as
  `m6-reports.ts`'s existing export bindings.
- `src/screens/member-detail.tsx` — new "Export PDF" button next to "Record volume"
  (line ~143), wired to `saveFileDialog` + the new IPC call + a toast, mirroring
  `reports.tsx`'s `handleExportMonthly` pattern exactly.

**Placement:** `m4_search`, not `m6_reports`. `m6_reports` is "M6 — Reports & Exports":
bulk extracts across every member. This is a single member's own detail, generated
from the same screen and the same query that already serves it — it belongs with
`get_member_detail`, not the bulk-export module.

**Dependency:** `genpdf = "0.2"` added to `src-tauri/Cargo.toml` (built on `printpdf`,
already MIT-licensed, no network fetch — consistent with the offline constraint,
NFR-14).

**Fonts (found during implementation planning, not in the original design):** genpdf
has no built-in font — it requires real static-weight TTF files embedded at build time,
via a `genpdf::fonts::FontFamily` built from raw bytes. The app's existing Inter asset
(`@fontsource-variable/inter`) is a variable-weight woff2, which neither `genpdf` nor
the `printpdf`/`ttf-parser` stack underneath it can consume directly. Static Inter
Regular + SemiBold TTFs (OFL-1.1, from Inter's v3.19 release — the last one distributed
as static per-weight files) are added at `src-tauri/assets/fonts/` and embedded via
`include_bytes!`, kept as two files rather than fetched over the network at build or
run time. SemiBold stands in for the screen's weight-650 emphasis (`07-design-system.md`
§2) — the nearest static weight to it.

### Why genpdf over printpdf

`printpdf` gives exact coordinate control but requires hand-rolled pagination — the
direct-legs table has no upper bound on row count. `genpdf` is a layout engine on top
of `printpdf` that paginates flowing content (paragraphs, tables) automatically.

**Open risk, to resolve in implementation (not here):** the chosen layout (Option B —
two-column mid-section: rewards detail | member details, matching the screen's own
`lg:grid-cols-[1.4fr_1fr]`) needs a `genpdf::TableLayout` row whose cells each contain
a further nested element. Whether genpdf correctly paginates a *nested* table when the
outer row would otherwise overflow a page is unverified. First implementation step
should be a small spike: render the two-column section with a direct-legs count large
enough to overflow a page, and confirm it either paginates cleanly or degrades
predictably. Fallback if it doesn't: drop to a single column for any content that
would need to break mid-page (matches Option A's structure, which was already
designed and rejected only on layout preference, not technical grounds) — or hand-roll
that one section's pagination with `printpdf` primitives while keeping the rest of the
document in `genpdf`.

## 4. Visual design

Mirrors the screen's own two-column grid, built from the existing Single Ledger design
tokens (`07-design-system.md`): indigo accent, tabular numerals for every figure,
8px-radius flat cards, 1px hairline borders, no shadows. Status ("Active"/"Inactive")
and slab keep the colour-plus-label rule — colour is an accent on the text label, never
the only signal, matching `07-design-system.md`'s Colour-Plus-Label Rule. Pills render
as simple square-cornered tinted backgrounds in the PDF (not the fully-rounded CSS
pill) — `genpdf`/`printpdf` rounded-rect drawing is disproportionate effort for a
detail that doesn't change what the document communicates.

Page contents, top to bottom:
1. Header — member name, active/inactive pill, member number, phone, joining date,
   period, generation timestamp.
2. Four stat boxes — Business Volume, Total Business Volume, Slab, Rewards this
   period (same four the screen shows).
3. Two-column section — Rewards detail table (own reward, each leg's differential
   line, royalty, total) beside member Details (address, email, introducer, direct-leg
   count, consent date).
4. Direct legs table, full width — name, member number, Total Business Volume, slab,
   status. Same columns as the screen's own "Direct legs" table.
5. Footer — generation note, page number.

Approved mockup (Option B, HTML preview built from the same design tokens):
`https://claude.ai/code/artifact/1803a008-8c6c-4c0d-9744-02b41ab2f324`

## 5. Error handling

No new failure modes. Member-not-found and period-resolution errors already exist in
`get_member_detail` and already surface to the screen today. A write failure at the
chosen output path follows the same `AppError::Export` path the `.xlsx` exports use
(`src-tauri/src/error.rs`).

## 6. Testing

- **Rust:** a test asserting the rendered PDF's extracted text/values match a
  `MemberDetail` fixture — golden-scenario style, consistent with this project's
  existing calculation tests. Covers: zero direct legs (no royalty line), many direct
  legs (pagination), inactive member (status pill still renders, Rule-28 — inactive
  has zero computational effect but must still display correctly).
- **Frontend:** component test for the button → save-dialog → IPC call → toast flow,
  mirroring whatever test pattern `reports.tsx`'s exports already use, including the
  cancel-the-dialog path (no call made, no error toast).

## 7. Out of scope

- Full downline / multi-level export (only direct legs, per approved scope).
- Bulk PDF export of every member (that's a `m6_reports`-shaped feature, not this one).
- Any period other than the one currently on screen — no period picker in the export
  flow itself.
- Letterhead, logo, or any company branding (BO-6, `01-product-and-scope.md` line 199).
