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
}

export interface SettingsImpactPreview {
  rewardsBefore: number;
  rewardsAfter: number;
  affectedMembers: MemberImpact[];
}

// API-33 — the only M3 command. Writes nothing; the engine swaps candidate
// settings in, recomputes, and restores them in a finally block.
export function previewSettingsImpact(
  candidate: CandidateSettings,
): Promise<SettingsImpactPreview> {
  return invokeCommand("preview_settings_impact", { candidate });
}
