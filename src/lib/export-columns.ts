// T-M7.2-4/D-1: the export column set every extract-producing screen
// shares — Settings' "default export columns" picker and Reports' own
// per-run picker (US-M6.1) must name columns identically, so both read
// from here rather than each keeping its own list that could drift.
// Keys match src-tauri/src/m6_reports/mod.rs's `OptionalColumn::parse`
// exactly.
export const MANDATORY_EXPORT_COLUMNS: { key: string; label: string }[] = [
  { key: "name", label: "Name" },
  { key: "member_number", label: "Member number" },
  { key: "phone", label: "Phone" },
  { key: "business_volume", label: "Business Volume" },
  { key: "total_business_volume", label: "Total Business Volume" },
];

// Rule-33's optional list, minus Total Business Volume — D-1 already moved
// that into the mandatory five above.
export const OPTIONAL_EXPORT_COLUMNS: { key: string; label: string }[] = [
  { key: "email", label: "Email" },
  { key: "address", label: "Address" },
  { key: "reference_number", label: "Reference number" },
  { key: "introducer_name", label: "Introducer name" },
  { key: "hierarchy_level", label: "Hierarchy level" },
  { key: "direct_legs_count", label: "Direct legs count" },
  { key: "slab_pct", label: "Slab %" },
  { key: "rewards", label: "Rewards" },
  { key: "royalty_earned", label: "Royalty earned" },
  { key: "joining_date", label: "Joining date" },
  { key: "active_status", label: "Active/inactive status" },
];
