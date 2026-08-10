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

// API-02 — Rule-34: a phone matching an inactive member is not an error,
// it's a reactivation offer. No member is created in that case; the caller
// decides whether to follow through (editMember + reactivateMember, S5).
export type AddMemberOutcome =
  | { status: "created"; member: Member }
  | { status: "reactivation_offer"; existingMember: Member };

export function addMember(input: AddMemberInput): Promise<AddMemberOutcome> {
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
