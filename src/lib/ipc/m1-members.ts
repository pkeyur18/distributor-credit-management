import { invokeCommand } from "./client";
import type { Member, SearchResult } from "./entities";

export interface CreateRootMemberInput {
  name: string;
  phone: string;
  address: string;
  email?: string;
  consentGiven: boolean;
}

// API-01
export function createRootMember(input: CreateRootMemberInput): Promise<Member> {
  return invokeCommand("create_root_member", { ...input });
}

export interface AddMemberInput extends CreateRootMemberInput {
  introducerMemberId: number;
}

// API-02
export function addMember(input: AddMemberInput): Promise<Member> {
  return invokeCommand("add_member", { ...input });
}

export interface EditMemberInput {
  id: number;
  name?: string;
  phone?: string;
  email?: string | null;
  address?: string;
}

// API-03 — introducerMemberId is deliberately not an accepted field (Rule-37).
export function editMember(input: EditMemberInput): Promise<Member> {
  return invokeCommand("edit_member", { ...input });
}

// API-04 — triggers no recalculation; is_active has zero computational effect.
export function deactivateMember(id: number): Promise<void> {
  return invokeCommand("deactivate_member", { id });
}

// API-05
export function reactivateMember(id: number): Promise<void> {
  return invokeCommand("reactivate_member", { id });
}

// API-06 — shared with M4's search box; empty query returns an empty result, not an error.
export function searchMembers(query: string): Promise<SearchResult[]> {
  return invokeCommand("search_members", { query });
}
