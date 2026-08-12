import { invokeCommand } from "./client";

export interface CandidateSettings {
  slabThresholds?: number[];
  slabPercentages?: number[];
  royaltyQualifyingCount?: number;
  royaltyRatePercent?: number;
}

export interface MemberImpact {
  memberId: number;
  memberName: string;
  rewardsBefore: number;
  rewardsAfter: number;
  slabPctBefore: number;
  slabPctAfter: number;
  royaltyBefore: number;
  royaltyAfter: number;
}

export interface SettingsImpactPreview {
  rewardsBefore: number;
  rewardsAfter: number;
  affectedMembers: MemberImpact[];
}

// API-33 — the only M3 command. Writes nothing: the candidate values are
// fed straight into the same pure engine function the real save uses,
// never written to the database.
export function previewSettingsImpact(
  candidate: CandidateSettings,
): Promise<SettingsImpactPreview> {
  return invokeCommand("preview_settings_impact", { candidate });
}
