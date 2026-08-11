// Mirrors the corrected DDL in src-tauri/src/db/migrations/0001_initial.sql.
// All ×100 fixed-point figures (ADR-004) stay integers here too — the ×100
// convention is undone only at the UI boundary, not in these types.

export interface Member {
  id: number;
  name: string;
  phone: string;
  email: string | null;
  address: string;
  introducerMemberId: number | null;
  level: number;
  isActive: boolean;
  joiningDate: string;
  consentGiven: boolean;
  consentDate: string;
  createdAt: string;
}

export interface BusinessVolumeEntry {
  id: number;
  memberId: number;
  amount: number;
  entryDate: string;
  periodMonth: string;
  createdAt: string;
  updatedAt: string | null;
}

export interface SlabRow {
  id: number;
  threshold: number;
  percentage: number;
  sortOrder: number;
}

// D-11: sortOrder is display-only — never used to decide which row matches.
export type PeriodStatus = "open" | "awaiting_close" | "closed";

export interface Period {
  id: number;
  periodMonth: string;
  status: PeriodStatus;
  endedAt: string | null;
  closedAt: string | null;
}

export interface MemberPeriodFigures {
  memberId: number;
  periodId: number;
  businessVolume: number;
  totalBusinessVolume: number;
  slabPct: number;
  differential: number;
  royalty: number;
  ownReward: number;
  rewards: number;
}

export interface MonthlySnapshot extends MemberPeriodFigures {
  id: number;
  version: number;
  isActiveStatus: boolean;
  createdAt: string;
}

export type BackupKind = "period_close" | "scheduled" | "manual" | "pre_restore_safety";
export type BackupScheduleKind = "daily" | "weekly" | "monthly";

export interface BackupRecord {
  id: number;
  periodId: number | null;
  kind: BackupKind;
  scheduleKind: BackupScheduleKind | null;
  version: number;
  checksum: string;
  isOriginal: boolean;
  createdAt: string;
}

// D-12/D-13 corrected value sets.
export type AuditEntityType = "member" | "entry" | "setting" | "period" | "backup" | "auth";
export type AuditCause =
  | "entry"
  | "edit"
  | "correction"
  | "settings_change"
  | "period_close"
  | "manual_backup"
  | "console_backup"
  | "restore";

export interface AuditLogEntry {
  id: number;
  entityType: AuditEntityType;
  entityId: number;
  field: string;
  oldValue: string | null;
  newValue: string | null;
  changedAt: string;
  cause: AuditCause;
}

// API-11's node shape. The Structure/Full-Hierarchy tree node still shows
// exactly three fields (name/ID/own BV), never Total Business Volume — that
// display rule lives in StructureTreeNode, not here. slabPct/rewards ride
// along on every node because Home's slab-distribution charts (US-M4.4)
// reuse this same command (full_tree: true, rooted at ROOT_ID) for their
// aggregation — no dedicated IPC command exists for either chart.
export interface ChartNode {
  memberId: number;
  name: string;
  ownBusinessVolume: number;
  isActive: boolean;
  introducerMemberId: number | null;
  slabPct: number;
  rewards: number;
  /** Direct-child count — lets the Structure screen show the leaf/expand
   *  affordance before that node's own children have been fetched. */
  legCount: number;
}

export interface SearchResult {
  id: number;
  name: string;
  phone: string;
  totalBusinessVolume: number;
  slabPct: number;
  isActive: boolean;
  // Not displayed by SearchResultsList (T-M1.4-5's field list is unchanged)
  // — carried so an Edit modal can open straight from a search result
  // without get_member_detail (M4.1, S8) existing yet.
  email: string | null;
  address: string;
  introducerMemberId: number | null;
}

export interface Settings {
  slabThresholds: number[];
  slabPercentages: number[];
  referenceUnitValue: number;
  hierarchyDepth: number;
  level2Width: number;
  level3Width: number;
  level4Width: number;
  royaltyQualifyingCount: number;
  royaltyRatePercent: number;
  yearlyCycle: { start: string; end: string };
  lowContributionThreshold: number;
  defaultExportColumns: string[];
  sessionTimeoutMinutes: number;
  consoleBackupSchedule: "off" | BackupScheduleKind;
  consoleBackupRetentionCount: number;
  consoleBackupFolder: string;
}
