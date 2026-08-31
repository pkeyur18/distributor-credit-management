// @vitest-environment jsdom
import { afterEach, describe, expect, it } from "vitest";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { previewSettingsImpact } from "./m3-calc";

afterEach(() => {
  clearMocks();
});

describe("m3-calc IPC wrappers", () => {
  it("previewSettingsImpact sends the candidate nested under `candidate`, not `input`", async () => {
    mockIPC((cmd, payload) => {
      expect(cmd).toBe("preview_settings_impact");
      expect(payload).toEqual({
        candidate: { royaltyQualifyingCount: 4 },
      });
      return {
        rewardsBefore: 0,
        rewardsAfter: 0,
        royaltyEarnerCountBefore: 0,
        royaltyEarnerCountAfter: 0,
        affectedMembers: [],
      };
    });
    await previewSettingsImpact({ royaltyQualifyingCount: 4 });
  });
});
