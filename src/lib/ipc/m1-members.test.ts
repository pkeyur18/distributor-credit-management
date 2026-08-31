// @vitest-environment jsdom
import { afterEach, describe, expect, it } from "vitest";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import {
  createRootMember,
  addMember,
  editMember,
  deactivateMember,
  reactivateMember,
  searchMembers,
} from "./m1-members";

afterEach(() => {
  clearMocks();
});

describe("m1-members IPC wrappers", () => {
  it("createRootMember sends its struct nested under `input`", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("create_root_member");
      expect(payload).toEqual({
        input: { name: "Root", phone: "9876500000", address: "HQ", consentGiven: true },
      });
      return { id: 100001 };
    });
    await createRootMember({ name: "Root", phone: "9876500000", address: "HQ", consentGiven: true });
  });

  it("addMember sends its struct nested under `input`", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("add_member");
      expect(payload).toEqual({
        input: {
          name: "Asha",
          phone: "9876543210",
          address: "1 Main St",
          consentGiven: true,
          introducerMemberId: 100001,
        },
      });
      return { status: "created", member: {}, warnings: [] };
    });
    await addMember({
      name: "Asha",
      phone: "9876543210",
      address: "1 Main St",
      consentGiven: true,
      introducerMemberId: 100001,
    });
  });

  it("editMember sends its struct nested under `input`", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("edit_member");
      expect(payload).toEqual({ input: { id: 284913, name: "Asha P." } });
      return {};
    });
    await editMember({ id: 284913, name: "Asha P." });
  });

  it("deactivateMember sends a flat scalar `id`", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("deactivate_member");
      expect(payload).toEqual({ id: 284913 });
      return null;
    });
    await deactivateMember(284913);
  });

  it("reactivateMember sends a flat scalar `id`", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("reactivate_member");
      expect(payload).toEqual({ id: 284913 });
      return null;
    });
    await reactivateMember(284913);
  });

  it("searchMembers sends flat `query`/`activeOnly`, defaulting activeOnly to false", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("search_members");
      expect(payload).toEqual({ query: "Asha", activeOnly: false });
      return [];
    });
    await searchMembers("Asha");
  });

  it("searchMembers passes activeOnly through when set", async () => {
    mockIPC((_cmd, payload) => {
      expect(payload).toEqual({ query: "Asha", activeOnly: true });
      return [];
    });
    await searchMembers("Asha", true);
  });
});
