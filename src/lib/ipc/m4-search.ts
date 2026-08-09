import { invokeCommand } from "./client";
import type { ChartNode, Member } from "./entities";

export interface RewardBreakdown {
  ownReward: { ownBusinessVolume: number; ownSlabPct: number; amount: number };
  differentials: Array<{
    childId: number;
    childName: string;
    childTotalBusinessVolume: number;
    childSlabPct: number;
    ownSlabPct: number;
    differentialPct: number;
    amount: number;
  }>;
  royalty: { qualifyingChildren: number; ratePercent: number; amount: number } | null;
  rewardsTotal: number;
}

export interface MemberDetail {
  member: Member;
  totalBusinessVolume: number;
  slabPct: number;
  legCount: number;
  rewards: RewardBreakdown;
  directChildren: ChartNode[];
}

// API-10
export function getMemberDetail(memberId: number): Promise<MemberDetail> {
  return invokeCommand("get_member_detail", { memberId });
}

export interface DirectChildrenChartRequest {
  memberId: number;
  /** false: member + direct children (FR-2). true: entire subtree (FR-10). */
  fullTree: boolean;
}

// API-11 — same node shape either way: name, ID, own Business Volume, active
// flag, introducer link. Never Total Business Volume (FR-2's constraint).
export function getDirectChildrenChart(request: DirectChildrenChartRequest): Promise<ChartNode[]> {
  return invokeCommand("get_direct_children_chart", { ...request });
}
