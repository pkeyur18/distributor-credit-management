// Rule-47 (CR-7): draft membership-level names, rank 1..4. Display only —
// the backend stores and sends the rank, and names are not a setting.
// Must match MEMBERSHIP_LEVEL_NAMES in src-tauri/src/m3_calc/engine.rs.
export const MEMBERSHIP_LEVEL_NAMES = ["Gold", "Platinum", "Diamond", "Ace"] as const;

export function membershipLevelName(rank: number): string {
  return MEMBERSHIP_LEVEL_NAMES[rank - 1] ?? "—";
}
