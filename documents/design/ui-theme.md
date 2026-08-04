# UI Theme — Design System Reference

| | |
|---|---|
| **Status** | Approved — applies globally to every screen |
| **Companion** | [architecture.md](architecture.md) — locks the stack this theme is implemented on (React + TS + shadcn/ui + Tailwind) |
| **Direction** | Clean enterprise (Linear/Notion/Vercel-style) — neutral, dense, one restrained accent |
| **Mode** | Light default, dark mode available via the same tokens |

This is the single source of truth for visual identity. Every screen — prototype or real build — uses these
tokens, not ad-hoc values. When the real Tailwind config is written, it is generated from this table, not
re-derived.

## Tokens

| Token | Light | Dark | Used for |
|---|---|---|---|
| `--accent` | `#4f46e5` | `#6366f1` | Primary buttons, links, active nav item, focus ring. **One accent only** — no secondary brand color, no gradient |
| `--bg` | `#f8fafc` | `#0f172a` | Page background |
| `--surface` | `#ffffff` | `#1e293b` | Cards, table rows, inputs, modals |
| `--border` | `#e2e8f0` | `#334155` | 1px hairlines. Flat design — shadows reserved for modals/popovers only, never for cards or rows |
| `--text` | `#0f172a` | `#f1f5f9` | Primary text |
| `--text-muted` | `#64748b` | `#94a3b8` | Labels, secondary text, table headers |
| `--success` | `#059669` | `#10b981` | Status only — never decorative |
| `--warning` | `#d97706` | `#f59e0b` | The outstanding-month banner (Rule 20). Amber, not red — it names a required action, not an error |
| `--danger` | `#dc2626` | `#ef4444` | Validation errors, locked/refused states |

## Type, shape, density

- **Font**: system-ui stack in prototypes (CSP-safe, no CDN). The real Tauri build bundles **Inter locally**
  — no web fonts, ever, consistent with the offline constraint (architecture.md §11.14).
- **Numbers**: `font-variant-numeric: tabular-nums`, right-aligned in every table. This system is read as a
  ledger all day — columns must line up.
- **Radius**: `6px` on buttons/inputs/table cells, `8px` on modals/panels. Restrained, not the rounded-SaaS
  look.
- **Density**: ~40px table row height, 4px base spacing unit, 24px page padding. One power user, daily use —
  more rows visible beats whitespace.

## Rules that aren't optional

- **Status is never color-only** (§11.8 accessibility). An inactive member gets a labelled gray pill
  ("Inactive"), not a colored dot alone. This directly implements M4.5/M6.5's "distinct colour" requirement
  without relying on color perception alone.
- **The outstanding-month banner has no dismiss control** — not even an icon that looks like one. Rule 20
  requires it to clear only on a completed close.
- **Vocabulary constraint (§1.2) applies to every string**, including placeholder/mock data: only *member,
  Business Volume, Rewards, royalty, volume, slab, level, leg*. No *sale/purchase/order/cash/payment/
  commission/invoice* anywhere, in any screen, ever.

## Reference CSS block

Every prototype embeds this exact block so consistency is checkable, not asserted:

```css
:root {
  --accent: #4f46e5;
  --bg: #f8fafc;
  --surface: #ffffff;
  --border: #e2e8f0;
  --text: #0f172a;
  --text-muted: #64748b;
  --success: #059669;
  --warning: #d97706;
  --danger: #dc2626;
  --radius-sm: 6px;
  --radius-lg: 8px;
}
@media (prefers-color-scheme: dark) {
  :root {
    --accent: #6366f1;
    --bg: #0f172a;
    --surface: #1e293b;
    --border: #334155;
    --text: #f1f5f9;
    --text-muted: #94a3b8;
    --success: #10b981;
    --warning: #f59e0b;
    --danger: #ef4444;
  }
}
```
